//! Points in time.

use serde::{Deserialize, Serialize};

/// A point in time: milliseconds since the Unix epoch, in UTC, read from the clock of the device
/// that recorded it.
///
/// Times from two devices differ by their clock skew. Comparisons across devices go through the
/// protocol's skew allowance (`binding.sec_clock_skew_allowance`); this type does not hide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// The time `millis` milliseconds after the Unix epoch.
    pub const fn from_unix_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Milliseconds since the Unix epoch.
    pub const fn unix_millis(self) -> u64 {
        self.0
    }

    /// This time plus `millis` milliseconds, saturating at the largest representable time.
    pub const fn saturating_add_millis(self, millis: u64) -> Self {
        Self(self.0.saturating_add(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_serialize_as_plain_milliseconds() {
        let t = Timestamp::from_unix_millis(1_791_100_000_000);
        assert_eq!(serde_json::to_string(&t).unwrap(), "1791100000000");
        assert_eq!(
            serde_json::from_str::<Timestamp>("1791100000000").unwrap(),
            t
        );
        assert!(serde_json::from_str::<Timestamp>("-1").is_err());
    }

    #[test]
    fn adding_saturates_instead_of_wrapping() {
        let end = Timestamp::from_unix_millis(u64::MAX - 1).saturating_add_millis(10);
        assert_eq!(end.unix_millis(), u64::MAX);
    }
}
