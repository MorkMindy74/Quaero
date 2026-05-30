use quaero_core::{ping as core_ping, PingRequest, PingResponse};

/// IPC command: thin mapping onto the pure `quaero_core::ping` handler.
#[tauri::command]
pub fn ping(request: PingRequest) -> PingResponse {
    core_ping(request)
}
