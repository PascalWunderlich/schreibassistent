//! Quellenseiten wie in der App erzeugen (P7): Einlesen, Schätzung, KI, Prüfung.
//!   cargo run --example ingest_lauf -- <Vault> <citekey>… [--konzepte] [--neu]
//! Anbieter per `ANBIETER`/`MODELL` (sonst Standard), Budget per `BUDGET` (Tokens).

use schreibassistent_lib::{index, ingest, ki, quellen};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = std::path::PathBuf::from(&args[0]);
    let keys: Vec<&String> = args[1..].iter().filter(|a| !a.starts_with("--")).collect();
    let neu = args.iter().any(|a| a == "--neu");
    let mut settings = ki::einstellungen::KiSettings::default();
    if let Ok(anbieter) = std::env::var("ANBIETER") {
        let anbieter: ki::einstellungen::Anbieter = serde_json::from_value(serde_json::json!(anbieter)).unwrap();
        let modell = std::env::var("MODELL").unwrap_or_else(|_| anbieter.default_model().to_owned());
        settings.aufgaben.insert(ki::einstellungen::Aufgabe::Ingest, ki::einstellungen::Wahl { anbieter, modell });
    }
    let idx = index::Index::build(&root, None).unwrap();
    let outline: Vec<String> = idx
        .chapters()
        .into_iter()
        .filter_map(|f| idx.typ.get(f))
        .flat_map(|f| f.headings.iter().filter(|h| h.level <= 2).map(|h| h.text.clone()))
        .collect();
    let sources = quellen::load(&root).unwrap();
    for key in keys {
        let status = ingest::status(&root, key).unwrap();
        if status.aktuell && status.seite.is_some() && !neu {
            println!("{key}: aktuell, keine KI-Anfrage ({:?})", status.stellen);
            continue;
        }
        let title = sources.iter().find(|s| &s.citekey == key).and_then(|s| s.title.clone()).unwrap_or_default();
        let start = std::time::Instant::now();
        let prepared = match ingest::prepare(&root, key, &title, &outline, &settings, std::env::var("BUDGET").ok().and_then(|b| b.parse().ok()).unwrap_or(ingest::BUDGET_TOKENS)) {
            Ok(p) => p,
            Err(e) => {
                println!("{key}: {e}");
                continue;
            }
        };
        let s = &prepared.schaetzung;
        println!(
            "{key}: {} Seiten ({}), {} von {} Abschnitten, ~{} Tokens → {} ({:.1} s)",
            s.seiten, s.seitenzahlen, s.abschnitte_gewaehlt, s.abschnitte, s.tokens_ein, s.modell, start.elapsed().as_secs_f32()
        );
        let start = std::time::Instant::now();
        match ingest::create_page(&root, &settings, prepared).await {
            Ok(r) => println!("   → {} · {} von {} Stellen bestätigt · Tokens {:?} · {:.1} s", r.seite, r.bestaetigt, r.gesamt, r.tokens, start.elapsed().as_secs_f32()),
            Err(e) => println!("   → Fehler: {e}"),
        }
    }
    if args.iter().any(|a| a == "--konzepte") {
        match ingest::create_concepts(&root, &settings, &outline).await {
            Ok(r) => println!("Konzepte: {:?}, {} Warnungen", r.seiten, r.warnungen),
            Err(e) => println!("Konzepte: {e}"),
        }
    }
}
