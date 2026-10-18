//! LTeX+ auf den Kapiteln eines Vaults laufen lassen (P5), Meldungen und Zeiten ausgeben.
//!   cargo run --manifest-path src-tauri/Cargo.toml --example ltex_lauf [Vault]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use schreibassistent_lib::pruefung::ltex::{LtexSession, find_ltex};

fn main() {
    let root = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit")));
    let mut files = Vec::new();
    for dir in ["Arbeit", "Arbeit/kapitel"] {
        for entry in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "typ") {
                let rel = p.strip_prefix(&root).unwrap().to_string_lossy().into_owned();
                files.push((rel, std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    files.sort();
    let start = Instant::now();
    let session = LtexSession::start(&find_ltex().expect("ltex-ls-plus"), &root).expect("Start");
    let first = session.check(&files, Duration::from_secs(60)).expect("Prüfung");
    let cold = start.elapsed();
    // Zweiter Lauf mit geändertem Text (sonst antwortet LTeX+ aus dem Zwischenspeicher).
    let changed: Vec<(String, String)> = files.iter().map(|(p, t)| (p.clone(), format!("{t}\nNoch ein Satz.\n"))).collect();
    let start = Instant::now();
    let second = session.check(&changed, Duration::from_secs(60)).expect("Prüfung");
    let warm = start.elapsed();
    for h in &first {
        println!("{:<40} {:>4}  {}", h.datei.as_deref().unwrap_or(""), h.zeile.unwrap_or(0), h.text);
    }
    println!("{} Meldungen; Kaltstart {:.1} s, nach Änderung {:.1} s ({} Meldungen)", first.len(), cold.as_secs_f64(), warm.as_secs_f64(), second.len());
}
