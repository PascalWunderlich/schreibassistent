pub mod projekt;
pub mod satz;
pub mod lsp;
pub mod quellen;
pub mod index;
pub mod pruefung;
pub mod ki;
pub mod protokoll;
pub mod ingest;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(satz::commands::SatzState::default())
        .manage(lsp::commands::LspState::default())
        .invoke_handler(tauri::generate_handler![
            satz::commands::satz_open_example,
            satz::commands::satz_paragraph_at,
            satz::commands::satz_replace,
            lsp::commands::lsp_open_example,
            lsp::commands::lsp_change,
            lsp::commands::lsp_scroll_preview,
            lsp::commands::lsp_relative_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
