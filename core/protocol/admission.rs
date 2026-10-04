//! The protocol's part of admitting a remote request on its destination (RC-10, RC-17, SI-12,
//! SI-13; `data.sec_execution_checks`).
//!
//! The destination runs these checks in order, and the first failure decides:
//!
//! - **EC-01:** the protocol version is at least the minimum secure version;
//! - **EC-02:** the request is well formed and bounded, for a registered operation, which
//!   decoding already guarantees;
//! - **EC-03:** the sender is the authenticated channel peer;
//! - **EC-04:** the request is addressed to this device, user and host session;
//! - **EC-05:** its request ID is new, otherwise the recorded outcome is the answer and nothing
//!   runs again;
//! - **EC-06:** it is fresh.
//!
//! An admitted request is claimed in the ledger before anything runs (EC-13). The destination's
//! security checks follow: grants, lock state, capabilities, policy and approvals (EC-07 to EC-09,
//! EC-12, EC-14). They belong to the runtime owners of RC-10 in `docs/security`, among them the
//! authority engine (TASK-022), approvals (TASK-055) and the session security (TASK-070). This
//! module also fixes what "stale" means for expected revisions (EC-10) and how a request's targets
//! must match the bound session (EC-11), so every implementation of those checks agrees.

use nexees_domain::errors::{ErrorKind, Outcome};
use nexees_domain::ids::{AgentSessionId, DeviceId, HostSessionId, UserId, WorkspaceId};
use nexees_domain::remote_request::ExpectedRevision;
use nexees_domain::requests::{Duplicate, LedgerEntry};
use nexees_domain::revision::ContentHash;
use nexees_domain::session::AgentSession;
use nexees_domain::time::Timestamp;

use crate::operations::RemoteRequest;

/// The checks of this module, by their identifiers in `data.sec_execution_checks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// EC-01: protocol version.
    ProtocolVersion,
    /// EC-03: the sender is the channel peer.
    ChannelPeer,
    /// EC-04: the destination is this host.
    Destination,
    /// EC-05: the request ID is new.
    Replay,
    /// EC-06: the request is fresh.
    Freshness,
    /// EC-10: the expected revisions are current.
    Revisions,
    /// EC-11: the targets match the bound session.
    Binding,
}

impl Check {
    /// The check's identifier, as audit records name it.
    pub const fn id(self) -> &'static str {
        match self {
            Self::ProtocolVersion => "EC-01",
            Self::ChannelPeer => "EC-03",
            Self::Destination => "EC-04",
            Self::Replay => "EC-05",
            Self::Freshness => "EC-06",
            Self::Revisions => "EC-10",
            Self::Binding => "EC-11",
        }
    }
}

/// This host as a destination: the triple a request must name exactly (EC-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostIdentity {
    /// This device.
    pub device_id: DeviceId,
    /// Its signed-in user.
    pub user_id: UserId,
    /// This host session: the OS login session on Desktop, the app instance on Android.
    pub session_id: HostSessionId,
}

/// The result of the protocol's admission checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admission {
    /// Passed EC-01 to EC-06. Store this entry in the ledger before anything runs (EC-13), then
    /// run the destination's security checks.
    Admit(LedgerEntry),
    /// A request this ledger already holds: answer with this outcome and run nothing (EC-05).
    Duplicate(Outcome),
    /// Refused by `check`; answer with `outcome` and audit it.
    Refuse {
        /// The first check that failed.
        check: Check,
        /// The outcome to acknowledge.
        outcome: Outcome,
    },
}

/// Runs EC-01 and EC-03 to EC-06 for `request`, received at `received_at` over a channel whose
/// authenticated peer is `channel_peer`, at `now`. `request_hash` is the SHA-256 of its canonical
/// form, and `ledger` is this host's entry for the request's ID, if it has one.
#[allow(clippy::too_many_arguments)]
pub fn admit(
    request: &RemoteRequest,
    request_hash: ContentHash,
    host: &HostIdentity,
    channel_peer: &DeviceId,
    ledger: Option<&LedgerEntry>,
    received_at: Timestamp,
    now: Timestamp,
    minimum_secure: u32,
) -> Admission {
    let header = request.header();
    let refuse = |check, outcome| Admission::Refuse { check, outcome };
    if header.protocol_version().get() < minimum_secure {
        return refuse(Check::ProtocolVersion, Outcome::Unsupported);
    }
    if header.requesting_device_id() != channel_peer {
        return refuse(Check::ChannelPeer, Outcome::Denied);
    }
    if header.destination() != (&host.device_id, &host.user_id, &host.session_id) {
        return refuse(Check::Destination, Outcome::Denied);
    }
    if let Some(entry) = ledger {
        if &entry.fields().request_id != header.request_id() {
            // Not this request's entry: fail closed rather than guess.
            return refuse(Check::Replay, Outcome::Denied);
        }
        return match entry.on_duplicate(&request_hash, channel_peer) {
            Duplicate::Recorded(outcome) => Admission::Duplicate(outcome),
            Duplicate::IdReused => refuse(Check::Replay, Outcome::Denied),
        };
    }
    match header.check_fresh(received_at, now) {
        Err(error) if error.kind == ErrorKind::Expired => {
            return refuse(Check::Freshness, Outcome::Expired);
        }
        Err(_) => return refuse(Check::Freshness, Outcome::Denied),
        Ok(()) => {}
    }
    Admission::Admit(LedgerEntry::admit(
        header.request_id().clone(),
        request_hash,
        channel_peer.clone(),
        header.operation().clone(),
        now,
    ))
}

/// The first expected revision that is not current, which makes the request stale (EC-10).
/// `is_current` answers for one revision on this host: a workspace's content revision, a
/// specification revision, a session's generation or the policy revision. Every expected revision
/// must be current; there is no partial match.
pub fn first_stale(
    expected: &[ExpectedRevision],
    is_current: impl Fn(&ExpectedRevision) -> bool,
) -> Option<&ExpectedRevision> {
    expected.iter().find(|revision| !is_current(revision))
}

/// Fails with [`Check::Binding`] unless an agent request's explicit targets are those of the
/// session it names, as bound on this host (EC-11). Nothing is ever taken from focus.
pub fn check_binding(
    workspace_id: Option<&WorkspaceId>,
    agent_id: Option<&AgentSessionId>,
    session: &AgentSession,
) -> Result<(), Check> {
    let bound = session.fields();
    if agent_id == Some(&bound.session_id) && workspace_id == Some(&bound.workspace_id) {
        Ok(())
    } else {
        Err(Check::Binding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::tests::{at, envelope, file_write, hash, host, workspace_revision};
    use crate::operations::{AgentPause, FileWrite};
    use crate::version_negotiation::MIN_SECURE_VERSION;
    use nexees_domain::ids::{ModelId, OperationId, ProviderId, RequestId};
    use nexees_domain::lcl::SpecificationMode;
    use nexees_domain::model_capabilities::ProviderModel;
    use nexees_domain::remote_request::{RequestEnvelope, RequestEnvelopeFields};
    use nexees_domain::revision::{Generation, RevisionId, WorkspaceRevision};
    use nexees_domain::session::{AgentSessionFields, AutonomyMode};

    fn check(
        request: &RemoteRequest,
        peer: &str,
        ledger: Option<&LedgerEntry>,
        now: u64,
    ) -> Admission {
        admit(
            request,
            hash('a'),
            &host(),
            &DeviceId::new(peer).unwrap(),
            ledger,
            at(now),
            at(now),
            MIN_SECURE_VERSION,
        )
    }

    fn refused(check: Check, outcome: Outcome) -> Admission {
        Admission::Refuse { check, outcome }
    }

    fn with(change: impl FnOnce(&mut RequestEnvelopeFields<FileWrite>)) -> RemoteRequest {
        let RemoteRequest::FileWrite(request) = file_write("q1") else {
            unreachable!()
        };
        let mut fields = request.into_fields();
        change(&mut fields);
        RemoteRequest::FileWrite(RequestEnvelope::try_from(fields).unwrap())
    }

    #[test]
    fn a_valid_request_is_admitted_and_claimed_before_it_runs() {
        match check(&file_write("q1"), "phone", None, 2_000) {
            Admission::Admit(entry) => {
                assert_eq!(entry.fields().state, Outcome::Accepted);
                assert_eq!(entry.fields().request_id, RequestId::new("q1").unwrap());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rc_t08_wrong_targets_peers_versions_replays_and_stale_requests_are_refused() {
        let wrong_device = with(|f| f.destination_device_id = DeviceId::new("other-pc").unwrap());
        let wrong_user = with(|f| f.destination_user_id = UserId::new("u2").unwrap());
        let wrong_session =
            with(|f| f.destination_session_id = HostSessionId::new("login-9").unwrap());
        for (request, peer, expected) in [
            (
                &wrong_device,
                "phone",
                refused(Check::Destination, Outcome::Denied),
            ),
            (
                &wrong_user,
                "phone",
                refused(Check::Destination, Outcome::Denied),
            ),
            (
                &wrong_session,
                "phone",
                refused(Check::Destination, Outcome::Denied),
            ),
            // The payload claims to come from the phone, but the channel's peer is the tablet.
            (
                &file_write("q1"),
                "tablet",
                refused(Check::ChannelPeer, Outcome::Denied),
            ),
        ] {
            assert_eq!(check(request, peer, None, 2_000), expected);
        }
        // A destination whose minimum secure version is 2 refuses a version 1 request.
        assert_eq!(
            admit(
                &file_write("q1"),
                hash('a'),
                &host(),
                &DeviceId::new("phone").unwrap(),
                None,
                at(2_000),
                at(2_000),
                2
            ),
            refused(Check::ProtocolVersion, Outcome::Unsupported)
        );
        // Expired by the destination's clock.
        assert_eq!(
            check(&file_write("q1"), "phone", None, 61_000),
            refused(Check::Freshness, Outcome::Expired)
        );
        // Issued one second ahead of the destination's clock, within the skew allowance: admitted.
        assert_eq!(
            admit(
                &file_write("q1"),
                hash('a'),
                &host(),
                &DeviceId::new("phone").unwrap(),
                None,
                at(0),
                at(0),
                MIN_SECURE_VERSION
            ),
            Admission::Admit(LedgerEntry::admit(
                RequestId::new("q1").unwrap(),
                hash('a'),
                DeviceId::new("phone").unwrap(),
                OperationId::new("file_write").unwrap(),
                at(0),
            ))
        );
        // Issued far ahead of the destination's clock: refused.
        let future = with(|f| {
            f.issued_at = at(500_000);
            f.expires_at = at(560_000);
        });
        assert_eq!(
            check(&future, "phone", None, 1_000),
            refused(Check::Freshness, Outcome::Denied)
        );
    }

    #[test]
    fn a_replay_gets_the_recorded_outcome_and_a_reused_id_is_refused() {
        let entry = match check(&file_write("q1"), "phone", None, 2_000) {
            Admission::Admit(entry) => entry.record(Outcome::Denied, at(2_001)).unwrap(),
            other => panic!("{other:?}"),
        };
        assert_eq!(
            check(&file_write("q1"), "phone", Some(&entry), 3_000),
            Admission::Duplicate(Outcome::Denied)
        );
        let reused = admit(
            &file_write("q1"),
            hash('b'),
            &host(),
            &DeviceId::new("phone").unwrap(),
            Some(&entry),
            at(3_000),
            at(3_000),
            MIN_SECURE_VERSION,
        );
        assert_eq!(reused, refused(Check::Replay, Outcome::Denied));
        // Another request's entry is never taken for this one.
        assert_eq!(
            check(&file_write("q2"), "phone", Some(&entry), 3_000),
            refused(Check::Replay, Outcome::Denied)
        );
    }

    #[test]
    fn a_request_for_a_stale_generation_or_revision_is_stale() {
        let expected = [
            workspace_revision("w", "r1"),
            ExpectedRevision::Generation(Generation::FIRST),
        ];
        // The session has moved to generation 2 by a model switch: the request is stale.
        let current_generation = Generation::FIRST.next().unwrap();
        let stale = first_stale(&expected, |r| match r {
            ExpectedRevision::Generation(g) => *g == current_generation,
            _ => true,
        });
        assert_eq!(
            stale,
            Some(&ExpectedRevision::Generation(Generation::FIRST))
        );
        assert_eq!(first_stale(&expected, |_| true), None);
    }

    #[test]
    fn agent_requests_must_name_the_bound_session_and_its_workspace() {
        let pause = RequestEnvelope::try_from(envelope(
            "q3",
            AgentPause {},
            Some("w"),
            Some("s1"),
            vec![ExpectedRevision::Generation(Generation::FIRST)],
        ))
        .unwrap();
        let session = AgentSession::try_from(AgentSessionFields {
            session_id: AgentSessionId::new("s1").unwrap(),
            workspace_id: WorkspaceId::new("w").unwrap(),
            execution_device_id: DeviceId::new("pc").unwrap(),
            checkout: WorkspaceRevision {
                workspace_id: WorkspaceId::new("w").unwrap(),
                revision: RevisionId::new("r1").unwrap(),
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
        .unwrap();
        let f = pause.fields();
        assert_eq!(
            check_binding(f.workspace_id.as_ref(), f.agent_id.as_ref(), &session),
            Ok(())
        );
        let elsewhere = WorkspaceId::new("lcl-next").unwrap();
        assert_eq!(
            check_binding(Some(&elsewhere), f.agent_id.as_ref(), &session),
            Err(Check::Binding)
        );
        assert_eq!(
            check_binding(f.workspace_id.as_ref(), None, &session),
            Err(Check::Binding)
        );
        assert_eq!(Check::Binding.id(), "EC-11");
    }
}
