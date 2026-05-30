//! Tauri desktop shell for Quaero. This crate depends on `quaero-core` and only
//! maps IPC commands onto pure core logic (ADR-0011); it holds no domain logic.

mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::ping::ping])
        .run(tauri::generate_context!())
        .expect("error while running Quaero");
}
