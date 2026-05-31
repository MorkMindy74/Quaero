//! Tauri desktop shell for Quaero. This crate depends on `quaero-core` and only
//! maps IPC commands onto pure core logic (ADR-0011); it holds no domain logic.

mod commands;
mod store;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            commands::workspace::create_workspace,
            commands::workspace::open_workspace,
            commands::workspace::search_workspaces,
            commands::workspace::import_document,
            commands::chat::chat_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Quaero");
}
