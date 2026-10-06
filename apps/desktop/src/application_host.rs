//! The Nexees Desktop local host: the composition root of HOST-DESKTOP (SS-LOCAL-HOST).
//!
//! One host runs per OS user, as a process separate from the window and at the user's privilege
//! (RC-04, RC-23). It holds the device's state store and serves its windows over the local IPC
//! channel. It is wiring, not domain logic: the rules live in the shared core and the platform
//! adapters.
//!
//! Commands, each answered by an exit code (listed in [`exit`]); the host prints nothing:
//!
//! - `nexees-host serve [--keep-running]` runs the host until it should stop.
//! - `nexees-host start` makes sure a host runs and answers, starting one only when none does,
//!   and returns once it answers. The window calls it before attaching (apps/desktop/src/main).
//!   Two windows that start at once still get one host: the second `serve` finds the host lock
//!   held and stops (RC-T14).
//! - `nexees-host status` tells whether a host answers.
//! - `nexees-host login-start enable|disable|status` manages the user-level start at login,
//!   which runs `serve --keep-running` (RC-05).
//!
//! **Serving a window.** A window must open with its hello. The host answers with its own and
//! keeps the window only when the two share a protocol version at or above the minimum secure
//! one; otherwise it closes the channel (RC-23). After that, a resync is answered with the host's
//! status, and an intent with an explicit `unsupported` outcome: this build carries out none yet
//! (RC-12). At most [`MAX_WINDOWS`] windows are attached at once.
//!
//! **The window's panels.** The host keeps the panel layout a window stores, in the state store,
//! and gives it back when a window asks: view state lives with the host, so a window that was
//! closed or stopped unexpectedly finds its panels as the user left them (ST-VIEW,
//! FD-UI-CLIENT). The layout is presentation only. The host checks its shape, keeps it under
//! [`WINDOW_CLIENT`] and reads nothing else into it; a window cannot name another client. A
//! window that agreed on a protocol version older than these messages is refused them, and when
//! the store cannot give or take the layout the host closes the window's channel instead of
//! answering with a guess.
//!
//! **The workspaces.** The host keeps the registry of its device's workspaces in the state store
//! (core/workspaces/workspace_registry) and answers a window that lists, creates, opens or closes
//! one. A new workspace gets its ID from sixteen random bytes. Opening a workspace makes it the
//! foreground workspace of [`WINDOW_CLIENT`], what the window shows; it binds no agent and
//! changes nothing else (C2, R11). A request the registry refuses is answered with the reason and
//! changes nothing; when the store fails, the host closes the window's channel. A list is sent in
//! pages that each fit one message of the channel.
//!
//! **Lifetime.** When the last window detaches, the host stops after [`GRACE`], so a window that
//! reloads reattaches to it. A host started with `--keep-running` stays: keeping the host after
//! the window closes is its own opt-in, and start at login is how the user gives it (RC-04). A
//! host outlives the window that started it, so it first closes every descriptor it inherited.
//!
//! **Bounds.** The host refuses to run as root (RC-23) and runs only inside a login session
//! (RC-05). It has no model provider at all: it waits on blocking reads and a condition
//! variable, and nothing polls (RC-08). The state store lives in the user's data folder,
//! `$XDG_DATA_HOME/nexees/state.db`.

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nexees_domain::client::{ClientLayout, PanelLayout};
use nexees_domain::errors::Outcome;
use nexees_domain::ids::{ClientId, DeviceId, WorkspaceId};
use nexees_domain::text::Note;
use nexees_domain::time::Timestamp;
use nexees_platform_desktop::lifecycle::application_lifecycle::{
    HostLock, LoginStart, LoginStartStatus, close_inherited_descriptors, elevated,
};
use nexees_platform_desktop::transport::local_ipc::{
    ChannelFolder, Connection, IpcError, Listener,
};
use nexees_protocol::messages::{
    Acknowledgment, ClientMessage, HostMessage, HostStatus, NewWorkspace, WorkspaceAnswer,
    WorkspaceEntry, WorkspaceRefusal, WorkspaceTarget,
};
use nexees_protocol::serialization::{MAX_IPC_MESSAGE_BYTES, encode};
use nexees_protocol::version_negotiation::{Hello, MIN_SECURE_VERSION, negotiate};
use nexees_state::state_store::{StateError, StateStore};
use nexees_workspaces::workspace_registry::{
    self as registry, RegistryError, RootProblem, Workspace,
};

/// How long the host stays after its last window detaches, so a reloading window reattaches.
const GRACE: Duration = Duration::from_secs(5);
/// Most windows attached at once; another one is refused.
const MAX_WINDOWS: usize = 8;
/// The client whose view state and foreground workspace the host keeps for its windows. Every
/// window of this user is that one client for now: windows are told apart only once they show
/// different workspaces side by side (TASK-014), with per-workspace view state (TASK-016).
const WINDOW_CLIENT: &str = "desktop-window";
/// How long a new window has to say hello, and how long `start` waits for one answer.
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
/// How long `start` waits for a host to answer.
const START_TIMEOUT: Duration = Duration::from_secs(15);

/// The exit codes of the commands.
pub mod exit {
    /// Done; for `status`, a host answers; for `login-start status`, start at login is enabled.
    pub const OK: u8 = 0;
    /// `status`: no host answers. `login-start status`: start at login is disabled.
    pub const NO: u8 = 1;
    /// The command line is not one of the commands.
    pub const USAGE: u8 = 2;
    /// `serve`: another host holds the lock, so this one does not start.
    pub const ALREADY_RUNNING: u8 = 3;
    /// No login session or no private channel folder: unsupported here (RC-05).
    pub const UNSUPPORTED: u8 = 4;
    /// The host refuses to run as root (RC-23).
    pub const ELEVATED: u8 = 5;
    /// The state store could not be opened, read or closed.
    pub const STORE: u8 = 6;
    /// `start`: no host answered in time.
    pub const NO_ANSWER: u8 = 7;
    /// Any other failure, such as another program's autostart entry of Nexees's name.
    pub const FAILED: u8 = 8;
}

fn main() -> ExitCode {
    let args: Option<Vec<String>> = env::args_os()
        .skip(1)
        .map(|a| a.into_string().ok())
        .collect();
    let args = args.unwrap_or_default();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    ExitCode::from(run(&args))
}

/// Runs one command.
fn run(args: &[&str]) -> u8 {
    if elevated() {
        return exit::ELEVATED;
    }
    match args {
        ["serve"] => serve(false),
        ["serve", "--keep-running"] => serve(true),
        ["start"] => start(),
        ["status"] => match ChannelFolder::from_environment() {
            Ok(folder) if answers(&folder) => exit::OK,
            Ok(_) => exit::NO,
            Err(_) => exit::UNSUPPORTED,
        },
        ["login-start", action] => login_start(action),
        _ => exit::USAGE,
    }
}

/// The host as a command: in this session's channel folder, with the store in the user's data
/// folder.
fn serve(keep_running: bool) -> u8 {
    // Before anything is opened: the host keeps no descriptor of whatever started it (RC-04).
    if close_inherited_descriptors().is_err() {
        return exit::FAILED;
    }
    let Ok(folder) = ChannelFolder::from_environment() else {
        return exit::UNSUPPORTED;
    };
    let Some(data) = data_folder() else {
        return exit::UNSUPPORTED;
    };
    serve_in(&folder, &data, keep_running, GRACE)
}

/// Runs the host in `folder`, with its store in `data`, until it should stop.
fn serve_in(folder: &ChannelFolder, data: &Path, keep_running: bool, grace: Duration) -> u8 {
    let lock = match HostLock::acquire(folder) {
        Ok(Some(lock)) => lock,
        Ok(None) => return exit::ALREADY_RUNNING,
        Err(_) => return exit::FAILED,
    };
    let Ok((store, device)) = open_store(data) else {
        return exit::STORE;
    };
    let Ok(listener) = Listener::bind(folder) else {
        return exit::FAILED;
    };
    let attached = Arc::new(Attached::default());
    let listener = Arc::new(listener);
    let store: SharedStore = Arc::new(Mutex::new(Some(store)));
    {
        let (listener, attached) = (Arc::clone(&listener), Arc::clone(&attached));
        let store = Arc::clone(&store);
        thread::spawn(move || accept_windows(&listener, &attached, &device, &store));
    }
    attached.wait_until_done(keep_running, grace);
    // No new window reaches a host that is stopping.
    let removed = listener.remove_socket();
    // Taken out of the windows' reach and closed in order; a window served after this finds no
    // store and is turned away.
    let closed = match lock_store(&store).take() {
        Some(store) => store.close(now()),
        None => Ok(()),
    };
    drop(lock);
    match (removed, closed) {
        (Ok(()), Ok(())) => exit::OK,
        (_, Err(_)) => exit::STORE,
        (Err(_), _) => exit::FAILED,
    }
}

/// The store in `data`, created when missing, and this device's identity, recorded on the first
/// run and kept from then on.
fn open_store(data: &Path) -> Result<(StateStore, DeviceId), StateError> {
    match fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(data)
    {
        Ok(()) => {}
        Err(error) => return Err(StateError::Database(error.to_string())),
    }
    let now = now();
    let (mut store, _) = StateStore::open(&data.join("state.db"), now)?;
    let device = match store.device_id()? {
        Some(device) => device,
        None => {
            let device = DeviceId::new(format!("desktop-{}", random_hex()?))?;
            store.write(now, |w| w.set_device_id(&device))?;
            device
        }
    };
    Ok((store, device))
}

/// Sixteen random bytes from the kernel, as hexadecimal.
fn random_hex() -> Result<String, StateError> {
    let mut bytes = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| StateError::Database(format!("no randomness: {error}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// The user's data folder for Nexees: `$XDG_DATA_HOME/nexees`, or `$HOME/.local/share/nexees`
/// when that is unset, as the XDG base directory specification says. Only absolute paths count.
fn data_folder() -> Option<PathBuf> {
    let absolute = |value: std::ffi::OsString| {
        let path = PathBuf::from(value);
        path.is_absolute().then_some(path)
    };
    env::var_os("XDG_DATA_HOME")
        .and_then(absolute)
        .or_else(|| {
            env::var_os("HOME")
                .and_then(absolute)
                .map(|home| home.join(".local/share"))
        })
        .map(|base| base.join("nexees"))
}

/// The host's one store, shared by the threads that serve its windows; `None` once the host is
/// stopping.
type SharedStore = Arc<Mutex<Option<StateStore>>>;

fn lock_store(store: &SharedStore) -> MutexGuard<'_, Option<StateStore>> {
    // The store commits or rolls back each write as a whole, so a window's thread that panicked
    // while holding the lock left it consistent.
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The panel layout the store keeps for the window client, if any.
fn kept_layout(store: &SharedStore) -> Result<Option<PanelLayout>, StateError> {
    let guard = lock_store(store);
    let store = guard.as_ref().ok_or(StateError::InUse)?;
    Ok(store
        .get::<ClientLayout>(WINDOW_CLIENT)?
        .map(|layout| layout.panels))
}

/// Keeps `panels` as the window client's layout, in place of the one kept before. It returns
/// once the store has it on disk.
fn keep_layout(
    store: &SharedStore,
    device: &DeviceId,
    panels: &PanelLayout,
) -> Result<(), StateError> {
    let layout = ClientLayout {
        client_id: ClientId::new(WINDOW_CLIENT)?,
        device_id: device.clone(),
        panels: panels.clone(),
    };
    let mut guard = lock_store(store);
    let store = guard.as_mut().ok_or(StateError::InUse)?;
    store.write(now(), |w| w.put(&layout))
}

/// Carries out a window's workspace request with the device's registry, for the window client.
/// A refusal is an answer; an error is the store's, and the window's channel then closes.
fn workspaces(
    store: &SharedStore,
    device: &DeviceId,
    request: ClientMessage,
) -> Result<WorkspaceAnswer, StateError> {
    let client = ClientId::new(WINDOW_CLIENT)?;
    let mut guard = lock_store(store);
    let store = guard.as_mut().ok_or(StateError::InUse)?;
    let done = match request {
        ClientMessage::ListWorkspaces { after } => return page(store, device, &client, after),
        ClientMessage::CreateWorkspace(new) => create_workspace(store, device, new).map(Some),
        ClientMessage::OpenWorkspace(target) => {
            match open_workspace(store, device, &client, target) {
                Ok(None) => {
                    return Ok(WorkspaceAnswer::Refused {
                        reason: WorkspaceRefusal::NotFound,
                        workspace_id: None,
                    });
                }
                opened => opened,
            }
        }
        ClientMessage::CloseWorkspace { workspace_id } => {
            registry::close(store, now(), &client, &workspace_id).map(|()| None)
        }
        _ => return Err(StateError::Database("not a workspace request".into())),
    };
    match done {
        Ok(workspace) => Ok(WorkspaceAnswer::Done {
            workspace: workspace.as_ref().map(entry),
            foreground: registry::foreground(store, &client).map_err(store_error)?,
        }),
        Err(error) => refused(error),
    }
}

/// Creates the workspace the window describes, under a new ID of sixteen random bytes.
fn create_workspace(
    store: &mut StateStore,
    device: &DeviceId,
    new: NewWorkspace,
) -> Result<Workspace, RegistryError> {
    let id = WorkspaceId::new(format!("ws-{}", random_hex()?))?;
    let root = Path::new(new.root.as_str());
    registry::create(store, now(), device, id, new.kind, new.name, root)
}

/// Opens the workspace the window names, by its ID or by its root; none when no workspace has
/// the root it names.
fn open_workspace(
    store: &mut StateStore,
    device: &DeviceId,
    client: &ClientId,
    target: WorkspaceTarget,
) -> Result<Option<Workspace>, RegistryError> {
    let id = match target {
        WorkspaceTarget::WorkspaceId(id) => id,
        WorkspaceTarget::Root(root) => {
            match registry::find_by_root(store, device, Path::new(root.as_str()))? {
                Some(found) => found.id().clone(),
                None => return Ok(None),
            }
        }
    };
    registry::open(store, now(), device, client, &id).map(Some)
}

/// The workspaces after `after`, in ID order, as many as one message to a window holds. A
/// workspace that does not fit a message on its own is left out; none the registry makes is that
/// large.
fn page(
    store: &StateStore,
    device: &DeviceId,
    client: &ClientId,
    after: Option<WorkspaceId>,
) -> Result<WorkspaceAnswer, StateError> {
    let foreground = registry::foreground(store, client).map_err(store_error)?;
    let mut entries: Vec<WorkspaceEntry> = Vec::new();
    let mut more = false;
    let later = |workspace: &&Workspace| {
        after
            .as_ref()
            .is_none_or(|a| workspace.id().as_str() > a.as_str())
    };
    for workspace in registry::list(store, device)
        .map_err(store_error)?
        .iter()
        .filter(later)
    {
        entries.push(entry(workspace));
        let answer = WorkspaceAnswer::Page {
            entries: entries.clone(),
            more: true,
            foreground: foreground.clone(),
        };
        if encode(&HostMessage::Workspaces(answer), MAX_IPC_MESSAGE_BYTES).is_err() {
            entries.pop();
            if !entries.is_empty() {
                more = true;
                break;
            }
        }
    }
    Ok(WorkspaceAnswer::Page {
        entries,
        more,
        foreground,
    })
}

/// A workspace as a window is told about it.
fn entry(workspace: &Workspace) -> WorkspaceEntry {
    let replica = workspace.replica.fields();
    WorkspaceEntry {
        workspace_id: workspace.record.workspace_id.clone(),
        kind: workspace.record.kind,
        name: workspace.record.name.clone(),
        root: replica.root.clone(),
        availability: replica.availability,
    }
}

/// The answer to a request the registry refused; an error of the store stays an error.
fn refused(error: RegistryError) -> Result<WorkspaceAnswer, StateError> {
    let (reason, workspace_id) = match error {
        RegistryError::NotFound(_) => (WorkspaceRefusal::NotFound, None),
        RegistryError::Root(problem) => (
            match problem {
                RootProblem::NotAbsolute => WorkspaceRefusal::RootNotAbsolute,
                RootProblem::Missing => WorkspaceRefusal::RootMissing,
                RootProblem::NotADirectory => WorkspaceRefusal::RootNotAFolder,
                RootProblem::Unusable => WorkspaceRefusal::RootUnusable,
                RootProblem::Moved => WorkspaceRefusal::RootMoved,
            },
            None,
        ),
        RegistryError::Overlaps(other) => (WorkspaceRefusal::Overlaps, Some(other)),
        RegistryError::NotHere(_) => (WorkspaceRefusal::NotHere, None),
        RegistryError::NotOpen(_) => (WorkspaceRefusal::NotOpen, None),
        // A new ID taken already, or a value the decoded message could not hold: not the window's
        // to answer for.
        error
        @ (RegistryError::Store(_) | RegistryError::Invalid(_) | RegistryError::Exists(_)) => {
            return Err(store_error(error));
        }
    };
    Ok(WorkspaceAnswer::Refused {
        reason,
        workspace_id,
    })
}

fn store_error(error: RegistryError) -> StateError {
    match error {
        RegistryError::Store(error) => error,
        other => StateError::Database(other.to_string()),
    }
}

/// The windows attached to the host, and since when none has been.
#[derive(Default)]
struct Attached {
    state: Mutex<Windows>,
    changed: Condvar,
}

struct Windows {
    count: usize,
    idle_since: Option<Instant>,
}

impl Default for Windows {
    fn default() -> Self {
        // A new host has no window yet; the one that asked for it attaches at once.
        Self {
            count: 0,
            idle_since: Some(Instant::now()),
        }
    }
}

impl Attached {
    fn lock(&self) -> MutexGuard<'_, Windows> {
        // A window's thread that panicked still leaves a consistent count.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Counts a new window in; `false` when the bound is reached and it must be refused.
    fn attach(&self) -> bool {
        let mut windows = self.lock();
        if windows.count >= MAX_WINDOWS {
            return false;
        }
        windows.count += 1;
        windows.idle_since = None;
        drop(windows);
        self.changed.notify_all();
        true
    }

    /// Counts a window out.
    fn detach(&self) {
        let mut windows = self.lock();
        windows.count = windows.count.saturating_sub(1);
        if windows.count == 0 {
            windows.idle_since = Some(Instant::now());
        }
        drop(windows);
        self.changed.notify_all();
    }

    /// Returns when the host should stop: when no window has been attached for `grace`, unless
    /// it keeps running. It sleeps on the condition variable; nothing polls.
    fn wait_until_done(&self, keep_running: bool, grace: Duration) {
        let mut windows = self.lock();
        loop {
            match windows.idle_since {
                Some(since) if !keep_running => {
                    let idle = since.elapsed();
                    if idle >= grace {
                        return;
                    }
                    windows = self
                        .changed
                        .wait_timeout(windows, grace - idle)
                        .unwrap_or_else(PoisonError::into_inner)
                        .0;
                }
                _ => {
                    windows = self
                        .changed
                        .wait(windows)
                        .unwrap_or_else(PoisonError::into_inner);
                }
            }
        }
    }
}

/// Accepts windows for as long as the host runs, each on its own thread.
fn accept_windows(
    listener: &Listener,
    attached: &Arc<Attached>,
    device: &DeviceId,
    store: &SharedStore,
) {
    loop {
        let window = match listener.accept() {
            Ok(window) => window,
            // A peer of another user is refused inside accept; nothing was attached.
            Err(IpcError::ForeignPeer { .. }) => continue,
            Err(_) => {
                // A failing accept, such as one out of file handles, is retried after a pause
                // instead of spinning.
                thread::sleep(Duration::from_millis(200));
                continue;
            }
        };
        if !attached.attach() {
            window.close();
            continue;
        }
        let (attached, device, store) = (Arc::clone(attached), device.clone(), Arc::clone(store));
        thread::spawn(move || {
            // However the window leaves, it is counted out.
            let _ = attend(window, &device, &store);
            attached.detach();
        });
    }
}

/// Serves one window until it detaches: the hellos, then its resyncs, its intents, its panel
/// layout and its workspace requests.
fn attend(mut window: Connection, device: &DeviceId, store: &SharedStore) -> Result<(), IpcError> {
    window.set_timeout(Some(HELLO_TIMEOUT))?;
    let ClientMessage::Hello(theirs) = window.receive()? else {
        // A window opens with its hello; anything else is refused.
        window.close();
        return Ok(());
    };
    let ours = Hello::current();
    window.send(&HostMessage::Hello(ours))?;
    let Ok(agreed) = negotiate(&ours, &theirs, MIN_SECURE_VERSION) else {
        // A version mismatch after an update is refused, never bridged (RC-23).
        window.close();
        return Ok(());
    };
    // Attached: the window may stay quiet for as long as it is open.
    window.set_timeout(None)?;
    loop {
        let message: ClientMessage = window.receive()?;
        if message.since() > agreed.get() {
            // The two agreed on a version that does not have this message.
            window.close();
            return Ok(());
        }
        match message {
            ClientMessage::Hello(_) => {
                window.close();
                return Ok(());
            }
            ClientMessage::LoadLayout {} => {
                let Ok(panels) = kept_layout(store) else {
                    window.close();
                    return Ok(());
                };
                window.send(&HostMessage::Layout(panels))?;
            }
            ClientMessage::StoreLayout(panels) => {
                if keep_layout(store, device, &panels).is_err() {
                    window.close();
                    return Ok(());
                }
                window.send(&HostMessage::Layout(Some(panels)))?;
            }
            request @ (ClientMessage::ListWorkspaces { .. }
            | ClientMessage::CreateWorkspace(_)
            | ClientMessage::OpenWorkspace(_)
            | ClientMessage::CloseWorkspace { .. }) => {
                let Ok(answer) = workspaces(store, device, request) else {
                    window.close();
                    return Ok(());
                };
                window.send(&HostMessage::Workspaces(answer))?;
            }
            // The host has no events yet, so a resync is its status alone.
            ClientMessage::Resync { .. } => window.send(&HostMessage::Status(status(device)?))?,
            ClientMessage::Intent(intent) => {
                window.send(&HostMessage::Acknowledgment(Acknowledgment {
                    request_id: intent.request_id().clone(),
                    outcome: Outcome::Unsupported,
                    recorded_at: now(),
                    detail: Note::new("This build of the Nexees host carries out no intents yet.")
                        .ok(),
                }))?
            }
        }
    }
}

/// The host's status now: this device, with no sessions or tasks in this build.
fn status(device: &DeviceId) -> Result<HostStatus, IpcError> {
    HostStatus::new(device.clone(), now(), Vec::new(), Vec::new())
        .map_err(|error| IpcError::Malformed(error.to_string()))
}

/// Makes sure a host runs and answers, starting one only when none does.
fn start() -> u8 {
    let Ok(folder) = ChannelFolder::from_environment() else {
        return exit::UNSUPPORTED;
    };
    if answers(&folder) {
        return exit::OK;
    }
    let Ok(program) = env::current_exe() else {
        return exit::FAILED;
    };
    // Its own process group, so the host outlives the window that asked for it (RC-04).
    let spawned = Command::new(program)
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();
    let Ok(mut child) = spawned else {
        return exit::FAILED;
    };
    // A bounded wait with growing pauses: the new host, or one another window started at the
    // same moment, answers within it or `start` gives up.
    let deadline = Instant::now() + START_TIMEOUT;
    let mut pause = Duration::from_millis(20);
    while Instant::now() < deadline {
        if answers(&folder) {
            return exit::OK;
        }
        // Reap the child when it has already stopped, such as a second host that found the
        // lock held; the host that holds it is the one to wait for.
        let _ = child.try_wait();
        thread::sleep(pause);
        pause = (pause * 2).min(Duration::from_millis(500));
    }
    exit::NO_ANSWER
}

/// Whether a host in `folder` answers a hello with a compatible one.
fn answers(folder: &ChannelFolder) -> bool {
    let attempt = || -> Result<bool, IpcError> {
        let mut host = Connection::connect(folder)?;
        host.set_timeout(Some(HELLO_TIMEOUT))?;
        let ours = Hello::current();
        host.send(&ClientMessage::Hello(ours))?;
        let compatible = match host.receive()? {
            HostMessage::Hello(theirs) => negotiate(&ours, &theirs, MIN_SECURE_VERSION).is_ok(),
            _ => false,
        };
        host.close();
        Ok(compatible)
    };
    attempt().unwrap_or(false)
}

/// The start at login: enable, disable or report it.
fn login_start(action: &str) -> u8 {
    let Ok(login) = LoginStart::from_environment() else {
        return exit::UNSUPPORTED;
    };
    match action {
        "enable" => {
            let Ok(program) = env::current_exe().and_then(|p| p.canonicalize()) else {
                return exit::FAILED;
            };
            if login.enable(&program).is_ok() {
                exit::OK
            } else {
                exit::FAILED
            }
        }
        "disable" => match login.disable() {
            Ok(_) => exit::OK,
            Err(_) => exit::FAILED,
        },
        "status" => match login.status() {
            Ok(LoginStartStatus::Enabled) => exit::OK,
            Ok(LoginStartStatus::Disabled) => exit::NO,
            _ => exit::FAILED,
        },
        _ => exit::USAGE,
    }
}

/// The current time.
fn now() -> Timestamp {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0);
    Timestamp::from_unix_millis(millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::{AtomicU64, Ordering};

    use nexees_domain::client::{ClientState, PanelLayoutFields, PanelState};
    use nexees_domain::device::ProtocolVersions;
    use nexees_domain::ids::{AgentSessionId, ModelId, OperationId, ProviderId, RequestId, ViewId};
    use nexees_domain::lcl::SpecificationMode;
    use nexees_domain::model_capabilities::ProviderModel;
    use nexees_domain::remote_request::ExpectedRevision;
    use nexees_domain::revision::{Generation, RevisionId, WorkspaceRevision};
    use nexees_domain::session::{AgentSession, AgentSessionFields, AutonomyMode};
    use nexees_domain::text::Label;
    use nexees_domain::workspace::{ContentAvailability, DeviceRoot, WorkspaceKind};
    use nexees_protocol::messages::{Intent, IntentFields};
    use nexees_protocol::operations::{AgentPause, LocalIntent};
    use nexees_workspaces::workspace_registry::MAX_ROOT_BYTES;

    /// A private runtime folder and data folder for one test, removed when it ends.
    struct Profile(PathBuf);

    impl Profile {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let unique = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "nexees-host-{}-{unique}-{name}",
                std::process::id()
            ));
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path.join("run"))
                .unwrap();
            Self(path)
        }

        fn folder(&self) -> ChannelFolder {
            ChannelFolder::open(&self.0.join("run")).unwrap()
        }

        fn data(&self) -> PathBuf {
            self.0.join("data")
        }

        /// Starts a host on its own thread; returns its exit code through the handle.
        fn host(&self, keep_running: bool, grace: Duration) -> thread::JoinHandle<u8> {
            let (folder, data) = (self.folder(), self.data());
            let host = thread::spawn(move || serve_in(&folder, &data, keep_running, grace));
            let deadline = Instant::now() + Duration::from_secs(5);
            while !answers(&self.folder()) {
                assert!(Instant::now() < deadline, "the host never answered");
                thread::sleep(Duration::from_millis(10));
            }
            host
        }
    }

    impl Drop for Profile {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Waits for the host thread to end, for at most ten seconds: a host that does not stop fails
    /// the test instead of hanging it.
    fn finish(host: thread::JoinHandle<u8>) -> u8 {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !host.is_finished() {
            assert!(Instant::now() < deadline, "the host did not stop");
            thread::sleep(Duration::from_millis(20));
        }
        host.join().unwrap()
    }

    /// A window: connected, hello exchanged.
    fn window(profile: &Profile) -> Connection {
        let mut window = Connection::connect(&profile.folder()).unwrap();
        window.set_timeout(Some(Duration::from_secs(5))).unwrap();
        window
            .send(&ClientMessage::Hello(Hello::current()))
            .unwrap();
        assert!(matches!(window.receive().unwrap(), HostMessage::Hello(_)));
        window
    }

    fn device_of(window: &mut Connection) -> DeviceId {
        window.set_timeout(Some(Duration::from_secs(5))).unwrap();
        window.send(&ClientMessage::Resync { after: None }).unwrap();
        match window.receive().unwrap() {
            HostMessage::Status(status) => {
                assert!(status.sessions().is_empty() && status.tasks().is_empty());
                status.host_device_id().clone()
            }
            other => panic!("expected the host's status, got {other:?}"),
        }
    }

    #[test]
    fn a_window_says_hello_and_resyncs_to_the_hosts_status() {
        let profile = Profile::new("hello");
        let host = profile.host(false, Duration::from_millis(300));
        let mut first = window(&profile);
        let device = device_of(&mut first);
        assert!(device.as_str().starts_with("desktop-"));
        drop(first);
        assert_eq!(finish(host), exit::OK);
    }

    #[test]
    fn the_device_identity_is_created_once_and_kept_across_restarts() {
        let profile = Profile::new("identity");
        let host = profile.host(false, Duration::from_millis(200));
        let device = device_of(&mut window(&profile));
        assert_eq!(finish(host), exit::OK);
        let host = profile.host(false, Duration::from_millis(200));
        assert_eq!(device_of(&mut window(&profile)), device);
        assert_eq!(finish(host), exit::OK);
    }

    #[test]
    fn a_second_host_does_not_start_while_one_runs() {
        let profile = Profile::new("single");
        let _first = profile.host(true, Duration::from_millis(100));
        // On its own thread, so a second host that wrongly starts fails the test, not hangs it.
        let (folder, data) = (profile.folder(), profile.data());
        let second =
            thread::spawn(move || serve_in(&folder, &data, true, Duration::from_millis(100)));
        assert_eq!(finish(second), exit::ALREADY_RUNNING);
        // The running host still answers: the second one did not touch its socket.
        assert!(answers(&profile.folder()));
    }

    #[test]
    fn the_host_stops_after_its_last_window_detaches_unless_it_keeps_running() {
        let grace = Duration::from_millis(200);
        let profile = Profile::new("lifetime");
        let host = profile.host(false, grace);
        let (first, second) = (window(&profile), window(&profile));
        drop(first);
        // One window is still attached: the host stays past the grace period.
        thread::sleep(grace * 3);
        assert!(!host.is_finished());
        drop(second);
        assert_eq!(finish(host), exit::OK);
        assert!(!profile.folder().socket().exists(), "the socket is removed");

        let kept = Profile::new("kept");
        let host = kept.host(true, grace);
        drop(window(&kept));
        thread::sleep(grace * 3);
        assert!(!host.is_finished());
        assert!(
            answers(&kept.folder()),
            "kept running after its window closed"
        );
    }

    #[test]
    fn a_reloading_window_reattaches_to_the_same_host() {
        let profile = Profile::new("reattach");
        let host = profile.host(false, Duration::from_millis(500));
        let device = device_of(&mut window(&profile));
        for _ in 0..3 {
            // Each reopened window finds the one running host, with the same identity.
            assert_eq!(device_of(&mut window(&profile)), device);
        }
        assert_eq!(finish(host), exit::OK);
    }

    #[test]
    fn a_window_of_another_protocol_version_or_without_a_hello_is_refused() {
        let profile = Profile::new("refused");
        let _host = profile.host(true, Duration::from_millis(100));
        let mut newer = Connection::connect(&profile.folder()).unwrap();
        newer.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let future = Hello::new(ProtocolVersions { min: 9, max: 9 }).unwrap();
        newer.send(&ClientMessage::Hello(future)).unwrap();
        assert!(matches!(newer.receive().unwrap(), HostMessage::Hello(_)));
        assert_eq!(
            newer.receive::<HostMessage>().unwrap_err(),
            IpcError::Closed
        );

        let mut rude = Connection::connect(&profile.folder()).unwrap();
        rude.set_timeout(Some(Duration::from_secs(5))).unwrap();
        rude.send(&ClientMessage::Resync { after: None }).unwrap();
        assert_eq!(rude.receive::<HostMessage>().unwrap_err(), IpcError::Closed);
    }

    #[test]
    fn more_windows_than_the_bound_are_refused() {
        let profile = Profile::new("bound");
        let _host = profile.host(true, Duration::from_millis(100));
        let attached: Vec<_> = (0..MAX_WINDOWS).map(|_| window(&profile)).collect();
        let mut extra = Connection::connect(&profile.folder()).unwrap();
        extra.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let _ = extra.send(&ClientMessage::Hello(Hello::current()));
        assert!(
            extra.receive::<HostMessage>().is_err(),
            "the extra window is closed"
        );
        drop(attached);
    }

    #[test]
    fn an_intent_is_answered_as_unsupported_rather_than_left_open() {
        let profile = Profile::new("intent");
        let _host = profile.host(true, Duration::from_millis(100));
        let mut window = window(&profile);
        let fields = IntentFields {
            request_id: RequestId::new("q1").unwrap(),
            operation: OperationId::new("agent_pause").unwrap(),
            arguments: AgentPause {},
            workspace_id: Some(WorkspaceId::new("w").unwrap()),
            agent_id: Some(AgentSessionId::new("s1").unwrap()),
            expected_revisions: vec![ExpectedRevision::Generation(Generation::FIRST)],
            issued_at: Timestamp::from_unix_millis(1_000),
            expires_at: Timestamp::from_unix_millis(2_000),
        };
        let intent = LocalIntent::AgentPause(Intent::try_from(fields).unwrap());
        window.send(&ClientMessage::Intent(intent)).unwrap();
        match window.receive().unwrap() {
            HostMessage::Acknowledgment(ack) => {
                assert_eq!(ack.outcome, Outcome::Unsupported);
                assert_eq!(ack.request_id.as_str(), "q1");
                assert!(ack.detail.is_some(), "the window can say why");
            }
            other => panic!("expected an acknowledgment, got {other:?}"),
        }
    }

    /// A panel layout whose right sidebar is as given; the other panels are fixed.
    fn panels(right_shown: bool, right_size: u32, right_view: &str) -> PanelLayout {
        let panel = |shown, size, view: &str| PanelState {
            shown,
            size: Some(size),
            selected: Some(ViewId::new(view).unwrap()),
        };
        PanelLayout::try_from(PanelLayoutFields {
            left: panel(true, 203, "explorer-view-container"),
            right: panel(right_shown, right_size, right_view),
            bottom: panel(true, 167, "terminal-0"),
        })
        .unwrap()
    }

    /// A window that speaks in raw frames, to send what no well-formed message can hold: four
    /// bytes of big-endian length, then the JSON. The hellos are exchanged.
    fn raw_window(profile: &Profile) -> UnixStream {
        let mut window = UnixStream::connect(profile.folder().socket()).unwrap();
        window
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        send_frame(&mut window, r#"{"hello":{"versions":{"min":1,"max":3}}}"#);
        let mut length = [0_u8; 4];
        window.read_exact(&mut length).unwrap();
        let mut hello = vec![0_u8; u32::from_be_bytes(length) as usize];
        window.read_exact(&mut hello).unwrap();
        assert!(hello.starts_with(br#"{"hello":"#));
        window
    }

    fn send_frame(window: &mut UnixStream, body: &str) {
        let length = u32::try_from(body.len()).unwrap().to_be_bytes();
        window.write_all(&length).unwrap();
        window.write_all(body.as_bytes()).unwrap();
    }

    /// Sends `message` and returns the layout the host answers with.
    fn layout_after(window: &mut Connection, message: &ClientMessage) -> Option<PanelLayout> {
        window.send(message).unwrap();
        match window.receive().unwrap() {
            HostMessage::Layout(kept) => kept,
            other => panic!("expected the kept layout, got {other:?}"),
        }
    }

    #[test]
    fn a_window_finds_the_panels_it_stored_after_the_host_restarted() {
        let profile = Profile::new("layout");
        let host = profile.host(false, Duration::from_millis(300));
        let mut first = window(&profile);
        assert_eq!(
            layout_after(&mut first, &ClientMessage::LoadLayout {}),
            None
        );
        let shown = panels(true, 354, "nexees-area-agent");
        assert_eq!(
            layout_after(&mut first, &ClientMessage::StoreLayout(shown.clone())),
            Some(shown)
        );
        // The next layout replaces the first: the right sidebar hidden, with the area and the
        // width it comes back with.
        let hidden = panels(false, 420, "nexees-area-tasks");
        assert_eq!(
            layout_after(&mut first, &ClientMessage::StoreLayout(hidden.clone())),
            Some(hidden.clone())
        );
        // Another window of the same user is the same client.
        assert_eq!(
            layout_after(&mut window(&profile), &ClientMessage::LoadLayout {}),
            Some(hidden.clone())
        );
        drop(first);
        assert_eq!(finish(host), exit::OK);
        // A new host on the same store, as after the window and the host were both closed.
        let host = profile.host(false, Duration::from_millis(300));
        let mut reopened = window(&profile);
        assert_eq!(
            layout_after(&mut reopened, &ClientMessage::LoadLayout {}),
            Some(hidden)
        );
        drop(reopened);
        assert_eq!(finish(host), exit::OK);
    }

    #[test]
    fn a_layout_that_is_none_closes_the_channel_and_leaves_the_kept_one() {
        let profile = Profile::new("bad-layout");
        let _host = profile.host(true, Duration::from_millis(100));
        let kept = panels(true, 354, "nexees-area-agent");
        let mut good = window(&profile);
        layout_after(&mut good, &ClientMessage::StoreLayout(kept.clone()));
        let hidden = r#"{"shown":false,"size":null,"selected":null}"#;
        let storing = |left: &str| {
            format!(r#"{{"store_layout":{{"left":{left},"right":{hidden},"bottom":{hidden}}}}}"#)
        };
        for bad in [
            // A size that is none, a view name that is no identifier, another client's name, a
            // field that rides along.
            storing(r#"{"shown":true,"size":0,"selected":null}"#),
            storing(r#"{"shown":false,"size":null,"selected":"../x"}"#),
            r#"{"load_layout":{"client_id":"another-window"}}"#.to_owned(),
            storing(r#"{"shown":false,"size":null,"selected":null,"run":"x"}"#),
        ] {
            let mut rude = raw_window(&profile);
            send_frame(&mut rude, &bad);
            // The host answers nothing and closes: the read ends without a byte.
            let mut answer = Vec::new();
            assert_eq!(rude.read_to_end(&mut answer).unwrap(), 0, "{bad}");
        }
        assert_eq!(
            layout_after(&mut good, &ClientMessage::LoadLayout {}),
            Some(kept)
        );
    }

    #[test]
    fn a_window_of_the_first_protocol_is_served_but_refused_the_layout_messages() {
        let profile = Profile::new("layout-v1");
        let _host = profile.host(true, Duration::from_millis(100));
        let mut old = Connection::connect(&profile.folder()).unwrap();
        old.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let first = Hello::new(ProtocolVersions { min: 1, max: 1 }).unwrap();
        old.send(&ClientMessage::Hello(first)).unwrap();
        assert!(matches!(old.receive().unwrap(), HostMessage::Hello(_)));
        // What the first protocol has still works.
        assert!(device_of(&mut old).as_str().starts_with("desktop-"));
        // What it does not have is refused: the two agreed on version 1.
        old.send(&ClientMessage::LoadLayout {}).unwrap();
        assert_eq!(old.receive::<HostMessage>().unwrap_err(), IpcError::Closed);
    }

    /// Sends a workspace request and returns the host's answer.
    fn ask(window: &mut Connection, message: ClientMessage) -> WorkspaceAnswer {
        window.send(&message).unwrap();
        match window.receive().unwrap() {
            HostMessage::Workspaces(answer) => answer,
            other => panic!("expected a workspace answer, got {other:?}"),
        }
    }

    fn new_workspace(kind: WorkspaceKind, name: &str, root: &Path) -> ClientMessage {
        ClientMessage::CreateWorkspace(NewWorkspace {
            kind,
            name: Label::new(name).unwrap(),
            root: DeviceRoot::new(root.to_str().unwrap()).unwrap(),
        })
    }

    /// The workspace a carried-out request concerned, and the foreground it left.
    fn done(answer: WorkspaceAnswer) -> (Option<WorkspaceEntry>, Option<WorkspaceId>) {
        match answer {
            WorkspaceAnswer::Done {
                workspace,
                foreground,
            } => (workspace, foreground),
            other => panic!("expected the request carried out, got {other:?}"),
        }
    }

    fn refusal(reason: WorkspaceRefusal) -> WorkspaceAnswer {
        WorkspaceAnswer::Refused {
            reason,
            workspace_id: None,
        }
    }

    /// Every workspace the host lists, through all its pages; the foreground; how many pages.
    fn listed(window: &mut Connection) -> (Vec<WorkspaceEntry>, Option<WorkspaceId>, usize) {
        let (mut all, mut after, mut pages) = (Vec::new(), None, 0);
        loop {
            let WorkspaceAnswer::Page {
                entries,
                more,
                foreground,
            } = ask(window, ClientMessage::ListWorkspaces { after })
            else {
                panic!("expected a page");
            };
            pages += 1;
            after = entries.last().map(|entry| entry.workspace_id.clone());
            all.extend(entries);
            if !more {
                return (all, foreground, pages);
            }
        }
    }

    fn by_id(mut entries: Vec<WorkspaceEntry>) -> Vec<WorkspaceEntry> {
        entries.sort_by(|a, b| a.workspace_id.as_str().cmp(b.workspace_id.as_str()));
        entries
    }

    #[test]
    fn a_window_creates_opens_lists_and_closes_workspaces_that_outlive_the_host() {
        let profile = Profile::new("workspaces");
        let roots = profile.0.join("roots");
        for dir in ["arch-dock", "lcl-next", "x"] {
            fs::create_dir_all(roots.join(dir)).unwrap();
        }
        let host = profile.host(false, Duration::from_millis(300));
        let mut first = window(&profile);
        assert_eq!(listed(&mut first), (Vec::new(), None, 1));
        let code = new_workspace(WorkspaceKind::Code, "Arch Dock", &roots.join("arch-dock"));
        let (code, foreground) = done(ask(&mut first, code));
        let code = code.unwrap();
        // A new workspace has a stable ID of sixteen random bytes, and creating opens nothing.
        assert!(code.workspace_id.as_str().starts_with("ws-"));
        assert_eq!(code.workspace_id.as_str().len(), 3 + 32);
        assert_eq!(
            (code.kind, code.availability),
            (WorkspaceKind::Code, ContentAvailability::Local)
        );
        assert_eq!(foreground, None);
        let lcl = new_workspace(WorkspaceKind::Lcl, "LCL — Next", &roots.join("lcl-next"));
        let lcl = done(ask(&mut first, lcl)).0.unwrap();
        // Opened by its root, however the path spells it, then the other by its ID.
        let spelled = DeviceRoot::new(format!("{}/x/../arch-dock", roots.display())).unwrap();
        let opened = done(ask(
            &mut first,
            ClientMessage::OpenWorkspace(WorkspaceTarget::Root(spelled)),
        ));
        assert_eq!(
            opened,
            (Some(code.clone()), Some(code.workspace_id.clone()))
        );
        let by_lcl_id =
            ClientMessage::OpenWorkspace(WorkspaceTarget::WorkspaceId(lcl.workspace_id.clone()));
        assert_eq!(
            done(ask(&mut first, by_lcl_id)).1,
            Some(lcl.workspace_id.clone())
        );
        let both = by_id(vec![code.clone(), lcl.clone()]);
        assert_eq!(
            listed(&mut first),
            (both.clone(), Some(lcl.workspace_id.clone()), 1)
        );
        // Another window of the same user is the same client, with the same foreground.
        assert_eq!(
            listed(&mut window(&profile)).1,
            Some(lcl.workspace_id.clone())
        );
        // Only the workspace the client shows can be closed; then it shows none.
        let close = |id: &WorkspaceId| ClientMessage::CloseWorkspace {
            workspace_id: id.clone(),
        };
        assert_eq!(
            ask(&mut first, close(&code.workspace_id)),
            refusal(WorkspaceRefusal::NotOpen)
        );
        assert_eq!(
            done(ask(&mut first, close(&lcl.workspace_id))),
            (None, None)
        );
        drop(first);
        assert_eq!(finish(host), exit::OK);
        // A new host on the same store lists the same workspaces under the same IDs.
        let host = profile.host(false, Duration::from_millis(300));
        let mut again = window(&profile);
        assert_eq!(listed(&mut again), (both, None, 1));
        drop(again);
        assert_eq!(finish(host), exit::OK);
    }

    #[test]
    fn a_workspace_request_that_cannot_be_carried_out_is_refused_and_changes_nothing() {
        let profile = Profile::new("workspace-refusals");
        let roots = profile.0.join("roots");
        fs::create_dir_all(roots.join("a/src")).unwrap();
        fs::create_dir_all(roots.join("c")).unwrap();
        fs::write(roots.join("file.txt"), "not a folder").unwrap();
        let _host = profile.host(true, Duration::from_millis(100));
        let mut window = window(&profile);
        let a = done(ask(
            &mut window,
            new_workspace(WorkspaceKind::Code, "A", &roots.join("a")),
        ));
        let a = a.0.unwrap();
        assert_eq!(
            ask(
                &mut window,
                new_workspace(WorkspaceKind::Lcl, "Inside", &roots.join("a/src"))
            ),
            WorkspaceAnswer::Refused {
                reason: WorkspaceRefusal::Overlaps,
                workspace_id: Some(a.workspace_id.clone())
            }
        );
        for (root, reason) in [
            (PathBuf::from("relative"), WorkspaceRefusal::RootNotAbsolute),
            (roots.join("missing"), WorkspaceRefusal::RootMissing),
            (roots.join("file.txt"), WorkspaceRefusal::RootNotAFolder),
        ] {
            let request = new_workspace(WorkspaceKind::Code, "W", &root);
            assert_eq!(
                ask(&mut window, request),
                refusal(reason),
                "{}",
                root.display()
            );
        }
        let open = |target| ClientMessage::OpenWorkspace(target);
        let unknown = WorkspaceTarget::WorkspaceId(WorkspaceId::new("ws-unknown").unwrap());
        assert_eq!(
            ask(&mut window, open(unknown)),
            refusal(WorkspaceRefusal::NotFound)
        );
        // A folder that is no workspace's root is not one, and neither is a missing folder.
        let other =
            WorkspaceTarget::Root(DeviceRoot::new(roots.join("c").to_str().unwrap()).unwrap());
        assert_eq!(
            ask(&mut window, open(other)),
            refusal(WorkspaceRefusal::NotFound)
        );
        let gone =
            WorkspaceTarget::Root(DeviceRoot::new(roots.join("gone").to_str().unwrap()).unwrap());
        assert_eq!(
            ask(&mut window, open(gone)),
            refusal(WorkspaceRefusal::RootMissing)
        );
        assert_eq!(listed(&mut window), (vec![a], None, 1));
        // A message that names a client or an agent, or carries anything else, closes the channel.
        for bad in [
            r#"{"list_workspaces":{"after":null,"client_id":"another-window"}}"#,
            r#"{"open_workspace":{"workspace_id":"ws-1","agent_session":"claude-1"}}"#,
            r#"{"create_workspace":{"kind":"code","name":"W","root":"/w","grants":["all"]}}"#,
        ] {
            let mut rude = raw_window(&profile);
            send_frame(&mut rude, bad);
            let mut answer = Vec::new();
            assert_eq!(rude.read_to_end(&mut answer).unwrap(), 0, "{bad}");
        }
    }

    #[test]
    fn a_window_of_protocol_version_2_is_served_but_refused_the_workspace_messages() {
        let profile = Profile::new("workspaces-v2");
        let _host = profile.host(true, Duration::from_millis(100));
        let mut older = Connection::connect(&profile.folder()).unwrap();
        older.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let second = Hello::new(ProtocolVersions { min: 1, max: 2 }).unwrap();
        older.send(&ClientMessage::Hello(second)).unwrap();
        assert!(matches!(older.receive().unwrap(), HostMessage::Hello(_)));
        // TASK-012's window keeps its layouts, and is refused what version 2 does not have.
        assert_eq!(
            layout_after(&mut older, &ClientMessage::LoadLayout {}),
            None
        );
        older
            .send(&ClientMessage::ListWorkspaces { after: None })
            .unwrap();
        assert_eq!(
            older.receive::<HostMessage>().unwrap_err(),
            IpcError::Closed
        );
    }

    fn session_bound_to(workspace: &WorkspaceId, device: &DeviceId) -> AgentSession {
        AgentSession::try_from(AgentSessionFields {
            session_id: AgentSessionId::new("claude-1").unwrap(),
            workspace_id: workspace.clone(),
            execution_device_id: device.clone(),
            checkout: WorkspaceRevision {
                workspace_id: workspace.clone(),
                revision: RevisionId::new("r7").unwrap(),
            },
            specification_mode: SpecificationMode::Standard,
            specification: None,
            model: ProviderModel {
                provider_id: ProviderId::new("anthropic").unwrap(),
                model_id: ModelId::new("claude-opus-5-5").unwrap(),
            },
            generation: Generation::FIRST,
            autonomy: AutonomyMode::Balanced,
            orientation_override: None,
        })
        .unwrap()
    }

    #[test]
    fn a_windows_foreground_switches_leave_a_bound_agent_session_unchanged() {
        // Claude runs on Arch Dock; the user switches the window to the LCL project and back.
        let profile = Profile::new("workspace-agent");
        let roots = profile.0.join("roots");
        for dir in ["arch-dock", "lcl-next"] {
            fs::create_dir_all(roots.join(dir)).unwrap();
        }
        let host = profile.host(false, Duration::from_millis(200));
        let mut first = window(&profile);
        let code = new_workspace(WorkspaceKind::Code, "Arch Dock", &roots.join("arch-dock"));
        let code = done(ask(&mut first, code)).0.unwrap().workspace_id;
        let lcl = new_workspace(WorkspaceKind::Lcl, "LCL — Next", &roots.join("lcl-next"));
        let lcl = done(ask(&mut first, lcl)).0.unwrap().workspace_id;
        drop(first);
        assert_eq!(finish(host), exit::OK);
        // The session is in the store before the host serves the switches.
        let db = profile.data().join("state.db");
        let (mut store, _) = StateStore::open(&db, now()).unwrap();
        let session = session_bound_to(&code, &store.device_id().unwrap().unwrap());
        store.write(now(), |w| w.insert(&session)).unwrap();
        store.close(now()).unwrap();

        let host = profile.host(false, Duration::from_millis(200));
        let mut window = window(&profile);
        for target in [&code, &lcl, &code, &lcl] {
            let open = ClientMessage::OpenWorkspace(WorkspaceTarget::WorkspaceId(target.clone()));
            assert_eq!(done(ask(&mut window, open)).1.as_ref(), Some(target));
        }
        drop(window);
        assert_eq!(finish(host), exit::OK);
        let (store, _) = StateStore::open(&db, now()).unwrap();
        assert_eq!(store.all::<AgentSession>().unwrap(), vec![session]);
        let state = store.get::<ClientState>(WINDOW_CLIENT).unwrap().unwrap();
        assert_eq!(state.foreground_workspace, Some(lcl));
        store.close(now()).unwrap();
    }

    #[test]
    fn the_largest_workspace_fits_one_page_and_a_long_list_comes_in_pages() {
        // The longest ID, a name and a root at the registry's limits made of characters that JSON
        // escapes: one such workspace still fits a message of the window's channel.
        let largest = WorkspaceEntry {
            workspace_id: WorkspaceId::new("w".repeat(128)).unwrap(),
            kind: WorkspaceKind::Code,
            name: Label::new("\"".repeat(256)).unwrap(),
            root: Some(DeviceRoot::new(format!("/{}", "\\".repeat(MAX_ROOT_BYTES - 1))).unwrap()),
            availability: ContentAvailability::SyncedCopy,
        };
        let page = HostMessage::Workspaces(WorkspaceAnswer::Page {
            foreground: Some(largest.workspace_id.clone()),
            entries: vec![largest],
            more: true,
        });
        assert!(encode(&page, MAX_IPC_MESSAGE_BYTES).is_ok());

        let profile = Profile::new("workspace-pages");
        let _host = profile.host(true, Duration::from_millis(100));
        let mut window = window(&profile);
        let mut made = Vec::new();
        for n in 0..14 {
            let root = profile.0.join(format!("roots/{n:02}-{}", "p".repeat(180)));
            fs::create_dir_all(&root).unwrap();
            let request = new_workspace(WorkspaceKind::Lcl, &format!("Project {n}"), &root);
            made.push(done(ask(&mut window, request)).0.unwrap());
        }
        let (all, foreground, pages) = listed(&mut window);
        assert!(pages > 1, "{pages} page(s)");
        assert_eq!((all, foreground), (by_id(made), None));
    }

    #[test]
    fn unknown_command_lines_are_refused() {
        for args in [
            &[][..],
            &["serve", "--now"][..],
            &["login-start"][..],
            &["stop"][..],
        ] {
            assert_eq!(run(args), exit::USAGE, "{args:?}");
        }
    }
}
