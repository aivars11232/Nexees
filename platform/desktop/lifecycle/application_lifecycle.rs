//! The Desktop host's lifetime and its start at login (RC-04, RC-05, RC-24, PD-STARTUP).
//!
//! - **One host per user.** [`HostLock`] holds an exclusive lock on `host.lock` in the channel
//!   folder for as long as the host runs. A second host finds it held and stops, so a window that
//!   opens again reattaches to the running host instead of starting a second executor (RC-T14).
//!   The kernel releases the lock when the host's process ends, however it ends.
//! - **Lifetime.** The host runs only inside the user's login session, at the user's privilege.
//!   A window starts it, and it stops soon after its last window detaches; or the user's start at
//!   login starts it, as a unit that systemd binds to the graphical session and stops with it.
//!   Nothing starts it before anyone logs in or keeps it after logout: that would take a system
//!   service or linger, which the design rules out. Started outside a login session, the host
//!   refuses to run and reports that as unsupported (RC-05).
//! - **Nothing of its starter's.** The host outlives whatever started it, so before it opens
//!   anything of its own it closes the descriptors it inherited ([`close_inherited_descriptors`]):
//!   a window's backend passes its own on without meaning to (RC-04).
//! - **Start at login.** [`LoginStart`] writes a user-level XDG autostart entry only when the
//!   user enables it, and removes it on disable or uninstall. It removes only an entry it wrote;
//!   another program's entry of the same name is reported and left alone (RC-24). Nothing here
//!   installs a system service, enables linger, adds a firewall rule or sets up automatic login.

use std::fmt;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::os::fd::RawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::transport::local_ipc::ChannelFolder;

/// The lock file's name in the channel folder.
const LOCK: &str = "host.lock";
/// The autostart entry's file name.
const ENTRY: &str = "nexees-host.desktop";
/// The key that marks an autostart entry as one this module wrote.
const OWNED: &str = "X-Nexees-Owned=1";

/// Why a lifecycle operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// The host program's path cannot be written into an autostart entry safely: it is not
    /// absolute, or it holds characters the entry would have to escape.
    UnsafeCommand(String),
    /// An autostart entry of Nexees's name exists that Nexees did not write; it is left alone.
    ForeignEntry(PathBuf),
    /// No configuration folder: neither `XDG_CONFIG_HOME` nor `HOME` names an absolute path.
    NoConfigFolder,
    /// A failure of the operating system, with its reason.
    Io(String),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsafeCommand(reason) => write!(f, "the host command is unsafe: {reason}"),
            Self::ForeignEntry(path) => write!(f, "{} was not written by Nexees", path.display()),
            Self::NoConfigFolder => write!(f, "no configuration folder"),
            Self::Io(reason) => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for LifecycleError {}

impl From<io::Error> for LifecycleError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Whether this process runs with root's privileges. The host refuses to (RC-23): it runs at the
/// user's privilege, and no privileged helper exists.
pub fn elevated() -> bool {
    nix::unistd::geteuid().is_root()
}

/// Closes every descriptor this process holds besides standard input, output and error, and
/// returns how many it closed. Only for the start of a process, before it opens anything of its
/// own: then every other descriptor is one its starter left open, which a host that outlives its
/// starter must not keep (RC-04).
pub fn close_inherited_descriptors() -> io::Result<usize> {
    let open: Vec<RawFd> = fs::read_dir("/proc/self/fd")?
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
        .filter(|descriptor| *descriptor > 2)
        .collect();
    // The listing's own descriptor is among them and closed already, so its close fails.
    Ok(open
        .into_iter()
        .filter(|descriptor| nix::unistd::close(*descriptor).is_ok())
        .count())
}

/// The host's single-instance lock. It is held while this value lives.
#[derive(Debug)]
pub struct HostLock {
    _file: File,
}

impl HostLock {
    /// Takes the host lock in `folder`; `None` when another host holds it.
    pub fn acquire(folder: &ChannelFolder) -> Result<Option<Self>, LifecycleError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(folder.path().join(LOCK))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(error)) => Err(error.into()),
        }
    }
}

/// Whether the host's start at login is set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginStartStatus {
    /// No entry: the host does not start at login (the default).
    Disabled,
    /// Nexees's entry exists.
    Enabled,
    /// An entry of Nexees's name exists that Nexees did not write.
    Foreign,
}

/// The user-level start at login: one XDG autostart entry in the user's configuration folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStart {
    folder: PathBuf,
}

impl LoginStart {
    /// The autostart folder under `config_home`, the user's configuration folder, which must be
    /// an absolute path.
    pub fn new(config_home: &Path) -> Result<Self, LifecycleError> {
        if !config_home.is_absolute() {
            return Err(LifecycleError::NoConfigFolder);
        }
        Ok(Self {
            folder: config_home.join("autostart"),
        })
    }

    /// The autostart folder of this user, from `XDG_CONFIG_HOME`, or `HOME/.config` when that is
    /// unset, as the XDG base directory specification says.
    pub fn from_environment() -> Result<Self, LifecycleError> {
        let absolute = |value: std::ffi::OsString| {
            let path = PathBuf::from(value);
            path.is_absolute().then_some(path)
        };
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .and_then(absolute)
            .or_else(|| {
                std::env::var_os("HOME")
                    .and_then(absolute)
                    .map(|home| home.join(".config"))
            })
            .ok_or(LifecycleError::NoConfigFolder)?;
        Self::new(&config)
    }

    /// The entry's path.
    pub fn entry(&self) -> PathBuf {
        self.folder.join(ENTRY)
    }

    /// Whether start at login is set up.
    pub fn status(&self) -> Result<LoginStartStatus, LifecycleError> {
        match fs::read_to_string(self.entry()) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LoginStartStatus::Disabled),
            Err(error) => Err(error.into()),
            Ok(text) if text.lines().any(|line| line == OWNED) => Ok(LoginStartStatus::Enabled),
            Ok(_) => Ok(LoginStartStatus::Foreign),
        }
    }

    /// Starts `host`, the absolute path of the host program, at the user's next logins, to keep
    /// running without a window: the user's explicit opt-in. Refused when another program's
    /// entry has Nexees's name.
    pub fn enable(&self, host: &Path) -> Result<(), LifecycleError> {
        let text = desktop_entry(host)?;
        if self.status()? == LoginStartStatus::Foreign {
            return Err(LifecycleError::ForeignEntry(self.entry()));
        }
        fs::create_dir_all(&self.folder)?;
        // Written beside its final name and renamed into place, so a crash leaves either the old
        // entry or the new one, never half of one.
        let partial = self.folder.join(format!(".{ENTRY}.partial"));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&partial)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&partial, self.entry())?;
        Ok(())
    }

    /// Removes Nexees's entry; returns whether there was one. Another program's entry of the
    /// same name is not removed, and is reported.
    pub fn disable(&self) -> Result<bool, LifecycleError> {
        match self.status()? {
            LoginStartStatus::Disabled => Ok(false),
            LoginStartStatus::Foreign => Err(LifecycleError::ForeignEntry(self.entry())),
            LoginStartStatus::Enabled => {
                fs::remove_file(self.entry())?;
                Ok(true)
            }
        }
    }
}

/// The autostart entry that runs `host` to keep running after login. `host` is written quoted
/// into `Exec`; a path that would need escaping there is refused rather than escaped, and so is
/// a relative path or anything that a shell would interpret.
fn desktop_entry(host: &Path) -> Result<String, LifecycleError> {
    let text = host
        .to_str()
        .ok_or_else(|| LifecycleError::UnsafeCommand("not valid UTF-8".into()))?;
    if !host.is_absolute() {
        return Err(LifecycleError::UnsafeCommand(format!(
            "{text} is not absolute"
        )));
    }
    if let Some(c) = text
        .chars()
        .find(|c| c.is_control() || matches!(c, '"' | '`' | '$' | '\\' | '%'))
    {
        return Err(LifecycleError::UnsafeCommand(format!(
            "the path contains {c:?}"
        )));
    }
    Ok(format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Nexees host\n\
         Comment=Starts the Nexees host at login; enabled and disabled in Nexees\n\
         Exec=\"{text}\" serve --keep-running\n\
         Terminal=false\n\
         NoDisplay=true\n\
         {OWNED}\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::local_ipc::tests::Runtime;

    #[test]
    fn only_one_host_holds_the_lock_at_a_time() {
        let runtime = Runtime::new("lock");
        let folder = ChannelFolder::open(runtime.path()).unwrap();
        let first = HostLock::acquire(&folder).unwrap();
        assert!(first.is_some());
        assert!(
            HostLock::acquire(&folder).unwrap().is_none(),
            "a second host"
        );
        drop(first);
        assert!(
            HostLock::acquire(&folder).unwrap().is_some(),
            "released when the first host ends"
        );
    }

    #[test]
    fn start_at_login_exists_only_while_enabled_and_runs_the_host_to_keep_running() {
        let config = Runtime::new("autostart");
        let login = LoginStart::new(config.path()).unwrap();
        assert_eq!(login.status().unwrap(), LoginStartStatus::Disabled);
        login
            .enable(Path::new("/opt/Nexees app/nexees-host"))
            .unwrap();
        assert_eq!(login.status().unwrap(), LoginStartStatus::Enabled);
        let entry = fs::read_to_string(login.entry()).unwrap();
        assert!(entry.contains("Exec=\"/opt/Nexees app/nexees-host\" serve --keep-running\n"));
        assert!(entry.starts_with("[Desktop Entry]\n"));
        // Enabling again rewrites Nexees's own entry.
        login.enable(Path::new("/opt/nexees/nexees-host")).unwrap();
        assert!(login.disable().unwrap());
        assert_eq!(login.status().unwrap(), LoginStartStatus::Disabled);
        assert!(!login.disable().unwrap(), "nothing left to remove");
        assert_eq!(
            fs::read_dir(config.path().join("autostart"))
                .unwrap()
                .count(),
            0
        );
    }

    #[test]
    fn a_relative_or_shell_like_command_is_refused() {
        let config = Runtime::new("unsafe");
        let login = LoginStart::new(config.path()).unwrap();
        for host in [
            "nexees-host",
            "/opt/nexees/$(rm -rf ~)",
            "/opt/nexees/`id`",
            "/opt/nexees/a\"b",
            "/opt/nexees/%f",
            "/opt/nexees/a\\b",
            "/opt/nexees/a\nb",
        ] {
            assert!(
                matches!(
                    login.enable(Path::new(host)),
                    Err(LifecycleError::UnsafeCommand(_))
                ),
                "{host:?}"
            );
        }
        assert_eq!(login.status().unwrap(), LoginStartStatus::Disabled);
    }

    #[test]
    fn another_programs_entry_of_the_same_name_is_never_touched() {
        let config = Runtime::new("foreign");
        let login = LoginStart::new(config.path()).unwrap();
        fs::create_dir_all(config.path().join("autostart")).unwrap();
        let foreign = "[Desktop Entry]\nType=Application\nExec=/usr/bin/other\n";
        fs::write(login.entry(), foreign).unwrap();
        assert_eq!(login.status().unwrap(), LoginStartStatus::Foreign);
        assert!(matches!(
            login.enable(Path::new("/opt/nexees/nexees-host")),
            Err(LifecycleError::ForeignEntry(_))
        ));
        assert!(matches!(
            login.disable(),
            Err(LifecycleError::ForeignEntry(_))
        ));
        assert_eq!(fs::read_to_string(login.entry()).unwrap(), foreign);
    }

    #[test]
    fn the_configuration_folder_must_be_absolute() {
        assert_eq!(
            LoginStart::new(Path::new("relative")).unwrap_err(),
            LifecycleError::NoConfigFolder
        );
    }
}
