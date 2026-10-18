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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(projekt::commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            projekt::commands::projekt_last,
            projekt::commands::projekt_open,
            projekt::commands::projekt_git_init,
            projekt::commands::projekt_tree,
            projekt::commands::projekt_read,
            projekt::commands::projekt_write,
            projekt::commands::projekt_new_note,
            projekt::commands::projekt_new_chapter,
            projekt::commands::projekt_export_dir,
            projekt::commands::projekt_log,
            satz::commands::satz_paragraph_at,
            satz::commands::satz_paragraph_at_offset,
            satz::commands::satz_replace,
            satz::commands::satz_draft,
            satz::commands::satz_compile,
            satz::commands::satz_all_pages,
            satz::commands::satz_font,
            satz::commands::satz_first_page,
            satz::commands::satz_export_pdf,
            index::commands::index_chapters,
            index::commands::index_chapter,
            index::commands::index_sources,
            quellen::commands::quellen_zotero,
            pruefung::commands::pruefung_run,
            pruefung::commands::pruefung_rechtschreibung,
            lsp::commands::lsp_completion,
            lsp::commands::lsp_hover,
            lsp::commands::lsp_definition,
            lsp::commands::lsp_field_diagnostics,
            lsp::commands::lsp_error_count,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                projekt::commands::on_exit(app);
            }
        });
}
