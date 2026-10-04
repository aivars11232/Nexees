//! The window-to-host local IPC channel (PD-LOCAL-IPC, RC-23, IF-ATTACH).
//!
//! The Desktop host listens on a Unix socket, `host.sock`, in a folder only its user can enter:
//! `nexees` inside the session's runtime folder (`$XDG_RUNTIME_DIR`), mode 0700, with the socket
//! at mode 0600. Both folders are checked before use: a real folder, not a link, owned by this
//! user and closed to everyone else. Every connection is checked before a byte is read: the user
//! the kernel reports for the peer process (`SO_PEERCRED`) must be the expected one, so a process
//! of another user is refused even if it could reach the socket. Both ends check the other.
//!
//! A message is a frame: four bytes of big-endian length, then that many bytes of JSON. The
//! length is checked against [`MAX_IPC_MESSAGE_BYTES`] before the body is read, and the body is
//! decoded strictly by the protocol crate (`binding.sec_max_ipc_message`). Waiting is a blocking
//! read with the caller's timeout; nothing polls (RC-08).
//!
//! This module moves frames and checks identities. What the messages mean, and what a window may
//! ask for, is the host's business.

use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use nexees_protocol::serialization::MAX_IPC_MESSAGE_BYTES;
use nexees_protocol::serialization::{ProtocolError, decode, encode};
use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt::PeerCredentials;
use nix::unistd::{Uid, geteuid};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// The channel folder's name inside the runtime folder.
const FOLDER: &str = "nexees";
/// The socket's name inside the channel folder.
const SOCKET: &str = "host.sock";

/// Why the channel could not be opened or used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError {
    /// The session has no usable runtime folder: no `XDG_RUNTIME_DIR`, or one that is not this
    /// user's private folder. Outside a login session the host does not run (RC-05).
    NoRuntimeFolder(String),
    /// The channel folder or the socket is not private to this user.
    NotPrivate(String),
    /// The peer process belongs to another user, by the kernel's account (RC-23).
    ForeignPeer {
        /// The peer's user ID.
        uid: u32,
    },
    /// A frame longer than the channel allows; it was refused before its body was read.
    TooLarge {
        /// Its length in bytes.
        size: usize,
        /// The channel's limit in bytes.
        limit: usize,
    },
    /// A frame that is not exactly one valid message. The text never quotes the input.
    Malformed(String),
    /// The other end closed the channel.
    Closed,
    /// Nothing arrived within the timeout.
    TimedOut,
    /// Any other failure of the operating system, with its reason.
    Io(String),
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRuntimeFolder(reason) => write!(f, "no usable runtime folder: {reason}"),
            Self::NotPrivate(reason) => write!(f, "the channel is not private: {reason}"),
            Self::ForeignPeer { uid } => write!(f, "refused a peer of another user ({uid})"),
            Self::TooLarge { size, limit } => {
                write!(f, "a frame of {size} bytes exceeds the limit of {limit}")
            }
            Self::Malformed(reason) => write!(f, "malformed message: {reason}"),
            Self::Closed => write!(f, "the channel is closed"),
            Self::TimedOut => write!(f, "no answer within the timeout"),
            Self::Io(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for IpcError {}

impl From<io::Error> for IpcError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => Self::TimedOut,
            io::ErrorKind::UnexpectedEof
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted => Self::Closed,
            _ => Self::Io(error.to_string()),
        }
    }
}

impl From<ProtocolError> for IpcError {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::TooLarge { size, limit } => Self::TooLarge { size, limit },
            ProtocolError::Malformed(reason) => Self::Malformed(reason),
        }
    }
}

/// The channel's folder: `nexees` in the session's runtime folder, private to this user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelFolder {
    path: PathBuf,
    uid: Uid,
}

impl ChannelFolder {
    /// The channel folder inside `runtime`, created when it is missing. `runtime` must be an
    /// absolute path to a real folder owned by this user and closed to everyone else, as the
    /// login manager creates `$XDG_RUNTIME_DIR`; anything else is refused, never repaired.
    pub fn open(runtime: &Path) -> Result<Self, IpcError> {
        let uid = geteuid();
        private_folder(runtime, uid).map_err(IpcError::NoRuntimeFolder)?;
        let path = runtime.join(FOLDER);
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        private_folder(&path, uid).map_err(IpcError::NotPrivate)?;
        Ok(Self { path, uid })
    }

    /// The channel folder of this session, from `XDG_RUNTIME_DIR`.
    pub fn from_environment() -> Result<Self, IpcError> {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| IpcError::NoRuntimeFolder("XDG_RUNTIME_DIR is not set".into()))?;
        Self::open(Path::new(&runtime))
    }

    /// The folder itself.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The host's socket.
    pub fn socket(&self) -> PathBuf {
        self.path.join(SOCKET)
    }

    /// The user the folder belongs to: this process's user.
    pub fn uid(&self) -> Uid {
        self.uid
    }
}

/// Whether `path` is an absolute path to a real folder owned by `uid` and closed to every other
/// user; the reason when it is not.
fn private_folder(path: &Path, uid: Uid) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("{} is not an absolute path", path.display()));
    }
    let meta =
        fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !meta.file_type().is_dir() {
        return Err(format!("{} is not a folder", path.display()));
    }
    if meta.uid() != uid.as_raw() {
        return Err(format!("{} belongs to another user", path.display()));
    }
    if meta.mode() & 0o077 != 0 {
        return Err(format!("{} is open to other users", path.display()));
    }
    Ok(())
}

/// The host's end: it listens on the channel's socket and admits only peers of one user.
#[derive(Debug)]
pub struct Listener {
    listener: UnixListener,
    socket: PathBuf,
    allowed: Uid,
}

impl Listener {
    /// Listens in `folder` for peers of this process's user. Only the holder of the host lock
    /// may call this: a socket left behind by a host that stopped is replaced.
    pub fn bind(folder: &ChannelFolder) -> Result<Self, IpcError> {
        Self::bind_for(folder, folder.uid)
    }

    /// As [`Listener::bind`], admitting only peers of `allowed`. Tests use another user to show
    /// that a peer of the wrong user is refused.
    pub fn bind_for(folder: &ChannelFolder, allowed: Uid) -> Result<Self, IpcError> {
        let socket = folder.socket();
        match fs::symlink_metadata(&socket) {
            Ok(meta) if meta.file_type().is_socket() => fs::remove_file(&socket)?,
            Ok(_) => {
                return Err(IpcError::NotPrivate(format!(
                    "{} is not a socket",
                    socket.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let listener = UnixListener::bind(&socket)?;
        // The folder is closed to others already; the socket is closed as well.
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
        Ok(Self {
            listener,
            socket,
            allowed,
        })
    }

    /// The next connection from a peer of the allowed user. A peer of another user is refused,
    /// and its connection closed, before anything is read from it.
    pub fn accept(&self) -> Result<Connection, IpcError> {
        let (stream, _) = self.listener.accept()?;
        let connection = Connection { stream };
        connection.check_peer(self.allowed)?;
        Ok(connection)
    }

    /// Removes the socket, so that no window reaches a host that is stopping.
    pub fn remove_socket(&self) -> Result<(), IpcError> {
        match fs::remove_file(&self.socket) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
}

/// One end of an admitted channel, carrying bounded frames.
#[derive(Debug)]
pub struct Connection {
    stream: UnixStream,
}

impl Connection {
    /// Connects to the host listening in `folder`, after checking that its socket is private to
    /// this user, and checks that the process that answers belongs to this user too.
    pub fn connect(folder: &ChannelFolder) -> Result<Self, IpcError> {
        let socket = folder.socket();
        let meta = fs::symlink_metadata(&socket)?;
        if !meta.file_type().is_socket()
            || meta.uid() != folder.uid.as_raw()
            || meta.mode() & 0o077 != 0
        {
            return Err(IpcError::NotPrivate(format!(
                "{} is not this user's private socket",
                socket.display()
            )));
        }
        let connection = Self {
            stream: UnixStream::connect(&socket)?,
        };
        connection.check_peer(folder.uid)?;
        Ok(connection)
    }

    /// Fails unless the kernel reports the peer process as one of `allowed`'s.
    fn check_peer(&self, allowed: Uid) -> Result<(), IpcError> {
        let peer = getsockopt(&self.stream, PeerCredentials)
            .map_err(|error| IpcError::Io(format!("peer credentials: {error}")))?;
        if peer.uid() == allowed.as_raw() {
            Ok(())
        } else {
            self.close();
            Err(IpcError::ForeignPeer { uid: peer.uid() })
        }
    }

    /// Bounds how long one read or write may wait; `None` waits until the peer acts or closes.
    pub fn set_timeout(&self, timeout: Option<Duration>) -> Result<(), IpcError> {
        self.stream.set_read_timeout(timeout)?;
        self.stream.set_write_timeout(timeout)?;
        Ok(())
    }

    /// Sends `message` as one frame; refused when it is larger than the channel allows.
    pub fn send<T: Serialize>(&mut self, message: &T) -> Result<(), IpcError> {
        let body = encode(message, MAX_IPC_MESSAGE_BYTES)?;
        let length = u32::try_from(body.len()).map_err(|_| IpcError::TooLarge {
            size: body.len(),
            limit: MAX_IPC_MESSAGE_BYTES,
        })?;
        let mut frame = Vec::with_capacity(4 + body.len());
        frame.extend_from_slice(&length.to_be_bytes());
        frame.extend_from_slice(&body);
        self.stream.write_all(&frame)?;
        Ok(())
    }

    /// Receives the next frame as exactly one `T`. A frame longer than the channel allows is
    /// refused before its body is read; the channel is then unusable and should be closed.
    pub fn receive<T: DeserializeOwned>(&mut self) -> Result<T, IpcError> {
        let mut header = [0_u8; 4];
        self.stream.read_exact(&mut header)?;
        let size = usize::try_from(u32::from_be_bytes(header)).unwrap_or(usize::MAX);
        if size > MAX_IPC_MESSAGE_BYTES {
            return Err(IpcError::TooLarge {
                size,
                limit: MAX_IPC_MESSAGE_BYTES,
            });
        }
        let mut body = vec![0_u8; size];
        self.stream.read_exact(&mut body)?;
        Ok(decode(&body, MAX_IPC_MESSAGE_BYTES)?)
    }

    /// Closes both directions; the peer's next read reports the channel closed.
    pub fn close(&self) {
        // Closing a stream the peer already closed is not a failure worth reporting.
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    use nexees_protocol::messages::{ClientMessage, HostMessage};
    use nexees_protocol::version_negotiation::Hello;

    /// A private runtime folder for one test, as the login manager would create it; removed when
    /// the test ends.
    pub(crate) struct Runtime(PathBuf);

    impl Runtime {
        pub(crate) fn new(name: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let unique = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("nexees-ipc-{}-{unique}-{name}", std::process::id()));
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            Self(path)
        }

        pub(crate) fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Runtime {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().mode() & 0o777
    }

    #[test]
    fn the_channel_lives_in_folders_private_to_the_user() {
        let runtime = Runtime::new("private");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        assert_eq!(mode(folder.path()), 0o700);
        let listener = Listener::bind(&folder).unwrap();
        assert_eq!(mode(&folder.socket()), 0o600);
        drop(listener);

        // A runtime folder open to others, a relative path and a link are refused, not repaired.
        let open = Runtime::new("open");
        fs::set_permissions(open.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            ChannelFolder::open(open.path()),
            Err(IpcError::NoRuntimeFolder(_))
        ));
        assert!(matches!(
            ChannelFolder::open(Path::new("relative/runtime")),
            Err(IpcError::NoRuntimeFolder(_))
        ));
        let linked = Runtime::new("linked");
        let link = linked.path().join("runtime");
        std::os::unix::fs::symlink(runtime.path(), &link).unwrap();
        assert!(matches!(
            ChannelFolder::open(&link),
            Err(IpcError::NoRuntimeFolder(_))
        ));
        // A channel folder someone opened up is refused too.
        let widened = Runtime::new("widened");
        fs::DirBuilder::new()
            .mode(0o755)
            .create(widened.path().join(FOLDER))
            .unwrap();
        fs::set_permissions(
            widened.path().join(FOLDER),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert!(matches!(
            ChannelFolder::open(widened.path()),
            Err(IpcError::NotPrivate(_))
        ));
    }

    #[test]
    fn messages_travel_as_bounded_frames_between_the_two_ends() {
        let runtime = Runtime::new("frames");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        let listener = Listener::bind(&folder).unwrap();
        let host = thread::spawn(move || {
            let mut window = listener.accept().unwrap();
            let hello: ClientMessage = window.receive().unwrap();
            assert_eq!(hello, ClientMessage::Hello(Hello::current()));
            window.send(&HostMessage::Hello(Hello::current())).unwrap();
        });
        let mut connection = Connection::connect(&folder).unwrap();
        connection
            .send(&ClientMessage::Hello(Hello::current()))
            .unwrap();
        let answer: HostMessage = connection.receive().unwrap();
        assert_eq!(answer, HostMessage::Hello(Hello::current()));
        host.join().unwrap();
        // The host is gone: the next read reports the channel closed.
        assert_eq!(
            connection.receive::<HostMessage>().unwrap_err(),
            IpcError::Closed
        );
    }

    #[test]
    fn a_frame_longer_than_the_channel_allows_is_refused_unread() {
        let runtime = Runtime::new("large");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        let listener = Listener::bind(&folder).unwrap();
        let host = thread::spawn(move || {
            let mut window = listener.accept().unwrap();
            // Bounded, so a frame that is wrongly waited for fails the test instead of hanging it.
            window.set_timeout(Some(Duration::from_secs(5))).unwrap();
            window.receive::<ClientMessage>().unwrap_err()
        });
        let connection = Connection::connect(&folder).unwrap();
        // A header announcing one byte over the limit, and no body at all: refusing it does not
        // wait for the body.
        let size = u32::try_from(MAX_IPC_MESSAGE_BYTES + 1).unwrap();
        (&connection.stream).write_all(&size.to_be_bytes()).unwrap();
        assert_eq!(
            host.join().unwrap(),
            IpcError::TooLarge {
                size: MAX_IPC_MESSAGE_BYTES + 1,
                limit: MAX_IPC_MESSAGE_BYTES
            }
        );
        // The sending side refuses an oversized message before writing anything.
        let mut sender = Connection {
            stream: connection.stream.try_clone().unwrap(),
        };
        let oversized = "x".repeat(MAX_IPC_MESSAGE_BYTES);
        assert!(matches!(
            sender.send(&oversized),
            Err(IpcError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_frame_that_is_not_one_valid_message_is_refused() {
        let runtime = Runtime::new("malformed");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        let listener = Listener::bind(&folder).unwrap();
        let host = thread::spawn(move || {
            let mut window = listener.accept().unwrap();
            // Bounded, so a frame that is wrongly waited for fails the test instead of hanging it.
            window.set_timeout(Some(Duration::from_secs(5))).unwrap();
            window.receive::<ClientMessage>().unwrap_err()
        });
        let connection = Connection::connect(&folder).unwrap();
        let body = br#"{"hello":{"versions":{"min":1,"max":1}},"extra":1}"#;
        let size = u32::try_from(body.len()).unwrap();
        (&connection.stream).write_all(&size.to_be_bytes()).unwrap();
        (&connection.stream).write_all(body).unwrap();
        assert!(matches!(host.join().unwrap(), IpcError::Malformed(_)));
    }

    #[test]
    fn a_peer_of_another_user_is_refused_before_anything_is_read() {
        let runtime = Runtime::new("foreign");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        // The listener admits only another user, so this test's own process is the foreigner.
        let other = Uid::from_raw(folder.uid().as_raw().wrapping_add(1));
        let listener = Listener::bind_for(&folder, other).unwrap();
        let host = thread::spawn(move || listener.accept().unwrap_err());
        let mut connection = Connection::connect(&folder).unwrap();
        assert_eq!(
            host.join().unwrap(),
            IpcError::ForeignPeer {
                uid: geteuid().as_raw()
            }
        );
        assert_eq!(
            connection.receive::<HostMessage>().unwrap_err(),
            IpcError::Closed
        );
    }

    #[test]
    fn a_stale_socket_is_replaced_and_anything_else_in_its_place_is_refused() {
        let runtime = Runtime::new("stale");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        // A socket left behind by a host that stopped without removing it.
        drop(UnixListener::bind(folder.socket()).unwrap());
        let listener = Listener::bind(&folder).unwrap();
        listener.remove_socket().unwrap();
        assert!(!folder.socket().exists());
        // A regular file where the socket belongs is neither replaced nor connected to.
        fs::write(folder.socket(), b"not a socket").unwrap();
        assert!(matches!(
            Listener::bind(&folder),
            Err(IpcError::NotPrivate(_))
        ));
        assert!(matches!(
            Connection::connect(&folder),
            Err(IpcError::NotPrivate(_))
        ));
        assert_eq!(fs::read(folder.socket()).unwrap(), b"not a socket");
    }

    #[test]
    fn a_silent_peer_times_out_instead_of_holding_the_reader() {
        let runtime = Runtime::new("silent");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        let listener = Listener::bind(&folder).unwrap();
        let _connection = Connection::connect(&folder).unwrap();
        let mut window = listener.accept().unwrap();
        window.set_timeout(Some(Duration::from_millis(50))).unwrap();
        assert_eq!(
            window.receive::<ClientMessage>().unwrap_err(),
            IpcError::TimedOut
        );
    }
}
