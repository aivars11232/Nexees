//! Typed device requests over mutually authenticated TLS 1.3, and the one
//! dispatcher both devices use.
//!
//! Security comes from stock components only: rustls with the ring provider
//! verifies both certificates with its WebPKI verifiers, and each device trusts
//! exactly one peer certificate, recorded at pairing. There is no custom
//! cryptography and no custom certificate check. Whatever arrives is then
//! judged on the executing device by [`dispatch`]: protocol, device binding,
//! direction, the directional grant held by this device, expiry and replay,
//! in that order, before anything runs.

use crate::lcl_bridge::{Lcl, Verdict};
use crate::store::Store;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const PROTOCOL: u32 = 1;
/// Largest request or reply accepted, in bytes.
pub const MAX_FRAME: usize = 64 * 1024;
/// Both certificates name this; trust comes from the pinned certificate.
const SERVER_NAME: &str = "nexees-device";
const IO_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    PcToPhone,
    PhoneToPc,
}

impl Direction {
    /// The grant the executing device must hold for this direction. Each
    /// device keeps its own; neither direction implies the other.
    pub fn grant(self) -> &'static str {
        match self {
            Direction::PcToPhone => "pc_to_phone",
            Direction::PhoneToPc => "phone_to_pc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Create or edit a file in an isolated LCL workspace on the receiving
    /// device, save it as a new revision, then validate it with LCL.
    WriteLclFile {
        workspace: String,
        path: String,
        content: String,
        /// The revision the sender edited; `0` for a new file.
        expected_revision: i64,
    },
    /// Launch an allowlisted application by its allowlist key.
    LaunchApp { app: String },
    /// Report what the receiving device can do now, per action.
    Capabilities,
    /// Report the current state of an earlier command.
    Status { command: String },
}

impl Action {
    /// What the command store records: the action kind, plus the allowlist
    /// key for a launch so the platform launcher can act on it later.
    /// Payloads such as file content are not recorded here.
    fn label(&self) -> String {
        match self {
            Action::WriteLclFile { .. } => "write_lcl_file".to_string(),
            Action::LaunchApp { app } => format!("launch_app:{app}"),
            Action::Capabilities => "capabilities".to_string(),
            Action::Status { .. } => "status".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub protocol: u32,
    /// Unique per command; a repeated ID is a replay.
    pub id: String,
    pub from_device: String,
    pub to_device: String,
    pub direction: Direction,
    /// Unix seconds after which the command must not run.
    pub expires_at: u64,
    pub action: Action,
}

/// The separate states RC-12 requires. A transport success is never reported
/// as `completed` on its own.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Accepted,
    Running,
    Completed,
    /// The action ran and definitely did not succeed.
    Failed,
    Denied,
    Unsupported,
    NeedsUserAction,
    Expired,
    OutcomeUnknown,
    Duplicate,
    Conflict,
    Invalid,
    Unavailable,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Accepted => "accepted",
            Status::Running => "running",
            Status::Completed => "completed",
            Status::Failed => "failed",
            Status::Denied => "denied",
            Status::Unsupported => "unsupported",
            Status::NeedsUserAction => "needs_user_action",
            Status::Expired => "expired",
            Status::OutcomeUnknown => "outcome_unknown",
            Status::Duplicate => "duplicate",
            Status::Conflict => "conflict",
            Status::Invalid => "invalid",
            Status::Unavailable => "unavailable",
        }
    }

    pub fn parse(text: &str) -> Status {
        [
            Status::Accepted,
            Status::Running,
            Status::Completed,
            Status::Failed,
            Status::Denied,
            Status::Unsupported,
            Status::NeedsUserAction,
            Status::Expired,
            Status::OutcomeUnknown,
            Status::Duplicate,
            Status::Conflict,
            Status::Invalid,
            Status::Unavailable,
        ]
        .into_iter()
        .find(|s| s.as_str() == text)
        .unwrap_or(Status::OutcomeUnknown)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub action: String,
    /// `available`, `needs_user_action` or `unsupported`.
    pub availability: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub id: String,
    pub status: Status,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<Verdict>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
}

impl Reply {
    pub fn new(id: &str, status: Status, detail: impl Into<String>) -> Reply {
        Reply {
            id: id.to_string(),
            status,
            detail: detail.into(),
            revision: None,
            sha256: None,
            validation: None,
            capabilities: Vec::new(),
        }
    }
}

/// What differs between Desktop and Android. Everything else is shared.
pub trait Platform {
    /// Launches an allowlisted application, or records why it cannot, and
    /// returns the honest state with its reason.
    fn launch(&mut self, command: &str, app: &str, expires_at: u64) -> (Status, String);
    fn capabilities(&self) -> Vec<Capability>;
}

/// The receiving device.
pub struct Device<'a> {
    pub id: String,
    /// The one paired peer, already authenticated by its pinned certificate.
    pub peer: String,
    /// The direction of the requests this device executes.
    pub inbound: Direction,
    /// Remote writes are confined to workspaces under this folder.
    pub workspaces: PathBuf,
    pub lcl: &'a Lcl,
}

/// Judges and executes one request. Every remote request on either device
/// goes through here; there is no second path.
pub fn dispatch(device: &Device, store: &mut Store, platform: &mut dyn Platform, envelope: Envelope) -> Reply {
    let id = envelope.id.clone();
    let refuse = |status, detail: &str| Reply::new(&id, status, detail);
    if envelope.protocol != PROTOCOL {
        return refuse(Status::Unsupported, "unsupported protocol version");
    }
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return refuse(Status::Invalid, "malformed command ID");
    }
    if envelope.to_device != device.id || envelope.from_device != device.peer {
        return refuse(Status::Denied, "the request is not bound to this device and its paired peer");
    }
    if envelope.direction != device.inbound {
        return refuse(Status::Denied, "this device does not execute requests in that direction");
    }
    match store.grant(envelope.direction.grant()) {
        Ok(true) => {}
        Ok(false) => {
            return refuse(Status::Denied, &format!("the {} grant is absent or revoked", envelope.direction.grant()))
        }
        Err(e) => return refuse(Status::Invalid, &e.to_string()),
    }
    if envelope.expires_at <= crate::unix_now() {
        return refuse(Status::Expired, "the request expired before it ran");
    }
    match &envelope.action {
        Action::Status { command } => return status_of(store, &id, command),
        Action::Capabilities => {
            let mut reply = refuse(Status::Completed, "capabilities observed now");
            reply.capabilities = platform.capabilities();
            return reply;
        }
        _ => {}
    }
    match store.claim_command(&id, envelope.expires_at, &envelope.action.label()) {
        Ok(true) => {}
        Ok(false) => return refuse(Status::Duplicate, "this command ID was already received; it does not run again"),
        Err(e) => return refuse(Status::Invalid, &e.to_string()),
    }
    let reply = match envelope.action {
        Action::WriteLclFile { workspace, path, content, expected_revision } => {
            write_lcl_file(device, store, &id, &workspace, &path, &content, expected_revision)
        }
        Action::LaunchApp { app } => {
            let (status, detail) = platform.launch(&id, &app, envelope.expires_at);
            refuse(status, &detail)
        }
        Action::Status { .. } | Action::Capabilities => unreachable!("answered above"),
    };
    let _ = store.set_command_status(&id, reply.status.as_str(), &reply.detail);
    reply
}

fn status_of(store: &Store, id: &str, command: &str) -> Reply {
    match store.command(command) {
        Ok(Some(c)) => Reply::new(id, Status::parse(&c.status), c.detail),
        Ok(None) => Reply::new(id, Status::OutcomeUnknown, "no such command was received"),
        Err(e) => Reply::new(id, Status::Invalid, e.to_string()),
    }
}

fn write_lcl_file(
    device: &Device,
    store: &mut Store,
    id: &str,
    workspace: &str,
    path: &str,
    content: &str,
    expected_revision: i64,
) -> Reply {
    let plain = |s: &str| {
        !s.is_empty() && !s.starts_with('.') && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    let parts: Vec<&str> = path.split('/').collect();
    if !plain(workspace) || !path.ends_with(".lcl.txt") || !parts.iter().all(|p| plain(p)) {
        return Reply::new(id, Status::Denied, "the path is outside the allowed LCL workspace files");
    }
    let current = match store.latest(workspace, path) {
        Ok(latest) => latest.map(|(saved, _)| saved.revision).unwrap_or(0),
        Err(e) => return Reply::new(id, Status::Invalid, e.to_string()),
    };
    if current != expected_revision {
        return Reply::new(id, Status::Conflict, format!("the file is at revision {current}, not {expected_revision}"));
    }
    let target = device.workspaces.join(workspace).join(path);
    if let Err(e) = write_atomically(&target, content.as_bytes()) {
        return Reply::new(id, Status::OutcomeUnknown, format!("the write did not complete: {e}"));
    }
    let saved = match store.save_revision(workspace, path, content.as_bytes()) {
        Ok(saved) => saved,
        Err(e) => return Reply::new(id, Status::OutcomeUnknown, format!("the revision was not recorded: {e}")),
    };
    let mut reply = Reply::new(id, Status::Completed, "saved, then validated");
    reply.revision = Some(saved.revision);
    reply.sha256 = Some(saved.sha256);
    reply.validation = Some(device.lcl.validate(&target).unwrap_or_else(|e| Verdict {
        outcome: "refused".to_string(),
        reached: String::new(),
        terminal_status: None,
        primary: Some(e),
    }));
    reply
}

/// Writes a sibling temporary file, syncs it, then renames it into place, so
/// a reader sees the old file or the new one and never a torn write.
pub fn write_atomically(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = target.parent().ok_or_else(|| io::Error::other("no parent folder"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.partial", std::process::id()));
    let mut file = fs::OpenOptions::new().write(true).create(true).truncate(true).open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, target)?;
    fs::File::open(parent)?.sync_all()
}

/// This device's TLS identity: a self-signed certificate and its key.
pub struct Identity {
    pub certificate: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
}

impl Identity {
    /// Loads the identity in `dir`, creating it on first use. The key never
    /// leaves the device and its file is readable by the owner only.
    pub fn load_or_create(dir: &Path) -> io::Result<Identity> {
        let cert_path = dir.join("identity.cert.der");
        let key_path = dir.join("identity.key.der");
        if !cert_path.exists() {
            fs::create_dir_all(dir)?;
            // A random subject keeps every device certificate distinct.
            let mut unique = [0u8; 16];
            ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut unique)
                .map_err(|_| io::Error::other("no system randomness"))?;
            let mut params = rcgen::CertificateParams::new(vec![SERVER_NAME.to_string()]).map_err(io::Error::other)?;
            params.distinguished_name = rcgen::DistinguishedName::new();
            let unique: String = unique.iter().map(|b| format!("{b:02x}")).collect();
            params.distinguished_name.push(rcgen::DnType::CommonName, format!("nexees-device-{unique}"));
            let signing_key = rcgen::KeyPair::generate().map_err(io::Error::other)?;
            let certificate = params.self_signed(&signing_key).map_err(io::Error::other)?;
            let mut key = fs::OpenOptions::new();
            key.write(true).create_new(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut key, 0o600);
            key.open(&key_path)?.write_all(&signing_key.serialize_der())?;
            fs::write(&cert_path, certificate.der())?;
        }
        Ok(Identity {
            certificate: CertificateDer::from(fs::read(cert_path)?),
            key: PrivatePkcs8KeyDer::from(fs::read(key_path)?),
        })
    }

    pub fn fingerprint(&self) -> String {
        crate::sha256_hex(&self.certificate)
    }

    fn chain_and_key(&self) -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
        (vec![self.certificate.clone()], PrivateKeyDer::Pkcs8(self.key.clone_key()))
    }
}

pub fn read_certificate(path: &Path) -> io::Result<CertificateDer<'static>> {
    Ok(CertificateDer::from(fs::read(path)?))
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn pinned(peer: &CertificateDer<'static>) -> Result<Arc<RootCertStore>, String> {
    let mut roots = RootCertStore::empty();
    roots.add(peer.clone()).map_err(|e| e.to_string())?;
    Ok(Arc::new(roots))
}

/// TLS 1.3 only; the client must present the pinned peer certificate.
pub fn server_config(identity: &Identity, peer: &CertificateDer<'static>) -> Result<Arc<ServerConfig>, String> {
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(pinned(peer)?, provider())
        .build()
        .map_err(|e| e.to_string())?;
    let (chain, key) = identity.chain_and_key();
    let config = ServerConfig::builder_with_provider(provider())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .with_client_cert_verifier(verifier)
        .with_single_cert(chain, key)
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(config))
}

/// TLS 1.3 only; the server must present the pinned peer certificate.
pub fn client_config(identity: &Identity, peer: &CertificateDer<'static>) -> Result<Arc<ClientConfig>, String> {
    let (chain, key) = identity.chain_and_key();
    let config = ClientConfig::builder_with_provider(provider())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .with_root_certificates(pinned(peer)?)
        .with_client_auth_cert(chain, key)
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(config))
}

/// Why a request got no reply. Each maps to an honest state for the user.
#[derive(Debug)]
pub enum SendError {
    /// Nothing accepted the connection: no receiver is reachable.
    Unavailable(String),
    /// The TLS peer was not the paired device, or it refused this one.
    Rejected(String),
    /// The request may have been delivered but no reply arrived.
    OutcomeUnknown(String),
}

impl SendError {
    pub fn status(&self) -> Status {
        match self {
            SendError::Unavailable(_) => Status::Unavailable,
            SendError::Rejected(_) => Status::Denied,
            SendError::OutcomeUnknown(_) => Status::OutcomeUnknown,
        }
    }
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::Unavailable(e) => write!(f, "unavailable: {e}"),
            SendError::Rejected(e) => write!(f, "rejected by TLS: {e}"),
            SendError::OutcomeUnknown(e) => write!(f, "outcome unknown: {e}"),
        }
    }
}

fn is_tls_error(e: &io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<rustls::Error>())
}

/// Sends one request and waits for its reply.
pub fn send(address: &str, config: Arc<ClientConfig>, envelope: &Envelope) -> Result<Reply, SendError> {
    let socket_address = address
        .to_socket_addrs()
        .map_err(|e| SendError::Unavailable(e.to_string()))?
        .next()
        .ok_or_else(|| SendError::Unavailable(format!("{address} did not resolve")))?;
    let socket = TcpStream::connect_timeout(&socket_address, IO_TIMEOUT)
        .map_err(|e| SendError::Unavailable(e.to_string()))?;
    socket.set_read_timeout(Some(IO_TIMEOUT)).ok();
    socket.set_write_timeout(Some(IO_TIMEOUT)).ok();
    let name = ServerName::try_from(SERVER_NAME).map_err(|e| SendError::Rejected(e.to_string()))?;
    let connection = rustls::ClientConnection::new(config, name).map_err(|e| SendError::Rejected(e.to_string()))?;
    let mut stream = rustls::StreamOwned::new(connection, socket);
    let classify = |e: io::Error| if is_tls_error(&e) { SendError::Rejected(e.to_string()) } else { SendError::Unavailable(e.to_string()) };
    while stream.conn.is_handshaking() {
        stream.conn.complete_io(&mut stream.sock).map_err(classify)?;
    }
    write_frame(&mut stream, envelope).map_err(classify)?;
    // From here the request may have been delivered: a missing reply is an
    // unknown outcome, except a TLS alert, which means it was refused.
    let body = read_frame(&mut stream).map_err(|e| {
        if is_tls_error(&e) { SendError::Rejected(e.to_string()) } else { SendError::OutcomeUnknown(e.to_string()) }
    })?;
    serde_json::from_slice(&body).map_err(|e| SendError::OutcomeUnknown(e.to_string()))
}

/// Serves one request per connection until `stop` is set. Connections are
/// handled in turn, each with I/O timeouts and the frame limit, so one slow
/// peer cannot hold the receiver for longer than the timeout.
pub fn serve(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    stop: &AtomicBool,
    mut handle: impl FnMut(Envelope) -> Reply,
    mut log: impl FnMut(&str),
) -> io::Result<()> {
    listener.set_nonblocking(true)?;
    while !stop.load(Ordering::SeqCst) {
        let (socket, peer) = match listener.accept() {
            Ok(accepted) => accepted,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(e),
        };
        match answer(socket, config.clone(), &mut handle) {
            Ok(status) => log(&format!("{peer}: {}", status.as_str())),
            Err(e) => log(&format!("{peer}: connection refused or failed: {e}")),
        }
    }
    Ok(())
}

fn answer(socket: TcpStream, config: Arc<ServerConfig>, handle: &mut impl FnMut(Envelope) -> Reply) -> io::Result<Status> {
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(IO_TIMEOUT))?;
    socket.set_write_timeout(Some(IO_TIMEOUT))?;
    let connection = rustls::ServerConnection::new(config).map_err(io::Error::other)?;
    let mut stream = rustls::StreamOwned::new(connection, socket);
    let body = read_frame(&mut stream)?;
    let reply = match serde_json::from_slice::<Envelope>(&body) {
        Ok(envelope) => handle(envelope),
        Err(e) => Reply::new("", Status::Invalid, format!("not a request envelope: {e}")),
    };
    write_frame(&mut stream, &reply)?;
    stream.conn.send_close_notify();
    stream.conn.complete_io(&mut stream.sock)?;
    Ok(reply.status)
}

fn write_frame(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    if body.len() > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame exceeds the limit"));
    }
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn read_frame(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame exceeds the limit"));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(body)
}
