//! Chat contract (#7). Pure and Tauri-free (ADR-0011).
//!
//! A [`ChatProvider`] turns a [`ChatRequest`] into a [`ChatReply`]. The only #7
//! implementation is [`StubProvider`]: deterministic, **offline**, with no
//! network, no API keys, and no state — it validates the architectural flow
//! (UI → IPC → Rust → provider) without a real LLM.
//!
//! Replies are explicitly **ungrounded**: `grounded` is always `false` and they
//! carry no citations. The UI marks them as exploratory and not a legal opinion,
//! so the anti-hallucination principle (ADR-0007) is preserved — real grounded
//! Risposte with Citazioni to Estratti come later (#8).

use serde::{Deserialize, Serialize};

/// Maximum accepted prompt length, in characters.
pub const MAX_PROMPT_CHARS: usize = 8_000;

/// A single chat-turn request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatRequest {
    pub prompt: String,
}

/// A chat reply. `grounded` is always `false` in #7 (no citations / Evidence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReply {
    pub reply: String,
    /// Always `false` in #7: the answer carries no citations.
    pub grounded: bool,
}

/// Why a chat request could not be handled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatError {
    /// The prompt was empty (after trimming).
    EmptyPrompt,
    /// The prompt exceeded [`MAX_PROMPT_CHARS`].
    PromptTooLong { limit: usize, actual: usize },
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatError::EmptyPrompt => write!(f, "empty prompt"),
            ChatError::PromptTooLong { limit, actual } => {
                write!(f, "prompt too long: {actual} chars (limit {limit})")
            }
        }
    }
}

impl std::error::Error for ChatError {}

/// Turns a request into a reply. Pure: no I/O, no network, no secrets.
pub trait ChatProvider {
    fn respond(&self, request: &ChatRequest) -> Result<ChatReply, ChatError>;
}

/// Deterministic, offline stub. Same input → same output. No network, no state,
/// no secrets, no file access. Echoes the (trimmed) prompt inside a fixed,
/// clearly-ungrounded scaffold.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubProvider;

impl ChatProvider for StubProvider {
    fn respond(&self, request: &ChatRequest) -> Result<ChatReply, ChatError> {
        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            return Err(ChatError::EmptyPrompt);
        }
        let count = prompt.chars().count();
        if count > MAX_PROMPT_CHARS {
            return Err(ChatError::PromptTooLong {
                limit: MAX_PROMPT_CHARS,
                actual: count,
            });
        }
        let reply = format!(
            "[bozza esplorativa] Ho ricevuto: \"{prompt}\". \
             Risposta non verificata e senza citazioni; non costituisce parere legale."
        );
        Ok(ChatReply {
            reply,
            grounded: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(prompt: &str) -> ChatRequest {
        ChatRequest {
            prompt: prompt.to_string(),
        }
    }

    #[test]
    fn stub_is_deterministic_and_ungrounded() {
        let a = StubProvider.respond(&req("ciao")).unwrap();
        let b = StubProvider.respond(&req("ciao")).unwrap();
        assert_eq!(a, b); // same input → same output
        assert!(!a.grounded);
        // the reply echoes the prompt and carries an explicit disclaimer
        assert!(a.reply.contains("ciao"));
        assert!(a.reply.contains("non verificata"));
        assert!(a.reply.contains("non costituisce parere legale"));
    }

    #[test]
    fn stub_trims_and_rejects_empty_prompt() {
        assert_eq!(
            StubProvider.respond(&req("   ")),
            Err(ChatError::EmptyPrompt)
        );
        assert_eq!(StubProvider.respond(&req("")), Err(ChatError::EmptyPrompt));
    }

    #[test]
    fn stub_rejects_prompt_over_the_cap() {
        let long = "a".repeat(MAX_PROMPT_CHARS + 1);
        assert_eq!(
            StubProvider.respond(&req(&long)),
            Err(ChatError::PromptTooLong {
                limit: MAX_PROMPT_CHARS,
                actual: MAX_PROMPT_CHARS + 1,
            })
        );
        // exactly at the cap is accepted
        let at = "a".repeat(MAX_PROMPT_CHARS);
        assert!(StubProvider.respond(&req(&at)).is_ok());
    }

    #[test]
    fn chat_payloads_round_trip_camelcase() {
        let request = req("ciao");
        let encoded = serde_json::to_string(&request).unwrap();
        assert_eq!(encoded, r#"{"prompt":"ciao"}"#);
        let decoded: ChatRequest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(request, decoded);

        let reply = ChatReply {
            reply: "x".to_string(),
            grounded: false,
        };
        let encoded = serde_json::to_string(&reply).unwrap();
        assert!(encoded.contains("\"grounded\":false"));
        let decoded: ChatReply = serde_json::from_str(&encoded).unwrap();
        assert_eq!(reply, decoded);
    }

    #[test]
    fn chat_request_rejects_unknown_fields() {
        let json = r#"{"prompt":"ciao","extra":true}"#;
        assert!(serde_json::from_str::<ChatRequest>(json).is_err());
    }
}
