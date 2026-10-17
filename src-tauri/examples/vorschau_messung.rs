//! Zeitmessung für P2, Prototyp B: Änderung (didChange) → Tinymist meldet „gesetzt“.
//! Enthält nicht das Zeichnen in der Vorschau (das misst die Oberfläche).
//!
//! Aufruf aus dem Repo-Root:
//!   cargo run --manifest-path src-tauri/Cargo.toml --example vorschau_messung

use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use schreibassistent_lib::lsp::{TinymistSession, find_tinymist};

const ROUNDS: usize = 30;

fn main() {
    let program = find_tinymist().expect("tinymist");
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
    let (tx, rx) = channel();
    let (session, files) = TinymistSession::start(&program, root, "Arbeit/main.typ", move |method, params| {
        if method == "tinymist/compileStatus" {
            let _ = tx.send(params["status"].as_str().unwrap_or_default().to_owned());
        }
    })
    .expect("Tinymist startet");
    // Erste Kompilierung abwarten.
    while rx.recv_timeout(Duration::from_secs(2)).is_ok() {}

    let scenarios = [
        ("Wort einfügen, Kapitel 1 (Anfang)", "Arbeit/kapitel/01-einleitung.typ", "Eine Monolith-Ablösung", " neue"),
        ("Wort einfügen, Kapitel 3 (Mitte)", "Arbeit/kapitel/03-umsetzung.typ", "Die Modellierung erfolgt", " hier"),
        ("Wort einfügen, Kapitel 5 (Ende)", "Arbeit/kapitel/05-zusammenfassung.typ", "== Ausblick\n\n", "Wort "),
        (
            "Satz einfügen vor langem Absatz (verschiebt Umbruch)",
            "Arbeit/kapitel/04-loesungskonzept.typ",
            "=== Datenabgleich\n\n",
            "Ein zusätzlicher Satz verschiebt den folgenden Text nach unten. ",
        ),
    ];
    println!("| Szenario | Median ms | schlechtester ms |");
    println!("|---|---|---|");
    for (name, path, anchor, insert) in scenarios {
        let original = files.iter().find(|f| f.path == path).expect("Datei").text.clone();
        let mut text = original.clone();
        let mut times = Vec::new();
        for _ in 0..ROUNDS {
            text = text.replacen(anchor, &format!("{anchor}{insert}"), 1);
            let start = Instant::now();
            session.change(path, &text).unwrap();
            loop {
                let status = rx.recv_timeout(Duration::from_secs(10)).expect("Status");
                if status != "compiling" {
                    break;
                }
            }
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        session.change(path, &original).unwrap();
        while rx.recv_timeout(Duration::from_millis(500)).is_ok() {}
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("| {name} | {:.0} | {:.0} |", times[ROUNDS / 2], times[ROUNDS - 1]);
    }
}
