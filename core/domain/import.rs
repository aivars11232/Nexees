//! LCL import transactions: what is imported, where to, from which staged content and under
//! which mapping (B22, I1–I5, ST-IMPORT).
//!
//! The target device, workspace and base revision are fixed when the import begins. UI focus
//! cannot redirect them. Staged content is untrusted and inactive: committing it requires a
//! reviewed preview and an unchanged destination, and the result is a draft specification,
//! never an adopted one ([`crate::lcl`]). Intake, staging limits and archive checks are TASK-051's,
//! structure planning and preview TASK-052's; this module fixes the record they share.

use serde::{Deserialize, Serialize};

use crate::errors::{Blocker, DomainError, ErrorKind};
use crate::ids::{DeviceId, ImportId, WorkspaceId};
use crate::revision::{ContentHash, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, ensure_unique, validated_record};
use crate::time::Timestamp;
use crate::workspace::RelativePath;

/// Where an import stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ImportState {
    /// Inspected and staged in app-private staging; nothing in the workspace changed.
    Staged,
    /// The plan was shown to the user for review.
    Previewed,
    /// Published into the workspace as a draft revision. Final.
    Committed,
    /// Cancelled; only the import's own staging is removed. Final.
    Cancelled,
    /// Kept by the user's choice as an inactive draft that is never executed. Final.
    RetainedDraft,
    /// Stopped safely, with the reason; the workspace is unchanged. Final.
    Failed(Blocker),
}

impl ImportState {
    const fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Committed | Self::Cancelled | Self::RetainedDraft | Self::Failed(_)
        )
    }
}

/// One staged file and the hash of its staged content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagedFile {
    /// Its path in the source, which is also its path in staging.
    pub path: RelativePath,
    /// The hash of the staged content.
    pub content: ContentHash,
}

/// Where one staged file goes in the workspace (I3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathMapping {
    /// The staged file.
    pub source: RelativePath,
    /// Its path in the workspace.
    pub destination: RelativePath,
}

/// The fields of an [`ImportTransaction`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportTransactionFields {
    /// The import.
    pub import_id: ImportId,
    /// The device whose workspace receives the import.
    pub target_device_id: DeviceId,
    /// The workspace that receives it.
    pub workspace_id: WorkspaceId,
    /// The workspace revision the plan was made against, a revision of `workspace_id`.
    pub base_revision: WorkspaceRevision,
    /// The staged files, each path once.
    pub staged: Vec<StagedFile>,
    /// The planned destinations. Each source is a staged file, and no two sources share a
    /// destination. Staged files left out of the mapping are reported, not imported.
    pub mapping: Vec<PathMapping>,
    /// Where the import stands.
    pub state: ImportState,
    /// When it began.
    pub started_at: Timestamp,
}

impl Validate for ImportTransactionFields {
    fn validate(&self) -> Result<(), DomainError> {
        self.base_revision
            .ensure_of(&self.workspace_id, "base_revision")?;
        let staged: Vec<&RelativePath> = self.staged.iter().map(|f| &f.path).collect();
        ensure_unique(&staged, "staged")?;
        let sources: Vec<&RelativePath> = self.mapping.iter().map(|m| &m.source).collect();
        ensure_unique(&sources, "mapping.source")?;
        let destinations: Vec<&RelativePath> =
            self.mapping.iter().map(|m| &m.destination).collect();
        ensure_unique(&destinations, "mapping.destination")?;
        if sources.iter().any(|source| !staged.contains(source)) {
            return Err(DomainError::new("mapping.source", ErrorKind::Missing));
        }
        if matches!(self.state, ImportState::Previewed | ImportState::Committed)
            && self.mapping.is_empty()
        {
            return Err(DomainError::new("mapping", ErrorKind::Empty));
        }
        Ok(())
    }
}

validated_record!(
    /// One import transaction (B22): `import -> target device + workspace + base revision +
    /// staged content hashes + source-to-destination mapping`.
    ImportTransaction,
    ImportTransactionFields
);

impl ImportTransaction {
    /// The import in `next` state: a staged import may be previewed, and an unfinished one
    /// cancelled, retained as an inactive draft or failed. Committing goes through
    /// [`ImportTransaction::commit`], which checks the destination.
    pub fn advance(&self, next: ImportState) -> Result<Self, DomainError> {
        let allowed = match (&self.0.state, &next) {
            (ImportState::Staged, ImportState::Previewed) => true,
            (
                current,
                ImportState::Cancelled | ImportState::RetainedDraft | ImportState::Failed(_),
            ) => !current.is_final(),
            _ => false,
        };
        if !allowed {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        Self::try_from(ImportTransactionFields {
            state: next,
            ..self.0.clone()
        })
    }

    /// The import committed into `current_destination`, the workspace revision now.
    ///
    /// Only a previewed import commits, and only while the destination is the revision the
    /// preview was made against; a changed destination invalidates the preview, which must be
    /// made again (B22, I4, IM-08).
    pub fn commit(&self, current_destination: &WorkspaceRevision) -> Result<Self, DomainError> {
        if self.0.state != ImportState::Previewed {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        if current_destination != &self.0.base_revision {
            return Err(DomainError::new("base_revision", ErrorKind::Stale));
        }
        Self::try_from(ImportTransactionFields {
            state: ImportState::Committed,
            ..self.0.clone()
        })
    }
}

impl Record for ImportTransaction {
    const SCHEMA: &'static str = "nexees.import.transaction";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::BlockerKind;
    use crate::revision::RevisionId;
    use crate::schema::Versioned;
    use crate::text::Note;

    fn revision(r: &str) -> WorkspaceRevision {
        WorkspaceRevision {
            workspace_id: WorkspaceId::new("lcl-b").unwrap(),
            revision: RevisionId::new(r).unwrap(),
        }
    }

    fn path(p: &str) -> RelativePath {
        RelativePath::new(p).unwrap()
    }

    fn staged(p: &str) -> StagedFile {
        StagedFile {
            path: path(p),
            content: ContentHash::new("c".repeat(64)).unwrap(),
        }
    }

    fn import(state: ImportState) -> ImportTransactionFields {
        ImportTransactionFields {
            import_id: ImportId::new("i1").unwrap(),
            target_device_id: DeviceId::new("phone").unwrap(),
            workspace_id: WorkspaceId::new("lcl-b").unwrap(),
            base_revision: revision("r1"),
            staged: vec![
                staged("pack/master.lcl.txt"),
                staged("pack/tasks/t1.lcl.txt"),
                staged("pack/readme.md"),
            ],
            mapping: vec![
                PathMapping {
                    source: path("pack/master.lcl.txt"),
                    destination: path("master.lcl.txt"),
                },
                PathMapping {
                    source: path("pack/tasks/t1.lcl.txt"),
                    destination: path("tasks/t1.lcl.txt"),
                },
            ],
            state,
            started_at: Timestamp::from_unix_millis(1),
        }
    }

    #[test]
    fn a_previewed_import_commits_only_into_the_reviewed_destination() {
        let previewed = ImportTransaction::try_from(import(ImportState::Staged))
            .unwrap()
            .advance(ImportState::Previewed)
            .unwrap();
        assert_eq!(
            previewed.commit(&revision("r2")).unwrap_err(),
            DomainError::new("base_revision", ErrorKind::Stale)
        );
        let committed = previewed.commit(&revision("r1")).unwrap();
        assert_eq!(committed.fields().state, ImportState::Committed);
        assert_eq!(
            committed.advance(ImportState::Cancelled).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn nothing_commits_without_a_preview() {
        let staged = ImportTransaction::try_from(import(ImportState::Staged)).unwrap();
        assert_eq!(
            staged.commit(&revision("r1")).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
        assert_eq!(
            staged.advance(ImportState::Committed).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn an_unfinished_import_can_stop_safely_in_every_final_state() {
        let failed = ImportState::Failed(Blocker {
            kind: BlockerKind::LimitReached,
            detail: Note::new("The archive expands beyond the configured limit.").unwrap(),
        });
        for stop in [ImportState::Cancelled, ImportState::RetainedDraft, failed] {
            for start in [ImportState::Staged, ImportState::Previewed] {
                let import = ImportTransaction::try_from(import(start)).unwrap();
                assert!(import.advance(stop.clone()).is_ok());
            }
        }
    }

    #[test]
    fn mappings_cannot_collide_invent_sources_or_target_another_workspace() {
        let mut collide = import(ImportState::Staged);
        collide.mapping[1].destination = path("master.lcl.txt");
        assert_eq!(
            ImportTransaction::try_from(collide).unwrap_err(),
            DomainError::new("mapping.destination", ErrorKind::Duplicate)
        );

        let mut invented = import(ImportState::Staged);
        invented.mapping[0].source = path("pack/elsewhere.lcl.txt");
        assert_eq!(
            ImportTransaction::try_from(invented).unwrap_err(),
            DomainError::new("mapping.source", ErrorKind::Missing)
        );

        let mut twice = import(ImportState::Staged);
        twice.staged.push(staged("pack/readme.md"));
        assert_eq!(
            ImportTransaction::try_from(twice).unwrap_err().kind,
            ErrorKind::Duplicate
        );

        let mut elsewhere = import(ImportState::Staged);
        elsewhere.base_revision = WorkspaceRevision {
            workspace_id: WorkspaceId::new("lcl-a").unwrap(),
            revision: RevisionId::new("r1").unwrap(),
        };
        assert_eq!(
            ImportTransaction::try_from(elsewhere).unwrap_err(),
            DomainError::new("base_revision", ErrorKind::Mismatch)
        );

        let mut empty = import(ImportState::Previewed);
        empty.mapping.clear();
        assert_eq!(
            ImportTransaction::try_from(empty).unwrap_err().kind,
            ErrorKind::Empty
        );
    }

    #[test]
    fn imports_round_trip_and_reject_escaping_paths_on_decode() {
        let import = ImportTransaction::try_from(import(ImportState::Previewed)).unwrap();
        let text = serde_json::to_string(&Versioned(import.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<ImportTransaction>>(&text)
                .unwrap()
                .0,
            import
        );
        let escaping = text.replace(
            "\"destination\":\"master.lcl.txt\"",
            "\"destination\":\"../master.lcl.txt\"",
        );
        assert_ne!(escaping, text);
        assert!(serde_json::from_str::<Versioned<ImportTransaction>>(&escaping).is_err());
    }
}
