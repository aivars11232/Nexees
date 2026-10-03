//! The Desktop side of the TASK-003 prototype, for Linux. Evidence only.
//!
//! ```text
//! nexees-host identity <state>
//! nexees-host receive <state> <listen> <peer-cert> <core-0.1.0> <core-0.3.0> [--allow-uid <uid>]
//! nexees-host ipc <state> <json>
//! nexees-host send <state> <address> <peer-cert> <from> <to> <direction> <action-file> <expires-in-seconds>
//! nexees-host startup <enable|disable|status> <config-home> [<absolute receiver command>...]
//! ```
//!
//! `receive` runs the Desktop receiver: requests from the paired phone arrive
//! over TLS and go through the shared dispatcher, and the local UI talks to it
//! over a Unix socket that only the receiver's own user may use. Only the
//! caller decides the listen address; the tests use 127.0.0.1, which keeps
//! the receiver off the LAN and needs no firewall change.

#[cfg(target_os = "linux")]
fn main() {
    desktop::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("nexees-host runs on Linux only");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
mod desktop {
    use nexees_feasibility::lcl_bridge::Lcl;
    use nexees_feasibility::remote::{self, Action, Capability, Device, Direction, Envelope, Identity, Platform, Status};
    use nexees_feasibility::store::Store;
    use serde_json::{json, Value};
    use std::fs;
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    /// Applications a request may launch, by allowlist key. Only the owner
    /// changes this list; no request can extend it.
    pub(crate) const ALLOWLIST: [(&str, &str); 1] = [("hoptodesk", "com.hoptodesk.HopToDesk.desktop")];
    const MAX_IPC_MESSAGE: u64 = 4096;
    const THIS_DEVICE: &str = "pc-1";
    const PAIRED_PHONE: &str = "phone-1";

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let path = Path::new;
        let result = match args.as_slice() {
            ["identity", state] => identity(path(state)),
            ["receive", state, listen, peer, core1, core3] => receive(path(state), listen, path(peer), path(core1), path(core3), None),
            ["receive", state, listen, peer, core1, core3, "--allow-uid", uid] => match uid.parse() {
                Ok(uid) => receive(path(state), listen, path(peer), path(core1), path(core3), Some(uid)),
                Err(e) => Err(e.to_string()),
            },
            ["ipc", state, message] => ipc(path(state), message),
            ["send", state, address, peer, from, to, direction, action, expires_in] => {
                send(path(state), address, path(peer), from, to, direction, path(action), expires_in)
            }
            ["startup", verb, config_home, exec @ ..] => startup(verb, path(config_home), exec),
            _ => Err("usage: see the comment at the top of src/bin/host.rs".to_string()),
        };
        match result {
            Ok(output) => println!("{output}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }

    fn text(e: impl std::fmt::Display) -> String {
        e.to_string()
    }

    /// Creates `dir` readable by its owner only.
    fn private_dir(dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(text)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(text)
    }

    fn identity(state: &Path) -> Result<String, String> {
        private_dir(state)?;
        let identity = Identity::load_or_create(&state.join("identity")).map_err(text)?;
        Ok(json!({"fingerprint": identity.fingerprint(), "certificate": state.join("identity/identity.cert.der")}).to_string())
    }

    fn receive(state: &Path, listen: &str, peer: &Path, core1: &Path, core3: &Path, allowed_uid: Option<u32>) -> Result<String, String> {
        private_dir(state)?;
        let identity = Identity::load_or_create(&state.join("identity")).map_err(text)?;
        let config = remote::server_config(&identity, &remote::read_certificate(peer).map_err(text)?)?;
        let lcl = Lcl::open(core1, core3)?;
        let database = state.join("state.sqlite");
        let mut store = Store::open(&database).map_err(text)?;
        let listener = TcpListener::bind(listen).map_err(text)?;
        let stop = Arc::new(AtomicBool::new(false));
        let allowed_uid = allowed_uid.unwrap_or_else(|| nix::unistd::getuid().as_raw());
        let socket = state.join("ipc.sock");
        let ipc = {
            let (socket, database, stop) = (socket.clone(), database.clone(), stop.clone());
            std::thread::spawn(move || serve_ipc(&socket, &database, allowed_uid, &stop))
        };
        let device = Device {
            id: THIS_DEVICE.into(),
            peer: PAIRED_PHONE.into(),
            inbound: Direction::PhoneToPc,
            workspaces: state.join("workspaces"),
            lcl: &lcl,
        };
        eprintln!("receiving on {listen}; local control at {}", socket.display());
        let served = remote::serve(
            listener,
            config,
            &stop,
            |envelope| remote::dispatch(&device, &mut store, &mut DesktopLauncher, envelope),
            |line| eprintln!("{line}"),
        );
        stop.store(true, Ordering::SeqCst);
        let ipc = ipc.join().map_err(|_| "the IPC thread panicked".to_string())?;
        served.map_err(text)?;
        ipc?;
        Ok("stopped".to_string())
    }

    fn serve_ipc(socket: &Path, database: &Path, allowed_uid: u32, stop: &AtomicBool) -> Result<(), String> {
        // A socket left by an earlier run in this private folder.
        let _ = fs::remove_file(socket);
        let listener = UnixListener::bind(socket).map_err(text)?;
        fs::set_permissions(socket, fs::Permissions::from_mode(0o600)).map_err(text)?;
        listener.set_nonblocking(true).map_err(text)?;
        while !stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(e) = answer_ipc(stream, database, allowed_uid, stop) {
                        eprintln!("ipc: {e}");
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(100)),
                Err(e) => return Err(e.to_string()),
            }
        }
        let _ = fs::remove_file(socket);
        Ok(())
    }

    /// The kernel reports the connecting process's user; the socket's file
    /// mode is only a first barrier. Messages are one bounded line of JSON.
    fn answer_ipc(mut stream: UnixStream, database: &Path, allowed_uid: u32, stop: &AtomicBool) -> io::Result<()> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let peer = nix::sys::socket::getsockopt(&stream, nix::sys::socket::sockopt::PeerCredentials)?;
        let reply = if peer.uid() != allowed_uid {
            json!({"status": "denied", "detail": format!("peer uid {} (pid {}) is not uid {allowed_uid}", peer.uid(), peer.pid())})
        } else {
            let mut line = Vec::new();
            BufReader::new(&mut stream).take(MAX_IPC_MESSAGE + 1).read_until(b'\n', &mut line)?;
            if line.len() as u64 > MAX_IPC_MESSAGE {
                json!({"status": "invalid", "detail": format!("a message is at most {MAX_IPC_MESSAGE} bytes")})
            } else {
                handle_ipc(&line, database, stop)
            }
        };
        writeln!(stream, "{reply}")
    }

    fn handle_ipc(line: &[u8], database: &Path, stop: &AtomicBool) -> Value {
        let Ok(message) = serde_json::from_slice::<Value>(line) else {
            return json!({"status": "invalid", "detail": "not JSON"});
        };
        let store = Store::open(database);
        match (message["op"].as_str(), store) {
            (_, Err(e)) => json!({"status": "invalid", "detail": e.to_string()}),
            // The PC holds only the grant for requests the PC executes.
            (Some("grant"), Ok(mut store)) => match (message["direction"].as_str(), message["enabled"].as_bool()) {
                (Some("phone_to_pc"), Some(enabled)) => match store.set_grant("phone_to_pc", enabled) {
                    Ok(()) => json!({"status": "completed", "grant": "phone_to_pc", "enabled": enabled}),
                    Err(e) => json!({"status": "invalid", "detail": e.to_string()}),
                },
                _ => json!({"status": "denied", "detail": "this device holds only the phone_to_pc grant"}),
            },
            (Some("status"), Ok(store)) => {
                json!({"status": "completed", "phone_to_pc": store.grant("phone_to_pc").unwrap_or(false), "pid": std::process::id()})
            }
            (Some("stop"), Ok(_)) => {
                stop.store(true, Ordering::SeqCst);
                json!({"status": "completed", "detail": "stopping"})
            }
            _ => json!({"status": "invalid", "detail": "unknown op"}),
        }
    }

    fn ipc(state: &Path, message: &str) -> Result<String, String> {
        let mut stream = UnixStream::connect(state.join("ipc.sock")).map_err(text)?;
        stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(text)?;
        writeln!(stream, "{message}").map_err(text)?;
        let mut reply = String::new();
        BufReader::new(stream).take(MAX_IPC_MESSAGE).read_line(&mut reply).map_err(text)?;
        Ok(reply.trim_end().to_string())
    }

    #[allow(clippy::too_many_arguments)]
    fn send(state: &Path, address: &str, peer: &Path, from: &str, to: &str, direction: &str, action: &Path, expires_in: &str) -> Result<String, String> {
        let identity = Identity::load_or_create(&state.join("identity")).map_err(text)?;
        let config = remote::client_config(&identity, &remote::read_certificate(peer).map_err(text)?)?;
        let action: Action = serde_json::from_slice(&fs::read(action).map_err(text)?).map_err(text)?;
        let direction = match direction {
            "pc_to_phone" => Direction::PcToPhone,
            "phone_to_pc" => Direction::PhoneToPc,
            other => return Err(format!("unknown direction {other}")),
        };
        let expires_in: i64 = expires_in.parse().map_err(text)?;
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let envelope = Envelope {
            protocol: remote::PROTOCOL,
            id: format!("{from}-{}-{nanos:x}", std::process::id()),
            from_device: from.into(),
            to_device: to.into(),
            direction,
            expires_at: (nexees_feasibility::unix_now() as i64 + expires_in).max(0) as u64,
            action,
        };
        let reply = match remote::send(address, config, &envelope) {
            Ok(reply) => serde_json::to_value(reply).map_err(text)?,
            Err(e) => json!({"id": envelope.id, "status": e.status().as_str(), "detail": e.to_string()}),
        };
        Ok(json!({"id": envelope.id, "reply": reply}).to_string())
    }

    /// The start-at-login registration: an XDG autostart entry in the config
    /// home the caller names. Plasma's systemd-xdg-autostart-generator turns
    /// it into a unit that starts after graphical login and ends with the
    /// session. Nothing here enables linger, a system service or automatic
    /// login, and nothing is registered until `enable` is called.
    fn startup(verb: &str, config_home: &Path, exec: &[&str]) -> Result<String, String> {
        let entry = config_home.join("autostart/dev.nexees.Receiver.desktop");
        match verb {
            "status" => Ok(json!({"enabled": entry.is_file(), "entry": entry}).to_string()),
            "disable" => {
                if entry.exists() {
                    fs::remove_file(&entry).map_err(text)?;
                }
                Ok(json!({"enabled": false, "entry": entry}).to_string())
            }
            "enable" => {
                let plain = |a: &str| !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric() || "/._-:".contains(c));
                if exec.is_empty() || !exec[0].starts_with('/') || !exec.iter().all(|a| plain(a)) {
                    return Err("the receiver command must be an absolute path followed by plain arguments".to_string());
                }
                let body = format!(
                    "[Desktop Entry]\nType=Application\nName=Nexees receiver\n\
                     Comment=Receives authorized requests from paired devices during this login session\n\
                     Exec={}\nTerminal=false\nNoDisplay=true\nX-GNOME-Autostart-enabled=true\n",
                    exec.join(" ")
                );
                remote::write_atomically(&entry, body.as_bytes()).map_err(text)?;
                Ok(json!({"enabled": true, "entry": entry}).to_string())
            }
            other => Err(format!("unknown startup verb {other}")),
        }
    }

    /// Launches allowlisted applications in the receiver's own graphical
    /// session through the desktop entry, never through a shell string.
    pub(crate) struct DesktopLauncher;

    impl Platform for DesktopLauncher {
        fn launch(&mut self, _command: &str, app: &str, _expires_at: u64) -> (Status, String) {
            let Some((_, desktop_id)) = ALLOWLIST.iter().find(|(key, _)| *key == app) else {
                return (Status::Denied, format!("{app} is not on the launch allowlist"));
            };
            if let Err(refusal) = session_ready() {
                return refusal;
            }
            let Some(entry) = desktop_entry(desktop_id) else {
                return (Status::Unsupported, format!("{desktop_id} is not installed; Nexees does not install applications"));
            };
            let app_id = desktop_id.trim_end_matches(".desktop");
            let name = entry_name(&entry).unwrap_or_else(|| app_id.to_string());
            let before = observe(app_id, &name);
            match start_entry(&entry) {
                Err(e) => return (Status::OutcomeUnknown, format!("gio could not be started: {e}")),
                Ok(status) if !status.success() => return (Status::Failed, format!("gio launch exited with {status}")),
                Ok(_) => {}
            }
            let mut after = observe(app_id, &name);
            for _ in 0..20 {
                if after.0 && after.1 == Some(true) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
                after = observe(app_id, &name);
            }
            let status = match after {
                (true, Some(true)) => Status::Completed,
                (true, _) => Status::Running,
                (false, _) => Status::OutcomeUnknown,
            };
            (
                status,
                format!(
                    "gio launch {desktop_id} returned 0; instance running before {}, after {}; window before {}, after {}. \
                     A launch does not establish screen control: {name}'s own permissions and session setup are separate.",
                    before.0,
                    after.0,
                    window_text(before.1),
                    window_text(after.1),
                ),
            )
        }

        fn capabilities(&self) -> Vec<Capability> {
            let session = session_ready();
            ALLOWLIST
                .iter()
                .map(|(key, desktop_id)| Capability {
                    action: format!("launch_app:{key}"),
                    availability: match (&session, desktop_entry(desktop_id)) {
                        (Err((Status::NeedsUserAction, _)), Some(_)) => "needs_user_action",
                        (Ok(()), Some(_)) => "available",
                        _ => "unsupported",
                    }
                    .to_string(),
                    note: "launch only; screen control is separate".to_string(),
                })
                .collect()
        }
    }

    /// Starts a desktop entry and returns gio's exit status. No output is
    /// captured: a launched app inherits gio's standard streams, and a captured
    /// pipe kept the receiver waiting until the app exited (found by RC-T17).
    pub(crate) fn start_entry(entry: &Path) -> io::Result<std::process::ExitStatus> {
        Command::new("gio")
            .arg("launch")
            .arg(entry)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }

    fn window_text(window: Option<bool>) -> &'static str {
        match window {
            Some(true) => "observed",
            Some(false) => "not observed",
            None => "unknown (no supported window query)",
        }
    }

    /// The receiver must run inside the user's active, unlocked graphical
    /// session; it never borrows another session's display.
    fn session_ready() -> Result<(), (Status, String)> {
        let display = std::env::var_os("WAYLAND_DISPLAY").or_else(|| std::env::var_os("DISPLAY"));
        let session = std::env::var("XDG_SESSION_ID").ok().filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric()));
        let (Some(_), Some(session)) = (display, session) else {
            return Err((Status::Unsupported, "this receiver is not running inside a graphical login session".to_string()));
        };
        let properties = Command::new("loginctl")
            .args(["show-session", &session, "-p", "Type", "-p", "Active", "-p", "LockedHint"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or_default();
        judge_session(&properties)
    }

    pub(crate) fn judge_session(properties: &str) -> Result<(), (Status, String)> {
        let value = |key: &str| properties.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('=')).unwrap_or("");
        if !matches!(value("Type"), "wayland" | "x11") {
            return Err((Status::Unsupported, format!("the session type '{}' has no graphical display", value("Type"))));
        }
        if value("Active") != "yes" {
            return Err((Status::NeedsUserAction, "the graphical session is not in the foreground; switch to it and retry".to_string()));
        }
        if value("LockedHint") == "yes" {
            return Err((Status::NeedsUserAction, "the graphical session is locked; unlock it and retry".to_string()));
        }
        Ok(())
    }

    fn desktop_entry(desktop_id: &str) -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let data_home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| home.map(|h| h.join(".local/share")));
        let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
        data_home
            .into_iter()
            .chain(data_dirs.split(':').filter(|d| !d.is_empty()).map(PathBuf::from))
            .map(|dir| dir.join("applications").join(desktop_id))
            .find(|path| path.is_file())
    }

    fn entry_name(entry: &Path) -> Option<String> {
        fs::read_to_string(entry).ok()?.lines().find_map(|l| l.strip_prefix("Name=").map(str::to_string))
    }

    /// Whether a Flatpak instance of the app runs, and whether KWin lists a
    /// window with its name. Neither proves the user can see or control it.
    fn observe(app_id: &str, name: &str) -> (bool, Option<bool>) {
        let running = Command::new("flatpak")
            .args(["ps", "--columns=application"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).lines().any(|l| l.trim() == app_id))
            .unwrap_or(false);
        let window = Command::new("gdbus")
            .args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", "/WindowsRunner", "--method", "org.kde.krunner1.Match", name])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).contains(&format!("'{name}'")));
        (running, window)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::desktop::{judge_session, start_entry, DesktopLauncher, ALLOWLIST};
    use nexees_feasibility::remote::{Platform, Status};

    #[test]
    fn a_locked_or_background_session_needs_the_user() {
        let locked = judge_session("Type=wayland\nActive=yes\nLockedHint=yes\n");
        assert!(matches!(locked, Err((Status::NeedsUserAction, _))));
        let background = judge_session("Type=x11\nActive=no\nLockedHint=no\n");
        assert!(matches!(background, Err((Status::NeedsUserAction, _))));
    }

    #[test]
    fn a_session_without_a_display_is_unsupported() {
        assert!(matches!(judge_session("Type=tty\nActive=yes\nLockedHint=no\n"), Err((Status::Unsupported, _))));
        assert!(matches!(judge_session(""), Err((Status::Unsupported, _))));
    }

    #[test]
    fn an_active_unlocked_graphical_session_is_ready() {
        assert!(judge_session("Type=wayland\nActive=yes\nLockedHint=no\n").is_ok());
    }

    #[test]
    fn an_app_off_the_allowlist_is_denied_before_anything_runs() {
        let (status, detail) = DesktopLauncher.launch("test", "firefox", u64::MAX);
        assert_eq!(status, Status::Denied);
        assert!(detail.contains("not on the launch allowlist"));
    }

    /// Reproduces the TASK-003 RC-T17 finding with a harmless program that
    /// keeps running, as a launched app does.
    #[test]
    fn a_launched_app_that_keeps_running_does_not_hold_the_receiver() {
        let dir = std::env::temp_dir().join(format!("nexees-launch-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("nexees-test-sleeper.desktop");
        std::fs::write(&entry, "[Desktop Entry]\nType=Application\nName=Nexees test sleeper\nExec=sleep 7.25\n").unwrap();
        let started = std::time::Instant::now();
        let status = start_entry(&entry).unwrap();
        let elapsed = started.elapsed();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(status.success());
        assert!(elapsed < std::time::Duration::from_secs(5), "gio launch held the caller for {elapsed:?}");
    }

    #[test]
    fn the_allowlist_holds_only_the_owner_approved_app() {
        assert_eq!(ALLOWLIST, [("hoptodesk", "com.hoptodesk.HopToDesk.desktop")]);
    }
}
