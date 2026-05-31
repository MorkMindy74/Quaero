//! IPC command for the #7 chat. Thin mapping onto the pure
//! `quaero_core::chat::StubProvider`: deterministic, offline, no network, no
//! API keys, no file access, no persistence. Errors cross the boundary as
//! `String` (no panics).

use quaero_core::chat::{ChatProvider, ChatReply, ChatRequest, StubProvider};

/// IPC command: answer a chat turn with the deterministic stub provider.
#[tauri::command]
pub fn chat_send(request: ChatRequest) -> Result<ChatReply, String> {
    StubProvider.respond(&request).map_err(|e| e.to_string())
}
