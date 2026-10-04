//! Grants and approvals: who may do what, where, under which permission epoch, until when
//! (ST-AUTHORITY, AD-04, RC-03, RC-17, RC-18).
//!
//! Authority is recorded and decided on the device that executes. These records never travel:
//! a sender may show a grant it was told about, but that display authorizes nothing (RC-10).
//! The remote records follow TASK-004's contracts exactly: [`RemoteGrant`] is
//! `type.sec_grant` and [`Approval`] is `type.sec_approval` (`docs/security/baseline.lcl.txt`).
//! Policy, risk classes and the decision itself belong to the permission engine (TASK-022,
//! TASK-055); these types only make every grant and approval explicit, scoped and checkable.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{
    ActionId, AgentSessionId, AllowlistKey, ApprovalId, DeviceId, ExtensionId, GrantCapability,
    GrantId, RequestId, UserId, WorkspaceId,
};
use crate::revision::{ContentHash, PermissionEpoch};
use crate::schema::{Record, SyncPolicy, Validate, ensure_unique, validated_record};
use crate::time::Timestamp;
use crate::workspace::RelativePath;

/// Who acts. Each kind is a separate principal; none inherits another's authority (AD-10).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Principal {
    /// The local user.
    User {
        /// The user.
        user_id: UserId,
    },
    /// An agent session; it never grants itself anything (authority_hierarchy).
    AgentSession {
        /// The session.
        session_id: AgentSessionId,
    },
    /// An editor extension, with its own read-only limits (AD-10).
    Extension {
        /// The extension.
        extension_id: ExtensionId,
    },
    /// A user on a paired device, acting through remote requests.
    RemotePeer {
        /// The peer device.
        device_id: DeviceId,
        /// The user on it.
        user_id: UserId,
    },
}

/// One target a grant covers. Grants list the narrowest targets; an empty scope is invalid,
/// never "everything".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ScopeEntry {
    /// A whole workspace.
    Workspace {
        /// The workspace.
        workspace_id: WorkspaceId,
    },
    /// One file or folder in a workspace.
    Path {
        /// The workspace.
        workspace_id: WorkspaceId,
        /// The file or folder.
        path: RelativePath,
    },
    /// One application of the launch allowlist (RC-11).
    App {
        /// Its allowlist key.
        allowlist_key: AllowlistKey,
    },
}

fn check_scope(scope: &[ScopeEntry]) -> Result<(), DomainError> {
    if scope.is_empty() {
        return Err(DomainError::new("scope", ErrorKind::Empty));
    }
    ensure_unique(scope, "scope")
}

fn check_expiry(issued: Timestamp, expires: Option<Timestamp>) -> Result<(), DomainError> {
    match expires {
        Some(expires) if expires <= issued => {
            Err(DomainError::new("expires_at", ErrorKind::OutOfRange))
        }
        _ => Ok(()),
    }
}

/// The fields of a [`PermissionGrant`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionGrantFields {
    /// The grant.
    pub grant_id: GrantId,
    /// Who may act.
    pub principal: Principal,
    /// The action, from the permission engine's closed registry.
    pub action: ActionId,
    /// The targets, at least one, each once.
    pub scope: Vec<ScopeEntry>,
    /// The epoch the grant was made under; a later epoch voids it.
    pub permission_epoch: PermissionEpoch,
    /// The local user who made it.
    pub granted_by: UserId,
    /// When it was made.
    pub granted_at: Timestamp,
    /// When it ends, if it does; after `granted_at`.
    pub expires_at: Option<Timestamp>,
    /// Set once revoked; a revoked grant is kept for audit and never reused.
    pub revoked: bool,
}

impl Validate for PermissionGrantFields {
    fn validate(&self) -> Result<(), DomainError> {
        check_scope(&self.scope)?;
        check_expiry(self.granted_at, self.expires_at)
    }
}

validated_record!(
    /// A local permission: a principal may perform one action on the listed targets.
    PermissionGrant,
    PermissionGrantFields
);

impl Record for PermissionGrant {
    const SCHEMA: &'static str = "nexees.authority.permission_grant";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The direction of remote control a grant allows (RC-03). The reverse direction is a separate
/// grant on the other device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// The phone controls the desktop.
    AndroidToDesktop,
    /// The desktop controls the phone.
    DesktopToAndroid,
}

/// The fields of a [`RemoteGrant`]: exactly those of `type.sec_grant`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteGrantFields {
    /// The grant.
    pub grant_id: GrantId,
    /// This device; grants are stored nowhere else.
    pub destination_device_id: DeviceId,
    /// The paired device allowed to send.
    pub peer_device_id: DeviceId,
    /// The user on that device.
    pub peer_user_id: UserId,
    /// The direction it allows, which must match the two devices' kinds.
    pub direction: Direction,
    /// The capability, from the closed grant registry.
    pub capability: GrantCapability,
    /// The narrowest targets, at least one, each once.
    pub scope: Vec<ScopeEntry>,
    /// Incremented on every revoke, stop, scope or lock-policy change.
    pub permission_epoch: PermissionEpoch,
    /// The local user who made it.
    pub granted_by: UserId,
    /// When it was made, on the destination's clock.
    pub granted_at: Timestamp,
    /// When it ends, if it does; after `granted_at`.
    pub expires_at: Option<Timestamp>,
    /// Set once revoked; a revoked grant is kept for audit and never reused.
    pub revoked: bool,
}

impl Validate for RemoteGrantFields {
    fn validate(&self) -> Result<(), DomainError> {
        if self.peer_device_id == self.destination_device_id {
            return Err(DomainError::new("peer_device_id", ErrorKind::Duplicate));
        }
        check_scope(&self.scope)?;
        check_expiry(self.granted_at, self.expires_at)
    }
}

validated_record!(
    /// One directional remote-control grant, stored only on its destination (RC-03). Pairing,
    /// an account or file synchronization never creates one.
    RemoteGrant,
    RemoteGrantFields
);

impl Record for RemoteGrant {
    const SCHEMA: &'static str = "nexees.authority.remote_grant";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The fields of an [`Approval`]: exactly those of `type.sec_approval`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalFields {
    /// The approval.
    pub approval_id: ApprovalId,
    /// The request it approves.
    pub request_id: RequestId,
    /// The SHA-256 of that request's canonical envelope without its approval field; the
    /// canonical form is TASK-007's (`core/protocol/serialization`).
    pub request_hash: ContentHash,
    /// The device where it was given; it is never transferred.
    pub destination_device_id: DeviceId,
    /// The local user who decided.
    pub decided_by: UserId,
    /// When the user decided.
    pub decided_at: Timestamp,
    /// After this the approval is void; after `decided_at`.
    pub expires_at: Timestamp,
    /// The epoch at decision time; a later epoch voids it.
    pub permission_epoch: PermissionEpoch,
    /// Set when the approved request executes; a used approval is never accepted again.
    pub used: bool,
}

impl Validate for ApprovalFields {
    fn validate(&self) -> Result<(), DomainError> {
        check_expiry(self.decided_at, Some(self.expires_at))
    }
}

validated_record!(
    /// One local approval of one pending request: single use, bound to that exact request
    /// (RC-17, RC-18).
    Approval,
    ApprovalFields
);

impl Approval {
    /// Fails unless this approval may execute the request `request_id` with canonical hash
    /// `request_hash` on `device` now: it is unused and unexpired, its epoch is `current_epoch`,
    /// and it was given for exactly that request on that device. Expired, replayed or
    /// stale approvals are refused (RC-T08).
    pub fn check_use(
        &self,
        request_id: &RequestId,
        request_hash: &ContentHash,
        device: &DeviceId,
        current_epoch: PermissionEpoch,
        now: Timestamp,
    ) -> Result<(), DomainError> {
        let a = &self.0;
        if a.used {
            return Err(DomainError::new("used", ErrorKind::Stale));
        }
        if &a.request_id != request_id {
            return Err(DomainError::new("request_id", ErrorKind::Mismatch));
        }
        if &a.request_hash != request_hash {
            return Err(DomainError::new("request_hash", ErrorKind::Mismatch));
        }
        if &a.destination_device_id != device {
            return Err(DomainError::new(
                "destination_device_id",
                ErrorKind::Mismatch,
            ));
        }
        if a.permission_epoch != current_epoch {
            return Err(DomainError::new("permission_epoch", ErrorKind::Stale));
        }
        if now >= a.expires_at {
            return Err(DomainError::new("expires_at", ErrorKind::Expired));
        }
        Ok(())
    }

    /// The approval marked as used, which it can be only once.
    pub fn mark_used(&self) -> Result<Self, DomainError> {
        if self.0.used {
            return Err(DomainError::new("used", ErrorKind::Stale));
        }
        Self::try_from(ApprovalFields {
            used: true,
            ..self.0.clone()
        })
    }
}

impl Record for Approval {
    const SCHEMA: &'static str = "nexees.authority.approval";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Versioned;

    fn workspace(id: &str) -> ScopeEntry {
        ScopeEntry::Workspace {
            workspace_id: WorkspaceId::new(id).unwrap(),
        }
    }

    fn remote_grant() -> RemoteGrantFields {
        RemoteGrantFields {
            grant_id: GrantId::new("g1").unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            peer_device_id: DeviceId::new("phone").unwrap(),
            peer_user_id: UserId::new("u1").unwrap(),
            direction: Direction::AndroidToDesktop,
            capability: GrantCapability::new("agent_control").unwrap(),
            scope: vec![workspace("arch-dock")],
            permission_epoch: PermissionEpoch::new(3),
            granted_by: UserId::new("u1").unwrap(),
            granted_at: Timestamp::from_unix_millis(100),
            expires_at: None,
            revoked: false,
        }
    }

    #[test]
    fn grants_name_their_narrow_targets_and_never_their_own_device_as_peer() {
        assert!(RemoteGrant::try_from(remote_grant()).is_ok());
        let mut empty = remote_grant();
        empty.scope.clear();
        assert_eq!(
            RemoteGrant::try_from(empty).unwrap_err(),
            DomainError::new("scope", ErrorKind::Empty)
        );
        let mut twice = remote_grant();
        twice.scope.push(workspace("arch-dock"));
        assert_eq!(
            RemoteGrant::try_from(twice).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let mut itself = remote_grant();
        itself.peer_device_id = itself.destination_device_id.clone();
        assert_eq!(
            RemoteGrant::try_from(itself).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let mut backwards = remote_grant();
        backwards.expires_at = Some(Timestamp::from_unix_millis(100));
        assert_eq!(
            RemoteGrant::try_from(backwards).unwrap_err().kind,
            ErrorKind::OutOfRange
        );
    }

    #[test]
    fn local_grants_need_a_scope_too() {
        let fields = PermissionGrantFields {
            grant_id: GrantId::new("g2").unwrap(),
            principal: Principal::AgentSession {
                session_id: AgentSessionId::new("s1").unwrap(),
            },
            action: ActionId::new("file_write").unwrap(),
            scope: vec![ScopeEntry::Path {
                workspace_id: WorkspaceId::new("arch-dock").unwrap(),
                path: RelativePath::new("src").unwrap(),
            }],
            permission_epoch: PermissionEpoch::new(1),
            granted_by: UserId::new("u1").unwrap(),
            granted_at: Timestamp::from_unix_millis(1),
            expires_at: Some(Timestamp::from_unix_millis(2)),
            revoked: false,
        };
        assert!(PermissionGrant::try_from(fields.clone()).is_ok());
        assert_eq!(
            PermissionGrant::try_from(PermissionGrantFields {
                scope: vec![],
                ..fields
            })
            .unwrap_err()
            .kind,
            ErrorKind::Empty
        );
    }

    #[test]
    fn authority_records_reject_unknown_and_malformed_authority_fields() {
        let mut value =
            serde_json::to_value(RemoteGrant::try_from(remote_grant()).unwrap()).unwrap();
        value["admin"] = serde_json::json!(true);
        assert!(serde_json::from_value::<RemoteGrant>(value.clone()).is_err());
        value.as_object_mut().unwrap().remove("admin");
        value["capability"] = serde_json::json!("Agent Control");
        assert!(serde_json::from_value::<RemoteGrant>(value.clone()).is_err());
        value["capability"] = serde_json::json!("agent_control");
        value["direction"] = serde_json::json!("both");
        assert!(serde_json::from_value::<RemoteGrant>(value.clone()).is_err());
        value["direction"] = serde_json::json!("android_to_desktop");
        value["scope"] =
            serde_json::json!([{"workspace": {"workspace_id": "arch-dock", "recursive": true}}]);
        assert!(serde_json::from_value::<RemoteGrant>(value).is_err());
    }

    fn approval() -> Approval {
        Approval::try_from(ApprovalFields {
            approval_id: ApprovalId::new("a1").unwrap(),
            request_id: RequestId::new("q1").unwrap(),
            request_hash: ContentHash::new("e".repeat(64)).unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            decided_by: UserId::new("u1").unwrap(),
            decided_at: Timestamp::from_unix_millis(1_000),
            expires_at: Timestamp::from_unix_millis(61_000),
            permission_epoch: PermissionEpoch::new(3),
            used: false,
        })
        .unwrap()
    }

    #[test]
    fn an_approval_executes_exactly_its_request_once_within_its_epoch_and_time() {
        let a = approval();
        let (q, hash, pc) = (
            RequestId::new("q1").unwrap(),
            ContentHash::new("e".repeat(64)).unwrap(),
            DeviceId::new("pc").unwrap(),
        );
        let (epoch, now) = (PermissionEpoch::new(3), Timestamp::from_unix_millis(2_000));
        assert!(a.check_use(&q, &hash, &pc, epoch, now).is_ok());

        let other_hash = ContentHash::new("f".repeat(64)).unwrap();
        assert_eq!(
            a.check_use(&RequestId::new("q2").unwrap(), &hash, &pc, epoch, now)
                .unwrap_err()
                .field,
            "request_id"
        );
        assert_eq!(
            a.check_use(&q, &other_hash, &pc, epoch, now)
                .unwrap_err()
                .field,
            "request_hash"
        );
        assert_eq!(
            a.check_use(&q, &hash, &DeviceId::new("phone").unwrap(), epoch, now)
                .unwrap_err()
                .field,
            "destination_device_id"
        );
        assert_eq!(
            a.check_use(&q, &hash, &pc, PermissionEpoch::new(4), now)
                .unwrap_err()
                .kind,
            ErrorKind::Stale
        );
        assert_eq!(
            a.check_use(&q, &hash, &pc, epoch, Timestamp::from_unix_millis(61_000))
                .unwrap_err()
                .kind,
            ErrorKind::Expired
        );

        let used = a.mark_used().unwrap();
        assert_eq!(
            used.check_use(&q, &hash, &pc, epoch, now).unwrap_err(),
            DomainError::new("used", ErrorKind::Stale)
        );
        assert_eq!(used.mark_used().unwrap_err().kind, ErrorKind::Stale);
    }

    #[test]
    fn authority_records_round_trip_and_never_travel() {
        let approval = approval();
        let text = serde_json::to_string(&Versioned(approval.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<Approval>>(&text)
                .unwrap()
                .0,
            approval
        );
        assert!(!<Approval as Record>::SYNC.travels());
        assert!(!<RemoteGrant as Record>::SYNC.travels());
        assert!(!<PermissionGrant as Record>::SYNC.travels());
    }
}
