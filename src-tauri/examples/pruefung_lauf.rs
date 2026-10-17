//! Feste Prüfungen auf einem Arbeits-Vault laufen lassen und die Hinweise ausgeben (P5).
//!
//!   cargo run --manifest-path src-tauri/Cargo.toml --example pruefung_lauf [Vault]
//! Ohne Angabe: beispiel-arbeit.

use std::path::PathBuf;
use std::time::Instant;

use schreibassistent_lib::index::Index;
use schreibassistent_lib::pruefung::{engine, profil};
use schreibassistent_lib::quellen;
use schreibassistent_lib::satz::Satz;

fn main() {
    let root = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"))
    });
    let mut satz = Satz::open(&root, "Arbeit/main.typ").expect("World");
    if let Err(errors) = satz.compile() {
        println!("Setzen fehlgeschlagen: {errors:?}");
    }
    let start = Instant::now();
    let profil = profil::load(&root).expect("Profil").expect("System/regelprofil.yaml fehlt");
    let sources = quellen::load(&root).expect("Quellen");
    let index = Index::build(&root, None).expect("Index");
    let hinweise = engine::check(&profil, &engine::Kontext { satz: &satz, index: &index, sources: &sources });
    let ms = start.elapsed().as_secs_f64() * 1000.0;
    for h in &hinweise {
        println!(
            "{:<22} {:<40} {:>4}  {}",
            h.regel_id,
            h.datei.as_deref().unwrap_or("–"),
            h.zeile.map_or("".to_owned(), |z| z.to_string()),
            h.text
        );
    }
    println!("{} Hinweise in {ms:.0} ms (ohne Setzen)", hinweise.len());
    if std::env::var_os("SEITE").is_some() {
        let n: usize = std::env::var("SEITE").unwrap().parse().unwrap();
        println!("--- Seite {n} ---\n{}", satz.page_texts()[n]);
    }
}
