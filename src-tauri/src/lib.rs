pub mod menu;
pub mod projekt;
pub mod satz;
pub mod lsp;
pub mod quellen;
pub mod index;
pub mod pruefung;
pub mod ki;
pub mod protokoll;
pub mod ingest;
pub mod werkzeuge;
pub mod vorlage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .menu(menu::build)
        .on_menu_event(|app, event| {
            use tauri::Emitter;
            let _ = app.emit("menu", event.id().0.clone());
        })
        .setup(|app| {
            use tauri::Manager;
            if let Ok(dir) = app.path().resource_dir() {
                werkzeuge::init(dir);
            }
            if let Ok(dir) = app.path().app_data_dir() {
                ingest::docling::init(dir.join("docling"));
            }
            // Vorlage und Pakete für den Satz bereitstellen (wenn sie fehlen); Fehler zeigt der erste Satz.
            match vorlage::install_bundled() {
                Ok(done) if !done.is_empty() => eprintln!("Vorlage installiert: {}", done.join(", ")),
                Ok(_) => {}
                Err(e) => eprintln!("Vorlage nicht installiert: {e}"),
            }
            Ok(())
        })
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
            projekt::commands::projekt_create,
            projekt::commands::projekt_copy_to,
            projekt::commands::projekt_sichern,
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
            quellen::commands::quellen_zotero_laeuft,
            quellen::commands::quellen_bib_uebernehmen,
            projekt::commands::app_ersteinstieg,
            projekt::commands::app_ersteinstieg_beenden,
            pruefung::commands::pruefung_run,
            pruefung::commands::pruefung_rechtschreibung,
            ki::commands::ki_einstellungen,
            ki::commands::ki_einstellungen_speichern,
            ki::commands::ki_schluessel_setzen,
            ki::commands::ki_schluessel_loeschen,
            ki::commands::ki_frage,
            ki::commands::protokoll_pruefen,
            ki::commands::protokoll_liste,
            ki::commands::ki_belegbedarf,
            ki::commands::ki_quelle_aussage,
            ki::commands::ki_roter_faden,
            ki::commands::ki_urteilsregeln,
            ki::commands::ki_hierzu,
            ki::commands::ki_hierzu_einordnen,
            ki::commands::ki_regelprofil_aus_pdf,
            ki::commands::ki_regelprofil_speichern,
            ki::commands::protokoll_export,
            ingest::commands::ingest_status,
            ingest::commands::ingest_schaetzen,
            ingest::commands::ingest_erzeugen,
            ingest::commands::ingest_konzepte,
            ingest::commands::ingest_werkzeug_status,
            ingest::commands::ingest_werkzeug_installieren,
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
