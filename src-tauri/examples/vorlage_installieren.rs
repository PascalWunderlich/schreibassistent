//! Vorlage und Typst-Pakete aus `build/werkzeuge` installieren (CI vor `cargo test`, Entwicklung):
//!   cargo run --example vorlage_installieren

fn main() {
    match schreibassistent_lib::vorlage::install_bundled() {
        Ok(done) if done.is_empty() => println!("Alles schon installiert."),
        Ok(done) => println!("Installiert: {}", done.join(", ")),
        Err(e) => {
            eprintln!("Fehler: {e}");
            std::process::exit(1);
        }
    }
}
