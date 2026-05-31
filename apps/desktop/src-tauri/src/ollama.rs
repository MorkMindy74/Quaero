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

    /// True iff the configured endpoint passes the fail-closed local-only check.
    /// Used by `chat_provider_kind` so the UI only reports a local model when the
    /// effective endpoint is genuinely loopback.
    pub fn endpoint_is_local(&self) -> bool {
        validate_local_endpoint(&self.base_url).is_ok()
    }

    /// Answer a chat turn via the local Ollama server. Async (uses the Tauri
    /// tokio runtime). Returns a user-facing `String` error, never panics.
    pub async fn respond(&self, request: &ChatRequest) -> Result<ChatReply, String> {
        // 0) Fail-closed local-only enforcement: reject any non-loopback / non-http
        //    endpoint BEFORE the Privacy Guard and before any send. This makes
        //    "local-only" an in-code invariant, not just a default config.
        validate_local_endpoint(&self.base_url)?;

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
            // Fail-closed local-only: a local (possibly untrusted) server could
            // answer with a 3xx `Location: http://evil.example/...`; reqwest's
            // DEFAULT policy would re-send this same body (system + user prompt)
            // off-device. A local provider has no legitimate reason to follow a
            // redirect, not even to another loopback host → disable them entirely.
            // Any 3xx then stays a non-success status and is handled as an error.
            .redirect(reqwest::redirect::Policy::none())
            // Fail-closed local-only #2: reqwest's default client honours ambient
            // HTTP_PROXY/ALL_PROXY; a configured proxy would receive the prompt
            // (system + user) before the loopback endpoint is ever reached, even
            // for a 127.0.0.1 URL. A local provider must never route through a
            // proxy → disable system/env proxy detection entirely.
            .no_proxy()
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

/// Enforce the local-only invariant **in code** (fail-closed). Accepts only an
/// `http` URL whose host is loopback (`127.0.0.1`, `localhost`, `::1`), with no
/// userinfo. Rejects remote hosts, `https`, credentials, and ambiguous URLs such
/// as `http://127.0.0.1@evil.com` (whose real host is `evil.com`).
fn validate_local_endpoint(raw: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw).map_err(|_| format!("endpoint non valido: {raw}"))?;
    if url.scheme() != "http" {
        return Err("endpoint non locale: è consentito solo http su loopback".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("endpoint non valido: credenziali nell'URL non consentite".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "endpoint non valido: host assente".to_string())?;
    if !is_loopback_host(host) {
        return Err(format!(
            "endpoint non locale: host {host} non è loopback (bloccato)"
        ));
    }
    Ok(())
}

/// Loopback iff `localhost` or an IP that `is_loopback()` (handles 127.0.0.0/8
/// and `::1`, with optional IPv6 brackets).
fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let stripped = host.trim_start_matches('[').trim_end_matches(']');
    stripped
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
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
        // Redirects are disabled (local-only): a 3xx is never followed and is
        // surfaced as an error so the prompt never leaves loopback via Location.
        300..=399 => {
            "il modello locale ha tentato un redirect (bloccato: nessun dato esce da loopback)"
                .to_string()
        }
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
        // Any 3xx is reported as a blocked redirect (never followed).
        assert!(map_status_error(307).contains("redirect"));
        assert!(map_status_error(308).contains("redirect"));
        assert!(map_status_error(301).contains("redirect"));
    }

    /// Regression for the Codex critical: a local server returning a 3xx
    /// `Location: http://evil.example/...` must NOT cause the prompt to be
    /// re-sent off-device. With redirects disabled the 307 stays a 307 → error,
    /// and the only connection ever made is to the loopback test server. If the
    /// redirect were followed, reqwest would instead try to reach `evil.example`
    /// (DNS/connect failure), producing a different, non-"redirect" error.
    #[test]
    fn local_307_redirect_to_remote_is_not_followed() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        // One-shot loopback server that replies 307 → remote, then closes.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, peer)) = listener.accept() {
                // The only inbound connection must come from loopback.
                assert!(peer.ip().is_loopback(), "server reached from non-loopback");
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf); // drain (best-effort) the request
                let response = "HTTP/1.1 307 Temporary Redirect\r\n\
                     Location: http://evil.example/leak\r\n\
                     Content-Length: 0\r\n\
                     Connection: close\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        let provider = OllamaLocalProvider {
            base_url: format!("http://127.0.0.1:{port}"),
            model: "qwen3".to_string(),
        };
        let request = ChatRequest {
            prompt: "bozza riservata".to_string(),
        };
        // Use Tauri's runtime (already a dependency) — no new dev-dep.
        let result = tauri::async_runtime::block_on(provider.respond(&request));
        handle.join().ok();

        let err = result.expect_err("a 3xx must surface as an error, not be followed");
        assert!(
            err.contains("redirect"),
            "expected a blocked-redirect error, got: {err}"
        );
    }

    /// Serialises the tests that mutate the process-global proxy env vars so they
    /// never observe each other's `HTTP_PROXY`/`ALL_PROXY`.
    static PROXY_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Regression for the Codex critical: reqwest's default client honours
    /// ambient `HTTP_PROXY`/`ALL_PROXY`, which would route the loopback POST
    /// (system + user prompt) through a non-loopback proxy. With `.no_proxy()`
    /// the request must reach ONLY the validated loopback endpoint and never
    /// touch the proxy — even when both proxy vars are set.
    #[test]
    fn ambient_proxy_is_ignored_request_stays_on_loopback() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let _guard = PROXY_ENV_LOCK.lock().unwrap();

        // Loopback "Ollama" that answers a valid 200 so respond() succeeds.
        let ollama = TcpListener::bind("127.0.0.1:0").expect("bind ollama");
        let ollama_port = ollama.local_addr().unwrap().port();
        let ollama_thread = std::thread::spawn(move || {
            if let Ok((mut s, peer)) = ollama.accept() {
                assert!(peer.ip().is_loopback(), "ollama reached from non-loopback");
                let mut buf = [0u8; 2048];
                let _ = s.read(&mut buf);
                let body = r#"{"message":{"content":"ok locale"}}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(resp.as_bytes());
                let _ = s.flush();
            }
        });

        // Proxy listener: non-blocking so we can assert it got NO connection.
        let proxy = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
        proxy.set_nonblocking(true).unwrap();
        let proxy_addr = proxy.local_addr().unwrap();

        // Point the ambient proxy env at our proxy listener.
        let prev_all = std::env::var("ALL_PROXY").ok();
        let prev_http = std::env::var("HTTP_PROXY").ok();
        std::env::set_var("ALL_PROXY", format!("http://{proxy_addr}"));
        std::env::set_var("HTTP_PROXY", format!("http://{proxy_addr}"));

        let provider = OllamaLocalProvider {
            base_url: format!("http://127.0.0.1:{ollama_port}"),
            model: "qwen3".to_string(),
        };
        let request = ChatRequest {
            prompt: "bozza riservata".to_string(),
        };
        let result = tauri::async_runtime::block_on(provider.respond(&request));

        // Restore the environment before asserting (keep the process clean).
        match prev_all {
            Some(v) => std::env::set_var("ALL_PROXY", v),
            None => std::env::remove_var("ALL_PROXY"),
        }
        match prev_http {
            Some(v) => std::env::set_var("HTTP_PROXY", v),
            None => std::env::remove_var("HTTP_PROXY"),
        }

        let reply = result.expect("respond must reach loopback Ollama, not the proxy");
        assert_eq!(reply.reply, "ok locale");

        // The proxy must never have been contacted: a non-blocking accept with
        // no pending connection returns WouldBlock.
        match proxy.accept() {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Ok(_) => panic!("request was routed through the proxy — local-only bypassed"),
            Err(e) => panic!("unexpected proxy accept error: {e}"),
        }
        ollama_thread.join().ok();
    }

    #[test]
    fn local_endpoints_are_accepted() {
        for ok in [
            "http://127.0.0.1:11434",
            "http://127.0.0.1",
            "http://localhost:11434",
            "http://localhost",
            "http://[::1]:11434",
        ] {
            assert!(validate_local_endpoint(ok).is_ok(), "should accept {ok}");
        }
    }

    #[test]
    fn non_local_endpoints_are_rejected_fail_closed() {
        for bad in [
            "http://remote.example:11434", // remote host
            "http://192.168.1.5:11434",    // LAN, not loopback
            "https://127.0.0.1:11434",     // https not allowed
            "http://user:pass@127.0.0.1",  // userinfo
            "http://127.0.0.1@evil.com",   // ambiguous: real host is evil.com
            "https://ollama.com/api",      // cloud
            "ftp://127.0.0.1",             // wrong scheme
            "not-a-url",                   // unparseable
        ] {
            assert!(validate_local_endpoint(bad).is_err(), "must reject {bad}");
        }
    }

    #[test]
    fn endpoint_is_local_reflects_validation() {
        let local = OllamaLocalProvider {
            base_url: "http://127.0.0.1:11434".to_string(),
            model: "qwen3".to_string(),
        };
        assert!(local.endpoint_is_local());
        let remote = OllamaLocalProvider {
            base_url: "http://evil.com".to_string(),
            model: "qwen3".to_string(),
        };
        assert!(!remote.endpoint_is_local());
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
