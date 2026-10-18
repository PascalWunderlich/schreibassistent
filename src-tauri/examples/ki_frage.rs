//! Eine Frage an die KI wie im Assistenten (Einstellungen: Standard, also Ollama), mit
//! Protokoll; danach Export nach Arbeit/anhang/ki-nutzung.typ.
//! `cargo run --example ki_frage -- <Vault> <Kapiteldatei> "<Frage>"`

use schreibassistent_lib::{ki, protokoll};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [root, datei, frage] = &args[..] else {
        eprintln!("Aufruf: ki_frage <Vault> <Kapiteldatei> \"<Frage>\"");
        std::process::exit(2);
    };
    let root = std::path::Path::new(root);
    let settings = ki::einstellungen::KiSettings::default();
    let kontext = ki::KontextWahl::Kapitel { datei: datei.clone() };
    let start = std::time::Instant::now();
    match ki::frage(root, &settings, ki::einstellungen::Aufgabe::Frage, frage, &kontext).await {
        Ok(r) => println!("{} ({}) in {:.1} s, Tokens {:?}:\n{}", r.modell, r.anbieter, start.elapsed().as_secs_f32(), r.tokens, r.text),
        Err(e) => println!("Fehler: {e}"),
    }
    let text = protokoll::export_typst(&protokoll::read_all(root).unwrap());
    std::fs::create_dir_all(root.join("Arbeit/anhang")).unwrap();
    std::fs::write(root.join(protokoll::EXPORT_FILE), &text).unwrap();
    println!("\n{text}");
}
