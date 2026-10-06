//! The registry of a device's workspaces and their lifecycle (core/workspaces/workspace_registry,
//! C1, B18, ST-WORKSPACE): create, open, close and list workspaces, each with a stable ID, a kind
//! (CODE or LCL), a name, a device-local root and its content availability.
//!
//! - **Persisted.** A workspace is a [`WorkspaceRecord`], which every participating device knows,
//!   and this device's [`Replica`] of it, with its root. Both live in the device's state store and
//!   are written in one transaction, so a workspace exists with both or not at all.
//! - **Stable IDs.** The caller gives a new workspace its ID, from a source of its choice; the
//!   registry refuses an ID it already holds, and never changes one.
//! - **One root, one workspace.** A root must be an existing directory, given as an absolute
//!   path, and is kept in its canonical form. It may not be, contain or lie inside another
//!   workspace's root on the device. Roots are compared by the identity of their directories on
//!   the file system, so a symbolic link, a bind mount or another spelling of the same directory
//!   cannot make one workspace's files another's: no workspace's scope reaches into another's.
//!   A root that has since been moved or replaced by a link is not opened.
//! - **Open and close set the foreground.** Opening a workspace makes it a client's foreground
//!   workspace, the one that client shows (C2, B2); closing it leaves the client showing none.
//!   Neither changes an agent session, a grant or any other workspace: an agent stays bound to
//!   the workspace its session names, whatever a client shows (C3, R11, B3, B4).

use std::fmt;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use nexees_domain::client::ClientState;
use nexees_domain::errors::DomainError;
use nexees_domain::ids::{ClientId, DeviceId, WorkspaceId};
use nexees_domain::schema::Record;
use nexees_domain::text::Label;
use nexees_domain::time::Timestamp;
use nexees_domain::workspace::{
    ContentAvailability, DeviceRoot, Replica, ReplicaFields, WorkspaceKind, WorkspaceRecord,
};
use nexees_state::state_store::{StateError, StateStore, Write};

/// The longest root a workspace may have, in bytes of its canonical path: short enough that a
/// window's channel carries a workspace with its root and its name in one message.
pub const MAX_ROOT_BYTES: usize = 1024;

/// A workspace as one device knows it: the record that every participating device shares, and
/// this device's replica of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// Its ID, kind and name.
    pub record: WorkspaceRecord,
    /// Where its content is on this device, if anywhere.
    pub replica: Replica,
}

impl Workspace {
    /// Its ID.
    pub fn id(&self) -> &WorkspaceId {
        &self.record.workspace_id
    }
}

/// Why a path cannot be a workspace's root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootProblem {
    /// It is not an absolute path.
    NotAbsolute,
    /// Nothing is there, or it cannot be read.
    Missing,
    /// It is not a directory.
    NotADirectory,
    /// Its canonical path is not UTF-8 text, or is longer than [`MAX_ROOT_BYTES`].
    Unusable,
    /// The recorded root now resolves to another directory: it was moved, or replaced by a link.
    Moved,
}

/// Why the registry refused. A refused call changed nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// The state store failed.
    Store(StateError),
    /// A value is not valid.
    Invalid(DomainError),
    /// The device knows no workspace with this ID.
    NotFound(WorkspaceId),
    /// A workspace already has this ID; an ID is never given twice.
    Exists(WorkspaceId),
    /// The path cannot be a workspace's root.
    Root(RootProblem),
    /// The root is, contains or lies inside the root of this workspace.
    Overlaps(WorkspaceId),
    /// Only a listing of this workspace is known here; its content is on another device (A3).
    NotHere(WorkspaceId),
    /// The client does not show this workspace.
    NotOpen(WorkspaceId),
}

impl From<StateError> for RegistryError {
    fn from(error: StateError) -> Self {
        Self::Store(error)
    }
}

impl From<DomainError> for RegistryError {
    fn from(error: DomainError) -> Self {
        Self::Invalid(error)
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(f, "the state store failed: {error}"),
            Self::Invalid(error) => write!(f, "{error}"),
            Self::NotFound(id) => write!(f, "this device has no workspace {id}"),
            Self::Exists(id) => write!(f, "a workspace already has the ID {id}"),
            Self::Root(problem) => f.write_str(match problem {
                RootProblem::NotAbsolute => "the root is not an absolute path",
                RootProblem::Missing => "the root does not exist or cannot be read",
                RootProblem::NotADirectory => "the root is not a folder",
                RootProblem::Unusable => "the root's path is not text or is too long",
                RootProblem::Moved => "the root was moved, or replaced by a link",
            }),
            Self::Overlaps(id) => {
                write!(f, "the root shares files with the root of workspace {id}")
            }
            Self::NotHere(id) => write!(f, "the content of workspace {id} is not on this device"),
            Self::NotOpen(id) => write!(f, "workspace {id} is not open"),
        }
    }
}

/// Every workspace that has a replica on `device`, by ID.
pub fn list(store: &StateStore, device: &DeviceId) -> Result<Vec<Workspace>, RegistryError> {
    let mut found = Vec::new();
    for replica in store.all::<Replica>()? {
        if &replica.fields().device_id == device {
            let record = record_of(store.get(replica.fields().workspace_id.as_str())?, &replica)?;
            found.push(Workspace { record, replica });
        }
    }
    found.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    Ok(found)
}

/// The workspace whose root on `device` is the directory `root` names, however it is spelled.
pub fn find_by_root(
    store: &StateStore,
    device: &DeviceId,
    root: &Path,
) -> Result<Option<Workspace>, RegistryError> {
    let wanted = CheckedRoot::check(root)?;
    Ok(list(store, device)?
        .into_iter()
        .find(|workspace| root_of(workspace).is_some_and(|other| wanted.is_same(other))))
}

/// The workspace `client` shows, if any.
pub fn foreground(
    store: &StateStore,
    client: &ClientId,
) -> Result<Option<WorkspaceId>, RegistryError> {
    Ok(store
        .get::<ClientState>(client.as_str())?
        .and_then(|state| state.foreground_workspace))
}

/// Creates a workspace on `device` with the ID `id`, whose content is the directory `root`.
pub fn create(
    store: &mut StateStore,
    now: Timestamp,
    device: &DeviceId,
    id: WorkspaceId,
    kind: WorkspaceKind,
    name: Label,
    root: &Path,
) -> Result<Workspace, RegistryError> {
    let checked = CheckedRoot::check(root)?;
    let record = WorkspaceRecord {
        workspace_id: id.clone(),
        kind,
        name,
    };
    let replica = Replica::try_from(ReplicaFields {
        workspace_id: id,
        device_id: device.clone(),
        root: Some(DeviceRoot::new(checked.text()?)?),
        availability: ContentAvailability::Local,
    })?;
    // The checks and the writes are one transaction, so two creations cannot both pass them.
    store.write(now, |w| {
        if w.get::<WorkspaceRecord>(record.workspace_id.as_str())?
            .is_some()
        {
            return Ok(Err(RegistryError::Exists(record.workspace_id.clone())));
        }
        for other in w.all::<Replica>()? {
            let other_root = other.fields().root.as_ref();
            if &other.fields().device_id == device
                && other_root.is_some_and(|r| checked.overlaps(Path::new(r.as_str())))
            {
                return Ok(Err(RegistryError::Overlaps(
                    other.fields().workspace_id.clone(),
                )));
            }
        }
        w.insert(&record)?;
        w.insert(&replica)?;
        Ok(Ok(Workspace {
            record: record.clone(),
            replica: replica.clone(),
        }))
    })?
}

/// Makes the workspace `id` the foreground workspace of `client` on `device`: the one it shows.
pub fn open(
    store: &mut StateStore,
    now: Timestamp,
    device: &DeviceId,
    client: &ClientId,
    id: &WorkspaceId,
) -> Result<Workspace, RegistryError> {
    store.write(now, |w| {
        let Some(workspace) = workspace_in(w, device, id)? else {
            return Ok(Err(RegistryError::NotFound(id.clone())));
        };
        let Some(root) = root_of(&workspace) else {
            return Ok(Err(RegistryError::NotHere(id.clone())));
        };
        if let Err(problem) = CheckedRoot::recorded(root) {
            return Ok(Err(RegistryError::Root(problem)));
        }
        w.put(&ClientState {
            client_id: client.clone(),
            device_id: device.clone(),
            foreground_workspace: Some(id.clone()),
        })?;
        Ok(Ok(workspace))
    })?
}

/// Closes the workspace `id` for `client`: the client then shows no workspace.
pub fn close(
    store: &mut StateStore,
    now: Timestamp,
    client: &ClientId,
    id: &WorkspaceId,
) -> Result<(), RegistryError> {
    store.write(now, |w| match w.get::<ClientState>(client.as_str())? {
        Some(state) if state.foreground_workspace.as_ref() == Some(id) => {
            w.put(&ClientState {
                foreground_workspace: None,
                ..state
            })?;
            Ok(Ok(()))
        }
        _ => Ok(Err(RegistryError::NotOpen(id.clone()))),
    })?
}

/// The workspace `id` as `device` knows it, within a transaction.
fn workspace_in(
    w: &Write<'_>,
    device: &DeviceId,
    id: &WorkspaceId,
) -> Result<Option<Workspace>, StateError> {
    let Some(replica) = w
        .all::<Replica>()?
        .into_iter()
        .find(|r| &r.fields().device_id == device && &r.fields().workspace_id == id)
    else {
        return Ok(None);
    };
    let record = w.get::<WorkspaceRecord>(id.as_str())?;
    Ok(Some(Workspace {
        record: record_of(record, &replica)?,
        replica,
    }))
}

/// The record of `replica`'s workspace, which the store must hold.
fn record_of(
    record: Option<WorkspaceRecord>,
    replica: &Replica,
) -> Result<WorkspaceRecord, StateError> {
    record.ok_or_else(|| StateError::Missing {
        schema: WorkspaceRecord::SCHEMA.into(),
        id: replica.fields().workspace_id.as_str().into(),
    })
}

/// The root of a workspace on its device, when its content is there.
fn root_of(workspace: &Workspace) -> Option<&Path> {
    workspace
        .replica
        .fields()
        .root
        .as_ref()
        .map(|root| Path::new(root.as_str()))
}

/// The identity of a directory on the file system: its device and inode.
type Identity = (u64, u64);

/// A root as checked on this device: its canonical path, and the identity of each directory
/// from it up to the file system's root, the root's own first.
struct CheckedRoot {
    path: PathBuf,
    chain: Vec<Identity>,
}

impl CheckedRoot {
    /// Checks that `root` is an absolute path to an existing directory.
    fn check(root: &Path) -> Result<Self, RootProblem> {
        if !root.is_absolute() {
            return Err(RootProblem::NotAbsolute);
        }
        let path = fs::canonicalize(root).map_err(|_| RootProblem::Missing)?;
        if !fs::metadata(&path)
            .map_err(|_| RootProblem::Missing)?
            .is_dir()
        {
            return Err(RootProblem::NotADirectory);
        }
        let chain = identities(&path).ok_or(RootProblem::Missing)?;
        let checked = Self { path, chain };
        checked.text()?;
        Ok(checked)
    }

    /// Checks a root the registry recorded: it must still be a directory, under the same path.
    fn recorded(root: &Path) -> Result<Self, RootProblem> {
        let checked = Self::check(root)?;
        if checked.path != root {
            return Err(RootProblem::Moved);
        }
        Ok(checked)
    }

    /// The canonical path as text, within [`MAX_ROOT_BYTES`].
    fn text(&self) -> Result<&str, RootProblem> {
        self.path
            .to_str()
            .filter(|text| text.len() <= MAX_ROOT_BYTES)
            .ok_or(RootProblem::Unusable)
    }

    /// Whether `other` names this same directory.
    fn is_same(&self, other: &Path) -> bool {
        Self::identity_chain(other).is_some_and(|chain| chain.first() == self.chain.first())
    }

    /// Whether this root and the root `other` share files: one is the other or lies inside it.
    fn overlaps(&self, other: &Path) -> bool {
        match Self::identity_chain(other) {
            Some(chain) => {
                // Each chain starts with its own directory's identity.
                self.chain.first().is_some_and(|own| chain.contains(own))
                    || chain.first().is_some_and(|own| self.chain.contains(own))
            }
            // A root that is no longer on the disk is compared by its recorded path.
            None => self.path.starts_with(other) || other.starts_with(&self.path),
        }
    }

    fn identity_chain(path: &Path) -> Option<Vec<Identity>> {
        fs::canonicalize(path)
            .ok()
            .and_then(|path| identities(&path))
    }
}

/// The identities of `path` and of every directory above it, `path`'s own first.
fn identities(path: &Path) -> Option<Vec<Identity>> {
    path.ancestors()
        .map(|dir| fs::metadata(dir).ok().map(|meta| (meta.dev(), meta.ino())))
        .collect()
}

impl From<RootProblem> for RegistryError {
    fn from(problem: RootProblem) -> Self {
        Self::Root(problem)
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};

    use nexees_domain::authority::PermissionGrant;
    use nexees_domain::ids::{AgentSessionId, ModelId, ProviderId};
    use nexees_domain::lcl::SpecificationMode;
    use nexees_domain::model_capabilities::ProviderModel;
    use nexees_domain::revision::{Generation, RevisionId, WorkspaceRevision};
    use nexees_domain::session::{AgentSession, AgentSessionFields, AutonomyMode};

    use super::*;

    /// A scratch folder for one test, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let unique = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nexees-workspaces-{}-{unique}-{name}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }

        /// A new directory inside the scratch folder.
        fn dir(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).unwrap();
            path
        }

        fn store(&self) -> StateStore {
            StateStore::open(&self.0.join("state.db"), at(1)).unwrap().0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn at(ms: u64) -> Timestamp {
        Timestamp::from_unix_millis(ms)
    }

    fn id(value: &str) -> WorkspaceId {
        WorkspaceId::new(value).unwrap()
    }

    fn pc() -> DeviceId {
        DeviceId::new("pc").unwrap()
    }

    fn window() -> ClientId {
        ClientId::new("desktop-window").unwrap()
    }

    fn create_at(
        store: &mut StateStore,
        name: &str,
        kind: WorkspaceKind,
        root: &Path,
    ) -> Result<Workspace, RegistryError> {
        create(
            store,
            at(2),
            &pc(),
            id(name),
            kind,
            Label::new(name).unwrap(),
            root,
        )
    }

    #[test]
    fn created_workspaces_keep_their_ids_kinds_names_and_roots_across_a_restart() {
        let scratch = Scratch::new("persisted");
        let (code, lcl) = (scratch.dir("arch-dock"), scratch.dir("lcl-next"));
        let mut store = scratch.store();
        let made = create_at(&mut store, "w-code", WorkspaceKind::Code, &code).unwrap();
        create_at(
            &mut store,
            "w-lcl",
            WorkspaceKind::Lcl,
            &lcl.join("../lcl-next"),
        )
        .unwrap();
        assert_eq!(
            made.replica.fields().availability,
            ContentAvailability::Local
        );
        store.close(at(3)).unwrap();

        let store = scratch.store();
        let listed = list(&store, &pc()).unwrap();
        let seen: Vec<_> = listed
            .iter()
            .map(|w| {
                let root = w
                    .replica
                    .fields()
                    .root
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .to_owned();
                (
                    w.id().as_str().to_owned(),
                    w.record.kind,
                    w.record.name.to_string(),
                    root,
                )
            })
            .collect();
        // The root is kept in its canonical form, however it was spelled.
        assert_eq!(
            seen,
            vec![
                (
                    "w-code".into(),
                    WorkspaceKind::Code,
                    "w-code".into(),
                    code.display().to_string()
                ),
                (
                    "w-lcl".into(),
                    WorkspaceKind::Lcl,
                    "w-lcl".into(),
                    lcl.display().to_string()
                ),
            ]
        );
        assert_eq!(listed[0], made);
        // Another device knows none of them.
        assert!(
            list(&store, &DeviceId::new("phone").unwrap())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn an_id_is_given_once() {
        let scratch = Scratch::new("ids");
        let mut store = scratch.store();
        create_at(&mut store, "w", WorkspaceKind::Code, &scratch.dir("one")).unwrap();
        let again = create_at(&mut store, "w", WorkspaceKind::Lcl, &scratch.dir("two"));
        assert_eq!(again.unwrap_err(), RegistryError::Exists(id("w")));
        // Nothing of the refused workspace was written.
        assert_eq!(list(&store, &pc()).unwrap().len(), 1);
        assert_eq!(
            find_by_root(&store, &pc(), &scratch.dir("two")).unwrap(),
            None
        );
    }

    #[test]
    fn a_root_must_be_an_absolute_path_to_an_existing_folder_of_bounded_length() {
        let scratch = Scratch::new("roots");
        let mut store = scratch.store();
        fs::write(scratch.0.join("file.txt"), "not a folder").unwrap();
        let deep = scratch.dir(&vec!["d".repeat(250); 5].join("/"));
        for (root, problem) in [
            (PathBuf::from("relative/folder"), RootProblem::NotAbsolute),
            (scratch.0.join("missing"), RootProblem::Missing),
            (scratch.0.join("file.txt"), RootProblem::NotADirectory),
            (deep, RootProblem::Unusable),
        ] {
            assert_eq!(
                create_at(&mut store, "w", WorkspaceKind::Code, &root).unwrap_err(),
                RegistryError::Root(problem),
                "{}",
                root.display()
            );
        }
        assert!(list(&store, &pc()).unwrap().is_empty());
    }

    #[test]
    fn roots_never_share_files_whatever_their_spelling() {
        let scratch = Scratch::new("overlap");
        let outer = scratch.dir("projects/arch-dock");
        let mut store = scratch.store();
        create_at(&mut store, "outer", WorkspaceKind::Code, &outer).unwrap();
        symlink(&outer, scratch.0.join("alias")).unwrap();
        for (name, root) in [
            ("same", outer.clone()),
            ("inside", scratch.dir("projects/arch-dock/src")),
            ("containing", scratch.0.join("projects")),
            ("linked", scratch.0.join("alias")),
            ("linked-inside", scratch.0.join("alias/src")),
            ("dotted", scratch.0.join("projects/x/../arch-dock")),
        ] {
            fs::create_dir_all(scratch.0.join("projects/x")).unwrap();
            assert_eq!(
                create_at(&mut store, name, WorkspaceKind::Lcl, &root).unwrap_err(),
                RegistryError::Overlaps(id("outer")),
                "{name}"
            );
        }
        // A sibling shares nothing, and the same name in another folder is another folder.
        create_at(
            &mut store,
            "sibling",
            WorkspaceKind::Lcl,
            &scratch.dir("projects/lcl"),
        )
        .unwrap();
        create_at(
            &mut store,
            "other",
            WorkspaceKind::Code,
            &scratch.dir("elsewhere/arch-dock"),
        )
        .unwrap();
        // A workspace's root is found under any of its spellings.
        let found = find_by_root(&store, &pc(), &scratch.0.join("alias"))
            .unwrap()
            .unwrap();
        assert_eq!(found.id(), &id("outer"));
    }

    #[test]
    fn opening_sets_the_clients_foreground_and_closing_clears_it() {
        let scratch = Scratch::new("foreground");
        let mut store = scratch.store();
        create_at(&mut store, "a", WorkspaceKind::Code, &scratch.dir("a")).unwrap();
        create_at(&mut store, "b", WorkspaceKind::Lcl, &scratch.dir("b")).unwrap();
        assert_eq!(foreground(&store, &window()).unwrap(), None);
        assert_eq!(
            open(&mut store, at(3), &pc(), &window(), &id("a"))
                .unwrap()
                .id(),
            &id("a")
        );
        assert_eq!(foreground(&store, &window()).unwrap(), Some(id("a")));
        open(&mut store, at(4), &pc(), &window(), &id("b")).unwrap();
        assert_eq!(foreground(&store, &window()).unwrap(), Some(id("b")));
        // Only the workspace a client shows can be closed for it.
        assert_eq!(
            close(&mut store, at(5), &window(), &id("a")).unwrap_err(),
            RegistryError::NotOpen(id("a"))
        );
        close(&mut store, at(6), &window(), &id("b")).unwrap();
        assert_eq!(foreground(&store, &window()).unwrap(), None);
        // A workspace the device does not know is not opened, and another client is unaffected.
        assert_eq!(
            open(&mut store, at(7), &pc(), &window(), &id("c")).unwrap_err(),
            RegistryError::NotFound(id("c"))
        );
        let phone = ClientId::new("android-app").unwrap();
        assert_eq!(foreground(&store, &phone).unwrap(), None);
    }

    fn session_bound_to(workspace: &str) -> AgentSession {
        AgentSession::try_from(AgentSessionFields {
            session_id: AgentSessionId::new("claude-1").unwrap(),
            workspace_id: id(workspace),
            execution_device_id: pc(),
            checkout: WorkspaceRevision {
                workspace_id: id(workspace),
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
    fn switching_the_foreground_leaves_a_bound_agent_and_every_other_record_untouched() {
        // Claude runs on Arch Dock; the user switches to the LCL project and back
        // (workspace_model).
        let scratch = Scratch::new("isolation");
        let mut store = scratch.store();
        create_at(
            &mut store,
            "arch-dock",
            WorkspaceKind::Code,
            &scratch.dir("arch-dock"),
        )
        .unwrap();
        create_at(
            &mut store,
            "lcl-next",
            WorkspaceKind::Lcl,
            &scratch.dir("lcl-next"),
        )
        .unwrap();
        let session = session_bound_to("arch-dock");
        store.write(at(3), |w| w.insert(&session)).unwrap();
        let before = (
            store.all::<AgentSession>().unwrap(),
            store.all::<PermissionGrant>().unwrap(),
            list(&store, &pc()).unwrap(),
        );

        open(&mut store, at(4), &pc(), &window(), &id("arch-dock")).unwrap();
        open(&mut store, at(5), &pc(), &window(), &id("lcl-next")).unwrap();
        close(&mut store, at(6), &window(), &id("lcl-next")).unwrap();
        open(&mut store, at(7), &pc(), &window(), &id("lcl-next")).unwrap();

        let after = (
            store.all::<AgentSession>().unwrap(),
            store.all::<PermissionGrant>().unwrap(),
            list(&store, &pc()).unwrap(),
        );
        assert_eq!(after, before);
        assert_eq!(after.0[0].fields().workspace_id, id("arch-dock"));
        assert_eq!(foreground(&store, &window()).unwrap(), Some(id("lcl-next")));
    }

    #[test]
    fn a_listing_without_content_here_is_listed_but_not_opened() {
        let scratch = Scratch::new("remote-only");
        let mut store = scratch.store();
        let listing = Replica::try_from(ReplicaFields {
            workspace_id: id("phone-notes"),
            device_id: pc(),
            root: None,
            availability: ContentAvailability::RemoteOnly,
        })
        .unwrap();
        let record = WorkspaceRecord {
            workspace_id: id("phone-notes"),
            kind: WorkspaceKind::Lcl,
            name: Label::new("Phone notes").unwrap(),
        };
        store
            .write(at(2), |w| {
                w.insert(&record)?;
                w.insert(&listing)
            })
            .unwrap();
        assert_eq!(list(&store, &pc()).unwrap().len(), 1);
        assert_eq!(
            open(&mut store, at(3), &pc(), &window(), &id("phone-notes")).unwrap_err(),
            RegistryError::NotHere(id("phone-notes"))
        );
        assert_eq!(foreground(&store, &window()).unwrap(), None);
    }

    #[test]
    fn a_root_that_was_moved_or_replaced_by_a_link_is_not_opened() {
        let scratch = Scratch::new("moved");
        let (first, second) = (scratch.dir("first"), scratch.dir("second"));
        let mut store = scratch.store();
        create_at(&mut store, "first", WorkspaceKind::Code, &first).unwrap();
        create_at(&mut store, "second", WorkspaceKind::Code, &second).unwrap();
        // The first root is replaced by a link to the second: opening the first would show the
        // second's files under the first's name.
        fs::rename(&first, scratch.0.join("first-moved")).unwrap();
        symlink(&second, &first).unwrap();
        assert_eq!(
            open(&mut store, at(3), &pc(), &window(), &id("first")).unwrap_err(),
            RegistryError::Root(RootProblem::Moved)
        );
        fs::remove_file(&first).unwrap();
        assert_eq!(
            open(&mut store, at(4), &pc(), &window(), &id("first")).unwrap_err(),
            RegistryError::Root(RootProblem::Missing)
        );
        // Another device's replica of a workspace is never opened here (B18).
        assert_eq!(
            open(
                &mut store,
                at(5),
                &DeviceId::new("phone").unwrap(),
                &window(),
                &id("second")
            )
            .unwrap_err(),
            RegistryError::NotFound(id("second"))
        );
        assert_eq!(foreground(&store, &window()).unwrap(), None);
    }
}
