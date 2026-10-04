//! Bounded, strict encoding of protocol messages (RC-23, TH-12), the canonical form of a request
//! that approvals bind to (`type.sec_approval`), and the encoding of content bytes.
//!
//! Messages are JSON. A message is measured before it is parsed: a frame between hosts holds at
//! most [`MAX_FRAME_BYTES`] and a local IPC message [`MAX_IPC_MESSAGE_BYTES`]
//! (`binding.sec_max_frame`, `binding.sec_max_ipc_message`). Larger input is refused unread.
//! Decoding is strict: every type of the protocol and the domain rejects unknown fields,
//! duplicate fields, unknown variants and invalid values, and nothing follows the message.

use std::fmt;

use nexees_domain::remote_request::{Operation, RequestEnvelope};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde_json::error::Category;

/// Largest frame between paired hosts, in bytes (`binding.sec_max_frame`).
pub const MAX_FRAME_BYTES: usize = 65_536;

/// Largest message between a window and its host over local IPC, in bytes
/// (`binding.sec_max_ipc_message`, PD-LOCAL-IPC).
pub const MAX_IPC_MESSAGE_BYTES: usize = 4_096;

/// Why a message could not be encoded or decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// The message is larger than its channel allows.
    TooLarge {
        /// Its size in bytes.
        size: usize,
        /// The channel's limit in bytes.
        limit: usize,
    },
    /// The message is not a valid message of its family. The text names the kind of fault and
    /// where decoding stopped. It never quotes the input, so it is safe to log.
    Malformed(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { size, limit } => {
                write!(f, "message of {size} bytes exceeds the limit of {limit}")
            }
            Self::Malformed(reason) => write!(f, "malformed message: {reason}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

fn malformed(reason: &str) -> ProtocolError {
    ProtocolError::Malformed(reason.to_owned())
}

/// A decoding error reduced to its kind and position. serde_json's own message can quote the
/// input, such as a string of the wrong type, and that must never reach a log.
fn malformed_json(error: serde_json::Error) -> ProtocolError {
    let kind = match error.classify() {
        Category::Io => "unreadable input",
        Category::Syntax => "invalid JSON",
        Category::Data => "invalid content",
        Category::Eof => "truncated input",
    };
    let (line, column) = (error.line(), error.column());
    ProtocolError::Malformed(format!("{kind} at line {line}, column {column}"))
}

/// Encoding fails only for a value JSON cannot represent, which no protocol type is.
fn unencodable(_: serde_json::Error) -> ProtocolError {
    malformed("the value cannot be encoded as JSON")
}

/// `message` encoded for a channel whose limit is `limit` bytes; refused when larger.
pub fn encode<T: Serialize>(message: &T, limit: usize) -> Result<Vec<u8>, ProtocolError> {
    let bytes = serde_json::to_vec(message).map_err(unencodable)?;
    if bytes.len() > limit {
        return Err(ProtocolError::TooLarge {
            size: bytes.len(),
            limit,
        });
    }
    Ok(bytes)
}

/// The message in `bytes`, refused unread when longer than `limit`, and refused when it is not
/// exactly one valid `T`.
pub fn decode<T: DeserializeOwned>(bytes: &[u8], limit: usize) -> Result<T, ProtocolError> {
    if bytes.len() > limit {
        return Err(ProtocolError::TooLarge {
            size: bytes.len(),
            limit,
        });
    }
    serde_json::from_slice(bytes).map_err(malformed_json)
}

/// The canonical form of a request, which an approval binds to by its SHA-256
/// (`type.sec_approval`): the envelope as compact JSON, object keys sorted at every level, without
/// `approval_id`. Two envelopes have the same canonical form exactly when every other field is the
/// same. The hash is computed by the security component that holds the hashing library (TASK-055,
/// TASK-070).
pub fn canonical_request<A: Operation>(
    request: &RequestEnvelope<A>,
) -> Result<Vec<u8>, ProtocolError> {
    // serde_json's map keeps keys sorted, since this workspace never enables its preserve_order
    // feature; a test below guards that assumption.
    let mut value = serde_json::to_value(request).map_err(unencodable)?;
    if let Value::Object(map) = &mut value {
        map.remove("approval_id");
    }
    serde_json::to_vec(&value).map_err(unencodable)
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Bytes that travel in JSON as standard base64 with padding (RFC 4648 section 4). Decoding is
/// strict: only the standard alphabet, padding only at the end, and unused bits zero, so every
/// byte string has exactly one accepted encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64Bytes(pub Vec<u8>);

impl Base64Bytes {
    /// The bytes in base64.
    pub fn encode(&self) -> String {
        let mut text = String::with_capacity(self.0.len().div_ceil(3) * 4);
        for chunk in self.0.chunks(3) {
            let b = [
                chunk[0],
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            for i in 0..4 {
                if i <= chunk.len() {
                    text.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
                } else {
                    text.push('=');
                }
            }
        }
        text
    }

    /// The bytes that `text` encodes, refused unless it is canonical base64.
    pub fn decode(text: &str) -> Result<Self, ProtocolError> {
        let input = text.as_bytes();
        if !input.len().is_multiple_of(4) {
            return Err(malformed("base64 length is not a multiple of 4"));
        }
        let value = |c: u8| ALPHABET.iter().position(|&a| a == c).map(|p| p as u32);
        let mut bytes = Vec::with_capacity(input.len() / 4 * 3);
        for (index, quad) in input.chunks(4).enumerate() {
            let last = index + 1 == input.len() / 4;
            let padding = quad.iter().rev().take_while(|&&c| c == b'=').count();
            if padding > 2 || (padding > 0 && !last) {
                return Err(malformed("base64 padding is misplaced"));
            }
            let mut n = 0u32;
            for &c in &quad[..4 - padding] {
                n = (n << 6) | value(c).ok_or_else(|| malformed("not a base64 character"))?;
            }
            n <<= 6 * padding as u32;
            let decoded = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
            let keep = 3 - padding;
            if decoded[keep..].iter().any(|&b| b != 0) {
                return Err(malformed("base64 has non-zero unused bits"));
            }
            bytes.extend_from_slice(&decoded[..keep]);
        }
        Ok(Self(bytes))
    }
}

impl Serialize for Base64Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.encode())
    }
}

impl<'de> Deserialize<'de> for Base64Bytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::decode(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{AppLaunch, FileText, FileWrite};
    use crate::version_negotiation::Hello;
    use nexees_domain::ids::{
        AllowlistKey, ApprovalId, DeviceId, HostSessionId, OperationId, RequestId, UserId,
        WorkspaceId,
    };
    use nexees_domain::remote_request::{ExpectedRevision, ProtocolVersion, RequestEnvelopeFields};
    use nexees_domain::revision::{PermissionEpoch, RevisionId, WorkspaceRevision};
    use nexees_domain::time::Timestamp;
    use nexees_domain::workspace::RelativePath;

    fn request<A: Operation>(
        arguments: A,
        workspace: Option<&str>,
        expected_revisions: Vec<ExpectedRevision>,
        approval: Option<&str>,
    ) -> RequestEnvelope<A> {
        RequestEnvelope::try_from(RequestEnvelopeFields {
            protocol_version: ProtocolVersion::new(1).unwrap(),
            request_id: RequestId::new("q1").unwrap(),
            requesting_device_id: DeviceId::new("phone").unwrap(),
            requesting_user_id: UserId::new("u1").unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            destination_user_id: UserId::new("u1").unwrap(),
            destination_session_id: HostSessionId::new("login-2").unwrap(),
            operation: OperationId::new(A::ID).unwrap(),
            arguments,
            workspace_id: workspace.map(|w| WorkspaceId::new(w).unwrap()),
            agent_id: None,
            expected_revisions,
            permission_epoch: PermissionEpoch::new(3),
            issued_at: Timestamp::from_unix_millis(1_000),
            expires_at: Timestamp::from_unix_millis(61_000),
            approval_id: approval.map(|a| ApprovalId::new(a).unwrap()),
        })
        .unwrap()
    }

    fn write_request(text: &str, approval: Option<&str>) -> RequestEnvelope<FileWrite> {
        request(
            FileWrite {
                path: RelativePath::new("notes.md").unwrap(),
                text: FileText::new(text).unwrap(),
            },
            Some("w"),
            vec![ExpectedRevision::Workspace(WorkspaceRevision {
                workspace_id: WorkspaceId::new("w").unwrap(),
                revision: RevisionId::new("r1").unwrap(),
            })],
            approval,
        )
    }

    #[test]
    fn oversized_input_is_refused_before_it_is_parsed() {
        let big = vec![b' '; MAX_IPC_MESSAGE_BYTES + 1];
        assert_eq!(
            decode::<Value>(&big, MAX_IPC_MESSAGE_BYTES),
            Err(ProtocolError::TooLarge {
                size: MAX_IPC_MESSAGE_BYTES + 1,
                limit: MAX_IPC_MESSAGE_BYTES
            })
        );
        let text = "x".repeat(MAX_IPC_MESSAGE_BYTES);
        assert!(matches!(
            encode(&text, MAX_IPC_MESSAGE_BYTES),
            Err(ProtocolError::TooLarge { .. })
        ));
    }

    #[test]
    fn decoding_accepts_exactly_one_valid_message() {
        let hello = br#"{"versions":{"min":1,"max":1}}"#;
        assert!(decode::<Hello>(hello, MAX_FRAME_BYTES).is_ok());
        for bad in [
            &br#"{"versions":{"min":1,"max":1}} {"versions":{"min":1,"max":1}}"#[..],
            br#"{"versions":{"min":1,"max":1},"versions":{"min":1,"max":1}}"#,
            br#"{"versions":{"min":1,"max":1}"#,
            b"\xef\xbb\xbf{\"versions\":{\"min\":1,\"max\":1}}",
        ] {
            assert!(
                decode::<Hello>(bad, MAX_FRAME_BYTES).is_err(),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn a_refusal_says_where_it_stopped_and_never_quotes_the_input() {
        let value = "private-value-17";
        for bad in [
            format!(r#"{{"versions":{{"min":"{value}","max":1}}}}"#),
            format!(r#"{{"versions":{{"min":1,"max":1}},"{value}":1}}"#),
            format!(r#"{{"versions":{{"min":1,"max":1}}}} {value}"#),
            format!(r#"{{"{value}""#),
        ] {
            let error = decode::<Hello>(bad.as_bytes(), MAX_FRAME_BYTES).unwrap_err();
            let text = error.to_string();
            assert!(text.contains(" at line 1, column "), "{text}");
            assert!(!text.contains(value), "{text}");
        }
    }

    #[test]
    fn the_canonical_form_binds_every_field_except_the_approval() {
        let plain = canonical_request(&write_request("hello", None)).unwrap();
        let approved = canonical_request(&write_request("hello", Some("a1"))).unwrap();
        assert_eq!(
            plain, approved,
            "the approval is not part of what it approves"
        );
        let other = canonical_request(&write_request("hello!", None)).unwrap();
        assert_ne!(
            plain, other,
            "any other change gives another canonical form"
        );
        // The exact bytes: compact, with object keys sorted at every level whatever order the
        // types declare them in. Changing them changes what every approval binds to.
        assert_eq!(
            String::from_utf8(plain).unwrap(),
            concat!(
                r#"{"agent_id":null,"arguments":{"path":"notes.md","text":"hello"},"#,
                r#""destination_device_id":"pc","destination_session_id":"login-2","#,
                r#""destination_user_id":"u1","#,
                r#""expected_revisions":[{"workspace":{"revision":"r1","workspace_id":"w"}}],"#,
                r#""expires_at":61000,"issued_at":1000,"operation":"file_write","#,
                r#""permission_epoch":3,"protocol_version":1,"request_id":"q1","#,
                r#""requesting_device_id":"phone","requesting_user_id":"u1","workspace_id":"w"}"#,
            )
        );
    }

    #[test]
    fn base64_round_trips_and_accepts_only_the_canonical_encoding() {
        for (bytes, text) in [
            (&b""[..], ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foobar", "Zm9vYmFy"),
            (&[0xff, 0xfe, 0x00], "//4A"),
        ] {
            assert_eq!(Base64Bytes(bytes.to_vec()).encode(), text);
            assert_eq!(Base64Bytes::decode(text).unwrap().0, bytes);
        }
        for bad in [
            "Zg=", "Zg===", "Z===", "Zh==", "Zm9v=", "Zg==Zg==", "Zm9-", "Zm 9", "Zm9v\n",
        ] {
            assert!(Base64Bytes::decode(bad).is_err(), "{bad:?}");
        }
        assert!(serde_json::from_str::<Base64Bytes>(r#""Zh==""#).is_err());
    }

    #[test]
    fn requests_of_another_operation_cannot_pass_as_this_one() {
        let launch = request(
            AppLaunch {
                app: AllowlistKey::new("hoptodesk").unwrap(),
            },
            None,
            vec![ExpectedRevision::Policy(RevisionId::new("p4").unwrap())],
            None,
        );
        let text = serde_json::to_string(&launch).unwrap();
        assert!(serde_json::from_str::<RequestEnvelope<AppLaunch>>(&text).is_ok());
        assert!(serde_json::from_str::<RequestEnvelope<FileWrite>>(&text).is_err());
    }
}
