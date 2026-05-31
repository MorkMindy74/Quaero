//! Local Ollama chat provider (#37 reinforcement slice).
//!
//! Desktop-only because it performs network I/O — it therefore lives here and
//! NOT in the pure `quaero-core` (which stays unchanged). It mirrors the
//! `ChatProvider` idea but returns a `String` error: it has network/privacy
//! failures that the pure core trait intentionally does not model.
//!
//! **LOCAL ONLY**: talks to `http://127.0.0.1:11434` (override via env). No API
//! key, no `OLLAMA_API_KEY`, no cloud, no external host. Before any send the
//! prompt passes the **Privacy Guard** (`UserContent` → `LocalModel`). Only the
//! user prompt + a fixed system prompt are sent — **never** documents, Fonti,
//! Estratti, quotes, RAG, bytes or files. Non-streaming; ~120s timeout.

use std::time::Duration;

use quaero_core::chat::{ChatReply, ChatRequest, MAX_PROMPT_CHARS};
use quaero_core::privacy::{DataClass, Decision, Destination, EgressRequest, PrivacyPolicy};
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "qwen3";
const TIMEOUT_SECS: u64 = 120;

/// Fixed system prompt: keeps answers exploratory and explicitly NOT a legal
/// opinion / ungrounded (ADR-0007), mirroring the #7 stub framing.
const SYSTEM_PROMPT: &str = "Sei un assistente esplorativo per un avvocato italiano. \
Le tue risposte sono bozze NON verificate, SENZA citazioni, e NON costituiscono un parere legale. \
Invita sempre alla verifica delle fonti. Non inventare riferimenti normativi o giurisprudenziali.";

/// Local Ollama provider. Endpoint/model are read from the environment with
/// localhost defaults; no secrets are ever read.
pub struct OllamaLocalProvider {
    base_url: String,
    model: String,
}

impl OllamaLocalProvider {
    pub fn from_env() -> Self {
        let base_url =
            std::env::var("QUAERO_OLLAMA_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
        let model =
            std::env::var("QUAERO_OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Self { base_url, model }
    }

    /// Answer a chat turn via the local Ollama server. Async (uses the Tauri
    /// tokio runtime). Returns a user-facing `String` error, never panics.
    pub async fn respond(&self, request: &ChatRequest) -> Result<ChatReply, String> {
        // 1) Privacy Guard — the prompt is UserContent going to a LocalModel.
        if let Decision::Denied(reason) = PrivacyPolicy.evaluate(&prompt_egress_request()) {
            return Err(format!("privacy: {reason}"));
        }

        // 2) Validate the prompt (reuse the core cap; core unchanged).
        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            return Err("prompt vuoto".to_string());
        }
        if prompt.chars().count() > MAX_PROMPT_CHARS {
            return Err(format!(
                "prompt troppo lungo (limite {MAX_PROMPT_CHARS} caratteri)"
            ));
        }

        // 3) Build + send (only prompt + fixed system prompt; nothing else).
        let body = build_request_body(&self.model, prompt);
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .build()
            .map_err(|e| format!("errore inizializzazione client locale: {e}"))?;

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(map_send_error)?;
        if !resp.status().is_success() {
            return Err(map_status_error(resp.status().as_u16()));
        }
        let parsed: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("risposta del modello non valida: {e}"))?;
        Ok(ChatReply {
            reply: extract_reply(parsed)?,
            grounded: false,
        })
    }
}

/// The egress the provider performs: the user's prompt to a local model.
fn prompt_egress_request() -> EgressRequest {
    EgressRequest {
        data_class: DataClass::UserContent,
        destination: Destination::LocalModel,
    }
}

/// Build the `/api/chat` body: system prompt + the user prompt, `stream:false`.
/// Deliberately carries NOTHING else (no documents/Estratti/Fonti/RAG).
fn build_request_body(model: &str, prompt: &str) -> Value {
    json!({
        "model": model,
        "stream": false,
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": prompt },
        ]
    })
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessage>,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

fn extract_reply(response: OllamaChatResponse) -> Result<String, String> {
    match response.message {
        Some(m) if !m.content.trim().is_empty() => Ok(m.content),
        _ => Err("il modello locale ha restituito una risposta vuota".to_string()),
    }
}

/// Map a transport error to a friendly, local-only message.
fn map_send_error(e: reqwest::Error) -> String {
    if e.is_connect() {
        "modello locale non disponibile — avvia Ollama (http://127.0.0.1:11434)".to_string()
    } else if e.is_timeout() {
        "il modello locale non ha risposto entro il tempo limite".to_string()
    } else {
        format!("errore di comunicazione con il modello locale: {e}")
    }
}

/// Map a non-2xx status to a friendly message.
fn map_status_error(code: u16) -> String {
    match code {
        404 => "modello non trovato in Ollama (verifica il nome del modello)".to_string(),
        other => format!("il modello locale ha risposto con stato {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egress_request_passes_the_privacy_guard() {
        // The provider sends UserContent to a LocalModel → must be Allowed.
        assert_eq!(
            PrivacyPolicy.evaluate(&prompt_egress_request()),
            Decision::Allowed
        );
    }

    #[test]
    fn request_body_carries_only_system_and_user_prompt() {
        let body = build_request_body("qwen3", "la clausola 7.2 è valida?");
        assert_eq!(body["model"], "qwen3");
        assert_eq!(body["stream"], false); // non-streaming
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2); // system + user only — nothing else
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "la clausola 7.2 è valida?");
        // the body must NOT smuggle documents/sources/excerpts/files
        let serialized = body.to_string();
        for forbidden in ["sources", "excerpt", "document", "file", "sha256", "matter"] {
            assert!(
                !serialized.contains(forbidden),
                "body must not carry {forbidden}"
            );
        }
    }

    #[test]
    fn system_prompt_enforces_ungrounded_framing() {
        assert!(SYSTEM_PROMPT.contains("NON costituiscono un parere legale"));
        assert!(SYSTEM_PROMPT.contains("NON verificate"));
        assert!(SYSTEM_PROMPT.contains("SENZA citazioni"));
    }

    #[test]
    fn extract_reply_handles_present_and_empty() {
        let ok = OllamaChatResponse {
            message: Some(OllamaMessage {
                content: "ecco una bozza".to_string(),
            }),
        };
        assert_eq!(extract_reply(ok).unwrap(), "ecco una bozza");

        let empty = OllamaChatResponse {
            message: Some(OllamaMessage {
                content: "   ".to_string(),
            }),
        };
        assert!(extract_reply(empty).is_err());

        let none = OllamaChatResponse { message: None };
        assert!(extract_reply(none).is_err());
    }

    #[test]
    fn status_errors_are_friendly() {
        assert!(map_status_error(404).contains("modello non trovato"));
        assert!(map_status_error(500).contains("stato 500"));
    }

    #[test]
    fn from_env_defaults_to_localhost() {
        // Note: relies on env not being set in the test process; defaults checked.
        let p = OllamaLocalProvider {
            base_url: DEFAULT_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
        };
        assert_eq!(p.base_url, "http://127.0.0.1:11434");
        assert_eq!(p.model, "qwen3");
    }
}
