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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
