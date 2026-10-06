//! Protocol version negotiation (TH-42, STOP-08, RC-23).
//!
//! Every channel opens with a [`Hello`] from each end stating the protocol versions it speaks:
//! between a window and its host, and between two paired hosts. Both ends then compute the same
//! agreed version: the newest one both speak, if it is at least [`MIN_SECURE_VERSION`]. Otherwise
//! the channel is refused. It never falls back to an older or weaker mode. A window and a host that
//! no longer agree after an update stop, as two peers do; neither guesses (STOP-08).
//!
//! The versions so far:
//!
//! 1. The first protocol.
//! 2. Adds the messages by which a window loads and stores its panel layout
//!    ([`LAYOUT_SINCE_VERSION`]). Two ends that agree on version 1 do without them.
//! 3. Adds the messages by which a window lists, creates, opens and closes the workspaces of its
//!    host's device ([`WORKSPACES_SINCE_VERSION`]).

use std::fmt;

use nexees_domain::device::ProtocolVersions;
use nexees_domain::errors::{DomainError, ErrorKind};
use nexees_domain::remote_request::ProtocolVersion;
use serde::{Deserialize, Serialize};

/// The newest protocol version this build speaks.
pub const CURRENT_VERSION: u32 = 3;

/// The first version that has the panel layout messages between a window and its host. An end
/// that agreed on an older version neither sends nor accepts them.
pub const LAYOUT_SINCE_VERSION: u32 = 2;
// This build speaks the version that has them.
const _: () = assert!(LAYOUT_SINCE_VERSION <= CURRENT_VERSION);

/// The first version that has the workspace messages between a window and its host. An end that
/// agreed on an older version neither sends nor accepts them.
pub const WORKSPACES_SINCE_VERSION: u32 = 3;
const _: () = assert!(WORKSPACES_SINCE_VERSION <= CURRENT_VERSION);

/// The oldest version this build accepts. Raising it retires versions whose security checks are
/// weaker; it is never lowered to reach an old peer.
pub const MIN_SECURE_VERSION: u32 = 1;

/// The versions this build speaks.
pub const fn supported() -> ProtocolVersions {
    ProtocolVersions {
        min: MIN_SECURE_VERSION,
        max: CURRENT_VERSION,
    }
}

/// The first message on every channel: the protocol versions the sender speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HelloFields")]
pub struct Hello {
    versions: ProtocolVersions,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HelloFields {
    versions: ProtocolVersions,
}

impl Hello {
    /// A hello for `versions`, which must start at 1 and be ordered.
    pub fn new(versions: ProtocolVersions) -> Result<Self, DomainError> {
        if versions.min == 0 || versions.min > versions.max {
            return Err(DomainError::new("versions", ErrorKind::OutOfRange));
        }
        Ok(Self { versions })
    }

    /// The hello of this build.
    pub fn current() -> Self {
        Self {
            versions: supported(),
        }
    }

    /// The versions the sender speaks.
    pub fn versions(&self) -> ProtocolVersions {
        self.versions
    }
}

impl TryFrom<HelloFields> for Hello {
    type Error = DomainError;

    fn try_from(fields: HelloFields) -> Result<Self, DomainError> {
        Self::new(fields.versions)
    }
}

/// Why two ends cannot talk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mismatch {
    /// They speak no version in common.
    NoCommonVersion,
    /// The newest version in common is below the minimum secure version.
    BelowMinimumSecure,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoCommonVersion => "no protocol version in common",
            Self::BelowMinimumSecure => "no secure protocol version in common",
        })
    }
}

impl std::error::Error for Mismatch {}

/// The version both ends use: the newest one both speak, refused when it does not exist or is
/// below `minimum_secure`. Both ends get the same answer from the same two hellos.
pub fn negotiate(
    local: &Hello,
    peer: &Hello,
    minimum_secure: u32,
) -> Result<ProtocolVersion, Mismatch> {
    let (a, b) = (local.versions, peer.versions);
    let newest = a.max.min(b.max);
    if newest < a.min.max(b.min) {
        return Err(Mismatch::NoCommonVersion);
    }
    if newest < minimum_secure {
        return Err(Mismatch::BelowMinimumSecure);
    }
    ProtocolVersion::new(newest).map_err(|_| Mismatch::NoCommonVersion)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(min: u32, max: u32) -> Hello {
        Hello::new(ProtocolVersions { min, max }).unwrap()
    }

    #[test]
    fn both_ends_agree_on_the_newest_common_version() {
        let agreed = negotiate(&hello(1, 3), &hello(2, 5), 1).unwrap();
        assert_eq!(agreed.get(), 3);
        assert_eq!(negotiate(&hello(2, 5), &hello(1, 3), 1).unwrap(), agreed);
        assert_eq!(
            negotiate(&Hello::current(), &Hello::current(), MIN_SECURE_VERSION)
                .unwrap()
                .get(),
            CURRENT_VERSION
        );
    }

    #[test]
    fn this_build_and_one_of_the_first_protocol_agree_on_version_1() {
        // After an update a new window may meet a host that still runs, or the reverse: they
        // talk, on the older version, and the newer end leaves out what that version lacks.
        let agreed = negotiate(&Hello::current(), &hello(1, 1), MIN_SECURE_VERSION).unwrap();
        assert_eq!(agreed.get(), 1);
        assert!(agreed.get() < LAYOUT_SINCE_VERSION);
        // A window of TASK-012's build keeps its layouts and does without the workspaces.
        let agreed = negotiate(&Hello::current(), &hello(1, 2), MIN_SECURE_VERSION).unwrap();
        assert_eq!(agreed.get(), LAYOUT_SINCE_VERSION);
        assert!(agreed.get() < WORKSPACES_SINCE_VERSION);
    }

    #[test]
    fn a_mismatch_is_refused_never_downgraded() {
        assert_eq!(
            negotiate(&hello(1, 1), &hello(2, 3), 1),
            Err(Mismatch::NoCommonVersion)
        );
        // An old peer whose newest version is below the secure minimum is refused, even though
        // a common version exists.
        assert_eq!(
            negotiate(&hello(1, 4), &hello(1, 2), 3),
            Err(Mismatch::BelowMinimumSecure)
        );
    }

    #[test]
    fn hellos_are_validated_and_strict() {
        assert!(Hello::new(ProtocolVersions { min: 0, max: 1 }).is_err());
        assert!(Hello::new(ProtocolVersions { min: 3, max: 2 }).is_err());
        assert!(serde_json::from_str::<Hello>(r#"{"versions":{"min":2,"max":1}}"#).is_err());
        assert!(
            serde_json::from_str::<Hello>(r#"{"versions":{"min":1,"max":1},"downgrade":true}"#)
                .is_err()
        );
        let text = serde_json::to_string(&Hello::current()).unwrap();
        assert_eq!(text, r#"{"versions":{"min":1,"max":3}}"#);
        assert_eq!(
            serde_json::from_str::<Hello>(&text).unwrap(),
            Hello::current()
        );
    }
}
