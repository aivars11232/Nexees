//! Remote request records: the destination's ledger and the sender's pending requests
//! (ST-REMOTE-REQUEST, RC-17, SI-13).
//!
//! A destination records every request ID it admits before the effect starts (EC-13). A
//! duplicate gets the recorded outcome and never runs again; a different request under a used ID
//! is refused. After a restart, an effect that may have started is outcome-unknown, and nothing
//! replays it blindly.
//!
//! The sender keeps a pending record until the destination's answer reconciles it. A lost
//! acknowledgment leaves the outcome unknown, never failed. After reconnecting, the sender asks for
//! the recorded outcome by request ID and resends only a request the destination never received,
//! only before its expiry, and only under the same request ID. Neither record ever travels by
//! synchronization. The stores are TASK-008's and the runtime TASK-070's; this module fixes the
//! records and their rules.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind, Outcome};
use crate::ids::{DeviceId, OperationId, RequestId};
use crate::revision::ContentHash;
use crate::schema::{Record, SyncPolicy, Validate, validated_record};
use crate::time::Timestamp;

/// The fields of a [`LedgerEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerEntryFields {
    /// The request.
    pub request_id: RequestId,
    /// The hash of its canonical envelope, which tells a duplicate from a reused ID.
    pub request_hash: ContentHash,
    /// The device that sent it.
    pub peer_device_id: DeviceId,
    /// Its operation.
    pub operation: OperationId,
    /// Where it stands, as the outcome the destination reports.
    pub state: Outcome,
    /// When the state was recorded.
    pub recorded_at: Timestamp,
}

impl Validate for LedgerEntryFields {
    fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

validated_record!(
    /// One request in the destination's ledger, which is authoritative for its outcome.
    LedgerEntry,
    LedgerEntryFields
);

/// What a destination does with a request whose ID its ledger already holds (EC-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duplicate {
    /// The same request again: answer with the recorded outcome and never run it again.
    Recorded(Outcome),
    /// Another request under a used ID: refuse it and leave the ledger as it is.
    IdReused,
}

impl LedgerEntry {
    /// The entry of a request admitted now, claimed before its effect starts (EC-13).
    pub fn admit(
        request_id: RequestId,
        request_hash: ContentHash,
        peer_device_id: DeviceId,
        operation: OperationId,
        now: Timestamp,
    ) -> Self {
        Self(LedgerEntryFields {
            request_id,
            request_hash,
            peer_device_id,
            operation,
            state: Outcome::Accepted,
            recorded_at: now,
        })
    }

    /// How to answer the same request ID arriving again with `request_hash` from `peer`.
    pub fn on_duplicate(&self, request_hash: &ContentHash, peer: &DeviceId) -> Duplicate {
        if &self.0.request_hash == request_hash && &self.0.peer_device_id == peer {
            Duplicate::Recorded(self.0.state)
        } else {
            Duplicate::IdReused
        }
    }

    /// The entry with a new state. An accepted request may run, wait for the user, or end;
    /// one waiting for the user is accepted again only after revalidation, or ends; a running one
    /// completes or becomes outcome-unknown; an unknown outcome is settled only by evidence that
    /// the effect did or did not happen. Completed, denied, unsupported and expired are final.
    pub fn record(&self, state: Outcome, now: Timestamp) -> Result<Self, DomainError> {
        use Outcome::*;
        let allowed = matches!(
            (self.0.state, state),
            (
                Accepted,
                Running
                    | Completed
                    | Denied
                    | Unsupported
                    | NeedsUserAction
                    | Expired
                    | OutcomeUnknown
            ) | (NeedsUserAction, Accepted | Denied | Expired)
                | (Running, Completed | OutcomeUnknown)
                | (OutcomeUnknown, Completed | Denied)
        );
        if !allowed {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        Ok(Self(LedgerEntryFields {
            state,
            recorded_at: now,
            ..self.0.clone()
        }))
    }

    /// The entry as it stands after the destination restarted. An effect that was admitted or
    /// running may or may not have happened, so it is outcome-unknown until reconciled (RC-17).
    pub fn after_restart(&self, now: Timestamp) -> Self {
        match self.0.state {
            Outcome::Accepted | Outcome::Running => Self(LedgerEntryFields {
                state: Outcome::OutcomeUnknown,
                recorded_at: now,
                ..self.0.clone()
            }),
            _ => self.clone(),
        }
    }
}

impl Record for LedgerEntry {
    const SCHEMA: &'static str = "nexees.requests.ledger_entry";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// Where a sender's request stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PendingState {
    /// Built but not sent, for example while disconnected.
    Unsent,
    /// Sent; no answer yet. After a disconnect its outcome is unknown until reconciled.
    Sent,
    /// The destination answered with this outcome, by acknowledgment or reconciliation.
    Answered(Outcome),
    /// Expired before the destination ever received it; never sent again. Final.
    Expired,
}

/// The fields of a [`PendingRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingRequestFields {
    /// The request.
    pub request_id: RequestId,
    /// The hash of its canonical envelope.
    pub request_hash: ContentHash,
    /// Where it was sent.
    pub destination_device_id: DeviceId,
    /// Its operation.
    pub operation: OperationId,
    /// Its expiry, from the envelope.
    pub expires_at: Timestamp,
    /// Where it stands.
    pub state: PendingState,
}

impl Validate for PendingRequestFields {
    fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

validated_record!(
    /// The sender's record of one request, kept until the destination's answer reconciles it.
    PendingRequest,
    PendingRequestFields
);

/// What a sender does with a pending request after reconnecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reconnect {
    /// The destination's answer settles it; nothing is sent.
    Settled(Outcome),
    /// The destination never received it and it has not expired: send the same envelope, under
    /// the same request ID, which the destination's ledger deduplicates.
    Resend,
    /// It expired before the destination received it: drop it with an audit record, never send it.
    Drop,
}

impl PendingRequest {
    /// The request as sent, unless it expired first, in which case it is never sent (RC-17).
    pub fn send(&self, now: Timestamp) -> Result<Self, DomainError> {
        match self.0.state {
            PendingState::Unsent if now >= self.0.expires_at => {
                Ok(self.with(PendingState::Expired))
            }
            PendingState::Unsent => Ok(self.with(PendingState::Sent)),
            _ => Err(DomainError::new("state", ErrorKind::InvalidTransition)),
        }
    }

    /// The request with the destination's acknowledged `outcome`.
    pub fn acknowledge(&self, outcome: Outcome) -> Result<Self, DomainError> {
        match self.0.state {
            PendingState::Sent | PendingState::Answered(_) => {
                Ok(self.with(PendingState::Answered(outcome)))
            }
            _ => Err(DomainError::new("state", ErrorKind::InvalidTransition)),
        }
    }

    /// What to do after reconnecting, given what the destination's ledger records for this
    /// request ID: an outcome, or nothing when it never received it.
    pub fn reconnect(&self, recorded: Option<Outcome>, now: Timestamp) -> (Self, Reconnect) {
        match (recorded, self.0.state) {
            (_, PendingState::Expired) => (self.clone(), Reconnect::Drop),
            (Some(outcome), _) => (
                self.with(PendingState::Answered(outcome)),
                Reconnect::Settled(outcome),
            ),
            (None, PendingState::Answered(outcome)) => (self.clone(), Reconnect::Settled(outcome)),
            (None, _) if now >= self.0.expires_at => {
                (self.with(PendingState::Expired), Reconnect::Drop)
            }
            (None, _) => (self.with(PendingState::Sent), Reconnect::Resend),
        }
    }

    /// Whether the user may be told the request is done. Only a completed answer counts; a sent
    /// request without an answer is not success (RC-12).
    pub fn completed(&self) -> bool {
        self.0.state == PendingState::Answered(Outcome::Completed)
    }

    fn with(&self, state: PendingState) -> Self {
        Self(PendingRequestFields {
            state,
            ..self.0.clone()
        })
    }
}

impl Record for PendingRequest {
    const SCHEMA: &'static str = "nexees.requests.pending";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(c: char) -> ContentHash {
        ContentHash::new(c.to_string().repeat(64)).unwrap()
    }

    fn at(ms: u64) -> Timestamp {
        Timestamp::from_unix_millis(ms)
    }

    fn entry() -> LedgerEntry {
        LedgerEntry::admit(
            RequestId::new("q1").unwrap(),
            hash('a'),
            DeviceId::new("phone").unwrap(),
            OperationId::new("app_launch").unwrap(),
            at(10),
        )
    }

    fn pending(state: PendingState) -> PendingRequest {
        PendingRequest::try_from(PendingRequestFields {
            request_id: RequestId::new("q1").unwrap(),
            request_hash: hash('a'),
            destination_device_id: DeviceId::new("pc").unwrap(),
            operation: OperationId::new("file_write").unwrap(),
            expires_at: at(1_000),
            state,
        })
        .unwrap()
    }

    #[test]
    fn a_duplicate_gets_the_recorded_outcome_and_never_runs_again() {
        let done = entry()
            .record(Outcome::Running, at(11))
            .and_then(|e| e.record(Outcome::Completed, at(12)))
            .unwrap();
        let phone = DeviceId::new("phone").unwrap();
        assert_eq!(
            done.on_duplicate(&hash('a'), &phone),
            Duplicate::Recorded(Outcome::Completed)
        );
        // Another request, or another device, under the same ID is refused.
        assert_eq!(done.on_duplicate(&hash('b'), &phone), Duplicate::IdReused);
        assert_eq!(
            done.on_duplicate(&hash('a'), &DeviceId::new("tablet").unwrap()),
            Duplicate::IdReused
        );
    }

    #[test]
    fn a_restart_turns_a_possibly_started_effect_into_an_unknown_outcome() {
        let running = entry().record(Outcome::Running, at(11)).unwrap();
        let after = running.after_restart(at(20));
        assert_eq!(after.fields().state, Outcome::OutcomeUnknown);
        // A duplicate after the restart is told the outcome is unknown; it does not run again.
        assert_eq!(
            after.on_duplicate(&hash('a'), &DeviceId::new("phone").unwrap()),
            Duplicate::Recorded(Outcome::OutcomeUnknown)
        );
        let completed = entry().record(Outcome::Completed, at(11)).unwrap();
        assert_eq!(completed.after_restart(at(20)), completed);
    }

    #[test]
    fn ledger_states_change_only_along_the_allowed_paths() {
        let unknown = entry().record(Outcome::OutcomeUnknown, at(11)).unwrap();
        assert_eq!(
            unknown.record(Outcome::Running, at(12)).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
        assert!(unknown.record(Outcome::Completed, at(12)).is_ok());
        let waiting = entry().record(Outcome::NeedsUserAction, at(11)).unwrap();
        assert_eq!(
            waiting.record(Outcome::Running, at(12)).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
        assert!(waiting.record(Outcome::Accepted, at(12)).is_ok());
        let denied = entry().record(Outcome::Denied, at(11)).unwrap();
        assert!(denied.record(Outcome::Accepted, at(12)).is_err());
    }

    #[test]
    fn a_lost_acknowledgment_is_reconciled_by_request_id_without_a_second_execution() {
        let sent = pending(PendingState::Sent);
        assert!(!sent.completed());
        // The destination ran it and recorded the outcome; the sender learns it and sends nothing.
        let (settled, action) = sent.reconnect(Some(Outcome::Completed), at(500));
        assert_eq!(action, Reconnect::Settled(Outcome::Completed));
        assert!(settled.completed());
        // The destination's ledger says unknown: still nothing is resent.
        let (unknown, action) = sent.reconnect(Some(Outcome::OutcomeUnknown), at(500));
        assert_eq!(action, Reconnect::Settled(Outcome::OutcomeUnknown));
        assert!(!unknown.completed());
    }

    #[test]
    fn a_request_the_destination_never_received_is_resent_only_before_its_expiry() {
        let sent = pending(PendingState::Sent);
        assert_eq!(sent.reconnect(None, at(999)).1, Reconnect::Resend);
        let (dropped, action) = sent.reconnect(None, at(1_000));
        assert_eq!(action, Reconnect::Drop);
        assert_eq!(dropped.fields().state, PendingState::Expired);
        // Once expired, nothing revives it, not even a later answer.
        assert_eq!(
            dropped.reconnect(Some(Outcome::Completed), at(1_001)).1,
            Reconnect::Drop
        );
    }

    #[test]
    fn a_request_that_expired_while_disconnected_is_never_sent() {
        let queued = pending(PendingState::Unsent);
        assert_eq!(
            queued.send(at(1_000)).unwrap().fields().state,
            PendingState::Expired
        );
        assert_eq!(
            queued.send(at(999)).unwrap().fields().state,
            PendingState::Sent
        );
        assert_eq!(
            queued.reconnect(None, at(2_000)).1,
            Reconnect::Drop,
            "RC-T10: an expired queued command is not replayed after reconnecting"
        );
    }

    #[test]
    fn request_records_never_travel_and_reject_unknown_fields() {
        assert!(!<LedgerEntry as Record>::SYNC.travels());
        assert!(!<PendingRequest as Record>::SYNC.travels());
        let mut value = serde_json::to_value(entry()).unwrap();
        value["approved_by"] = serde_json::json!("u1");
        assert!(serde_json::from_value::<LedgerEntry>(value).is_err());
    }
}
