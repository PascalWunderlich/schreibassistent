//! Zeitmessung für P2 (Editor-Test): Änderung → Kompilieren → SVG der geänderten Seiten.
//!
//! Aufruf aus dem Repo-Root:
//!   cargo run --manifest-path src-tauri/Cargo.toml --example satz_messung           (Dev-Profil)
//!   cargo run --manifest-path src-tauri/Cargo.toml --example satz_messung --release (Release)

use std::time::Instant;

use schreibassistent_lib::satz::Satz;

const ROUNDS: usize = 30;

struct Scenario {
    name: &'static str,
    file: &'static str,
    anchor: &'static str,
    insert: &'static str,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "Wort einfügen, Kapitel 1 (Anfang)",
        file: "Arbeit/kapitel/01-einleitung.typ",
        anchor: "Eine Monolith-Ablösung",
        insert: " neue",
    },
    Scenario {
        name: "Wort einfügen, Kapitel 3 (Mitte)",
        file: "Arbeit/kapitel/03-umsetzung.typ",
        anchor: "Die Modellierung erfolgt",
        insert: " hier",
    },
    Scenario {
        name: "Wort einfügen, Kapitel 5 (Ende)",
        file: "Arbeit/kapitel/05-zusammenfassung.typ",
        anchor: "== Ausblick\n\n",
        insert: "Wort ",
    },
    Scenario {
        name: "Satz einfügen vor langem Absatz (verschiebt Umbruch)",
        file: "Arbeit/kapitel/04-loesungskonzept.typ",
        anchor: "=== Datenabgleich\n\n",
        insert: "Ein zusätzlicher Satz verschiebt den folgenden Text nach unten. ",
    },
];

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit");
    let profile = if cfg!(debug_assertions) { "dev (Abhängigkeiten opt-level 3)" } else { "release" };
    println!("Profil: {profile}, {ROUNDS} Änderungen je Szenario\n");

    let start = Instant::now();
    let mut satz = Satz::open(root, "Arbeit/main.typ").expect("World");
    let first = satz.compile().expect("Kompilieren");
    println!(
        "Erstes Kompilieren: {:.0} ms gesamt ({:.0} ms Satz, {:.0} ms SVG), {} Seiten\n",
        start.elapsed().as_secs_f64() * 1000.0,
        first.compile_ms,
        first.svg_ms,
        first.page_count
    );

    println!("| Szenario | Median ms | schlechtester ms | Satz ms (Median) | SVG ms (Median) | geänderte Seiten |");
    println!("|---|---|---|---|---|---|");
    for scenario in SCENARIOS {
        let original = satz.world().source_text(scenario.file).expect("Quelltext");
        assert!(original.contains(scenario.anchor), "Anker fehlt: {}", scenario.anchor);
        let mut text = original.clone();
        let mut totals = Vec::new();
        let mut compiles = Vec::new();
        let mut svgs = Vec::new();
        let mut changed = Vec::new();
        for _ in 0..ROUNDS {
            text = text.replacen(scenario.anchor, &format!("{}{}", scenario.anchor, scenario.insert), 1);
            let t = Instant::now();
            satz.world().set_source_text(scenario.file, &text).unwrap();
            let report = satz.compile().expect("Kompilieren");
            totals.push(t.elapsed().as_secs_f64() * 1000.0);
            compiles.push(report.compile_ms);
            svgs.push(report.svg_ms);
            changed.push(report.changed_pages.len());
        }
        satz.world().set_source_text(scenario.file, &original).unwrap();
        satz.compile().unwrap();
        changed.sort();
        println!(
            "| {} | {:.0} | {:.0} | {:.0} | {:.1} | {}–{} |",
            scenario.name,
            median(&mut totals),
            totals.iter().cloned().fold(0.0, f64::max),
            median(&mut compiles),
            median(&mut svgs),
            changed[0],
            changed[changed.len() - 1],
        );
    }
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}
