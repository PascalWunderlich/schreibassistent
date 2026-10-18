//! Docling-Erstinstallation wie in der App ausprobieren (lädt rund 2 GB, braucht Netz):
//!   cargo run --example docling_installieren -- <Zielordner>

use schreibassistent_lib::ingest::docling;

fn main() {
    let dir = std::env::args().nth(1).expect("Zielordner angeben");
    docling::init(dir.into());
    println!("Vorher: {:?}", docling::status());
    match docling::install(|line| println!("  {line}")) {
        Ok(()) => println!("Fertig: {:?}", docling::status()),
        Err(e) => {
            eprintln!("Fehler: {e}");
            std::process::exit(1);
        }
    }
}
