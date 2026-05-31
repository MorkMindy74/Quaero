//! IPC commands for the chat (#7 stub + #37 local Ollama).
//!
//! Provider selection is opt-in via the `QUAERO_CHAT_PROVIDER` env var; the
//! default is the offline `StubProvider`. The local Ollama provider talks only
//! to localhost (no API key, no cloud). Errors cross the boundary as `String`.

use quaero_core::chat::{ChatProvider, ChatReply, ChatRequest, StubProvider};

use crate::ollama::OllamaLocalProvider;

/// Whether the local Ollama provider is enabled (opt-in; default = stub).
fn use_ollama_local() -> bool {
    std::env::var("QUAERO_CHAT_PROVIDER")
        .map(|v| v.eq_ignore_ascii_case("ollama"))
        .unwrap_or(false)
}

/// IPC command: answer a chat turn. Async so the local provider's HTTP call can
/// await on the Tauri runtime. Default = offline stub; `QUAERO_CHAT_PROVIDER=ollama`
/// switches to the local Ollama provider (localhost only).
#[tauri::command]
pub async fn chat_send(request: ChatRequest) -> Result<ChatReply, String> {
    if use_ollama_local() {
        OllamaLocalProvider::from_env().respond(&request).await
    } else {
        StubProvider.respond(&request).map_err(|e| e.to_string())
    }
}

/// IPC command: which chat provider is active, so the UI can show an honest
/// privacy posture ("stub" offline vs "ollamaLocal"). Returns a config flag, not
/// any user/client data.
#[tauri::command]
pub fn chat_provider_kind() -> String {
    if use_ollama_local() {
        "ollamaLocal".to_string()
    } else {
        "stub".to_string()
    }
}
