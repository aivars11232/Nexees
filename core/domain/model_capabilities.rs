//! Providers and models, and what a model can do (provider_and_agent_model).
//!
//! A model provides reasoning; Nexees provides agency and the authoritative state. What a model
//! can do is kept apart from where a session executes and what that host can run
//! ([`crate::device::HostCapabilities`]). A model listed on one device is not available on
//! another merely by name (provider_and_agent_model v0.2), so the provider catalogue built from
//! these types never travels (ST-PROVIDER-CATALOG). That catalogue, with each provider's
//! reachability, is TASK-041's.

use serde::{Deserialize, Serialize};

use crate::ids::{ModelId, ProviderId};

/// The categories of provider Nexees integrates (provider_and_agent_model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// A subscription-backed integration, where the provider permits programmatic use.
    Subscription,
    /// An ordinary API provider.
    Api,
    /// A local provider, such as Ollama.
    Local,
    /// An OpenAI-compatible endpoint, local or remote.
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    /// An external coding agent through an interoperability protocol.
    ExternalAgent,
}

/// The provider and model a session uses. Changing it never changes where the session executes
/// or what it is bound to (A5, C21).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderModel {
    /// The configured provider.
    pub provider_id: ProviderId,
    /// The model, as the provider names it.
    pub model_id: ModelId,
}

/// What a model, reached through its provider, can do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCapabilities {
    /// The provider's category.
    pub provider_kind: ProviderKind,
    /// Streams its responses.
    pub streaming: bool,
    /// Proposes structured tool calls.
    pub tool_calls: bool,
    /// Reports token usage.
    pub usage_reporting: bool,
    /// Can cancel a request in flight.
    pub cancellation: bool,
    /// Can run the read-only orientation phase with the tool restriction enforced by Nexees.
    /// An external agent that cannot prove the restriction must report `false` (H3, HO-08).
    pub enforced_read_only_tools: bool,
    /// The context window in tokens, when the provider states it.
    pub context_tokens: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_round_trip_and_reject_unknown_claims() {
        let capabilities = ModelCapabilities {
            provider_kind: ProviderKind::ExternalAgent,
            streaming: true,
            tool_calls: true,
            usage_reporting: false,
            cancellation: true,
            enforced_read_only_tools: false,
            context_tokens: None,
        };
        let text = serde_json::to_string(&capabilities).unwrap();
        assert_eq!(
            serde_json::from_str::<ModelCapabilities>(&text).unwrap(),
            capabilities
        );
        let claimed = text.replacen('{', r#"{"execution_host":"pc","#, 1);
        assert!(serde_json::from_str::<ModelCapabilities>(&claimed).is_err());
    }

    #[test]
    fn provider_kinds_are_a_closed_set() {
        assert_eq!(
            serde_json::to_string(&ProviderKind::OpenAiCompatible).unwrap(),
            "\"openai_compatible\""
        );
        assert!(serde_json::from_str::<ProviderKind>("\"plugin\"").is_err());
    }
}
