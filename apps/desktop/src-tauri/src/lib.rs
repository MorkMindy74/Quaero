//! Tauri desktop shell for Quaero. This crate depends on `quaero-core` and only
//! maps IPC commands onto pure core logic (ADR-0011); it holds no domain logic.

mod commands;
mod local_model;
mod ollama;
mod store;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping::ping,
            commands::workspace::create_workspace,
            commands::workspace::open_workspace,
            commands::workspace::search_workspaces,
            commands::workspace::import_document,
            commands::workspace::add_excerpt,
            commands::workspace::add_citation,
            commands::workspace::export_markdown,
            commands::workspace::set_source_text,
            commands::workspace::get_source_text,
            commands::workspace::update_excerpt,
            commands::workspace::delete_excerpt,
            commands::workspace::update_citation,
            commands::workspace::delete_citation,
            commands::chat::chat_send,
            commands::chat::chat_provider_kind,
            commands::evidence::propose_evidence,
            commands::evidence::accept_evidence_candidate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Quaero");
}
