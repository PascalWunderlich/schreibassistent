//! KI-Protokoll eines Arbeits-Vaults prüfen (Hash-Kette).
//! `cargo run --example protokoll_pruefen -- ~/Arbeiten/Testarbeit`

fn main() {
    let root = std::env::args().nth(1).expect("Pfad zum Arbeits-Vault angeben");
    match schreibassistent_lib::protokoll::verify(std::path::Path::new(&root)) {
        Ok(check) => match check.fehler {
            None => println!("Protokoll in Ordnung: {} Einträge, Kette lückenlos.", check.eintraege),
            Some((line, reason)) => {
                println!("Protokoll FEHLERHAFT in Zeile {line}: {reason} ({} Einträge davor in Ordnung).", check.eintraege);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
