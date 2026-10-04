//! Devices: their kinds, the trust between paired devices, and the execution capabilities a
//! host offers (A1, A4, A9, ST-DEVICE-TRUST).
//!
//! Pairing establishes identity only. A [`PeerDevice`] records who a peer is and whether it is
//! still trusted; what it may do is a separate grant on the destination
//! ([`crate::authority::RemoteGrant`]). What a host can execute ([`HostCapabilities`]) is
//! separate from what a model can do ([`crate::model_capabilities`]), and neither is inferred
//! from the other.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{DeviceId, HostCapability, UserId};
use crate::schema::{Record, SyncPolicy, Validate, ensure_unique, validated_record};
use crate::text::Label;
use crate::time::Timestamp;

/// The kind of a device and of the Nexees host on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    /// A desktop computer with the Desktop host.
    Desktop,
    /// A phone or tablet with the Android host.
    Android,
}

/// Whether a paired device is still trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustState {
    /// Paired and trusted.
    Paired,
    /// Revoked; kept for audit, never trusted again under the same pairing.
    Revoked,
}

/// The range of protocol versions a peer advertised when it last paired or reconnected. The
/// version negotiation itself is TASK-007's (`core/protocol/version_negotiation`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolVersions {
    /// The oldest version the peer speaks; at least 1.
    pub min: u32,
    /// The newest version the peer speaks.
    pub max: u32,
}

/// The fields of a [`PeerDevice`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerDeviceFields {
    /// The peer.
    pub device_id: DeviceId,
    /// The user signed in on the peer.
    pub user_id: UserId,
    /// The peer's kind.
    pub kind: DeviceKind,
    /// The name the user gave the peer; never used to identify it (TH-47).
    pub display_name: Label,
    /// Whether it is still trusted.
    pub trust: TrustState,
    /// When it was paired.
    pub paired_at: Timestamp,
    /// When it was revoked; present exactly when it is revoked.
    pub revoked_at: Option<Timestamp>,
    /// The protocol versions it advertised.
    pub protocol_versions: ProtocolVersions,
}

impl Validate for PeerDeviceFields {
    fn validate(&self) -> Result<(), DomainError> {
        match (self.trust, self.revoked_at) {
            (TrustState::Paired, Some(_)) => {
                return Err(DomainError::new("revoked_at", ErrorKind::Unexpected));
            }
            (TrustState::Revoked, None) => {
                return Err(DomainError::new("revoked_at", ErrorKind::Missing));
            }
            (TrustState::Revoked, Some(at)) if at < self.paired_at => {
                return Err(DomainError::new("revoked_at", ErrorKind::OutOfRange));
            }
            _ => {}
        }
        let versions = self.protocol_versions;
        if versions.min == 0 || versions.min > versions.max {
            return Err(DomainError::new("protocol_versions", ErrorKind::OutOfRange));
        }
        Ok(())
    }
}

validated_record!(
    /// A paired device as this device knows it: identity, user, trust and pairing history
    /// (ST-DEVICE-TRUST). It holds no keys (those are secrets, ST-SECRETS) and no authority.
    PeerDevice,
    PeerDeviceFields
);

impl Record for PeerDevice {
    const SCHEMA: &'static str = "nexees.device.peer";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The fields of [`HostCapabilities`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostCapabilitiesFields {
    /// The host's device.
    pub device_id: DeviceId,
    /// The host's kind.
    pub kind: DeviceKind,
    /// What the host can execute, each capability once.
    pub capabilities: Vec<HostCapability>,
    /// When the host last checked them.
    pub observed_at: Timestamp,
}

impl Validate for HostCapabilitiesFields {
    fn validate(&self) -> Result<(), DomainError> {
        ensure_unique(&self.capabilities, "capabilities")
    }
}

validated_record!(
    /// What one execution host can run: tools, toolchains and test backends (A4, A9). A task
    /// that needs a capability its host lacks is blocked, never marked complete (A4).
    HostCapabilities,
    HostCapabilitiesFields
);

impl Record for HostCapabilities {
    const SCHEMA: &'static str = "nexees.device.host_capabilities";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(trust: TrustState, revoked_at: Option<u64>) -> PeerDeviceFields {
        PeerDeviceFields {
            device_id: DeviceId::new("phone-key-1").unwrap(),
            user_id: UserId::new("u1").unwrap(),
            kind: DeviceKind::Android,
            display_name: Label::new("Phone").unwrap(),
            trust,
            paired_at: Timestamp::from_unix_millis(100),
            revoked_at: revoked_at.map(Timestamp::from_unix_millis),
            protocol_versions: ProtocolVersions { min: 1, max: 1 },
        }
    }

    #[test]
    fn revocation_is_recorded_with_its_time() {
        assert!(PeerDevice::try_from(peer(TrustState::Paired, None)).is_ok());
        assert!(PeerDevice::try_from(peer(TrustState::Revoked, Some(200))).is_ok());
        assert_eq!(
            PeerDevice::try_from(peer(TrustState::Revoked, None)).unwrap_err(),
            DomainError::new("revoked_at", ErrorKind::Missing)
        );
        assert_eq!(
            PeerDevice::try_from(peer(TrustState::Paired, Some(200))).unwrap_err(),
            DomainError::new("revoked_at", ErrorKind::Unexpected)
        );
        assert_eq!(
            PeerDevice::try_from(peer(TrustState::Revoked, Some(50)))
                .unwrap_err()
                .kind,
            ErrorKind::OutOfRange
        );
    }

    #[test]
    fn two_peers_may_share_a_name_but_never_an_identity() {
        let a = PeerDevice::try_from(peer(TrustState::Paired, None)).unwrap();
        let mut other = peer(TrustState::Paired, None);
        other.device_id = DeviceId::new("phone-key-2").unwrap();
        let b = PeerDevice::try_from(other).unwrap();
        assert_eq!(a.fields().display_name, b.fields().display_name);
        assert_ne!(a.fields().device_id, b.fields().device_id);
    }

    #[test]
    fn pairing_records_cannot_carry_grants() {
        let mut json =
            serde_json::to_value(PeerDevice::try_from(peer(TrustState::Paired, None)).unwrap())
                .unwrap();
        json["capability"] = serde_json::json!("agent_control");
        assert!(serde_json::from_value::<PeerDevice>(json).is_err());
    }

    #[test]
    fn protocol_ranges_must_be_ordered_and_start_at_one() {
        let mut fields = peer(TrustState::Paired, None);
        fields.protocol_versions = ProtocolVersions { min: 2, max: 1 };
        assert_eq!(
            PeerDevice::try_from(fields.clone()).unwrap_err().kind,
            ErrorKind::OutOfRange
        );
        fields.protocol_versions = ProtocolVersions { min: 0, max: 1 };
        assert_eq!(
            PeerDevice::try_from(fields).unwrap_err().kind,
            ErrorKind::OutOfRange
        );
    }

    #[test]
    fn host_capabilities_list_each_capability_once() {
        let capability = |c| HostCapability::new(c).unwrap();
        let fields = HostCapabilitiesFields {
            device_id: DeviceId::new("phone").unwrap(),
            kind: DeviceKind::Android,
            capabilities: vec![capability("file_tools"), capability("boa_tests")],
            observed_at: Timestamp::from_unix_millis(1),
        };
        assert!(HostCapabilities::try_from(fields.clone()).is_ok());
        let mut twice = fields;
        twice.capabilities.push(capability("boa_tests"));
        assert_eq!(
            HostCapabilities::try_from(twice).unwrap_err().kind,
            ErrorKind::Duplicate
        );
    }
}
