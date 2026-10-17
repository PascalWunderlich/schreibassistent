//! Regelprofil `System/regelprofil.yaml` lesen und prüfen (Schema: `regelprofil.schema.json`).

use std::path::Path;

use serde::Deserialize;

pub const PROFILE_FILE: &str = "System/regelprofil.yaml";

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profil {
    pub profil: String,
    pub quelle: String,
    pub bestaetigt: bool,
    pub regeln: Vec<Regel>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Klasse {
    Maschinell,
    Vorlage,
    Urteil,
}

/// Festes Vokabular (Schema `art`); unbekannte Arten scheitern beim Lesen.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Art {
    Seitenumfang,
    Beschriftung,
    Gliederungstiefe,
    Zitierstil,
    QuellenVollstaendig,
    Textbaustein,
    Stilregel,
}

impl Art {
    pub const ALL: [&'static str; 7] = [
        "seitenumfang",
        "beschriftung",
        "gliederungstiefe",
        "zitierstil",
        "quellen_vollstaendig",
        "textbaustein",
        "stilregel",
    ];
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Schwere {
    Fehler,
    #[default]
    Warnung,
    Hinweis,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Abbildung,
    Tabelle,
    Listing,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Oben,
    Unten,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuellenPruefung {
    ZitiertFehlt,
    Ungenutzt,
    Pflichtfelder,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Regel {
    pub id: String,
    pub klasse: Klasse,
    pub art: Option<Art>,
    pub fundstelle: String,
    #[serde(default)]
    pub schwere: Schwere,
    pub text: Option<String>,
    pub gilt_fuer: Option<String>,
    pub bereich: Option<String>,
    pub min: Option<u32>,
    pub max: Option<u32>,
    pub element: Option<Element>,
    pub position: Option<Position>,
    pub referenziert: Option<bool>,
    pub unterabschnitt_allein: Option<bool>,
    pub erlaubt: Option<Vec<String>>,
    pub fussnoten: Option<String>,
    pub pruefe: Option<Vec<QuellenPruefung>>,
    pub wortlaut: Option<String>,
    pub vale: Option<String>,
}

/// Profil lesen. Fehlt die Datei: `Ok(None)` (keine Prüfungen außer Rechtschreibung).
pub fn load(root: &Path) -> Result<Option<Profil>, String> {
    let path = root.join(PROFILE_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else { return Ok(None) };
    parse(&text).map(Some)
}

pub fn parse(text: &str) -> Result<Profil, String> {
    let profil: Profil = serde_norway::from_str(text).map_err(|e| format!("{PROFILE_FILE}: {e}"))?;
    validate(&profil)?;
    Ok(profil)
}

/// Regeln, die das Schema allein nicht ausdrückt.
fn validate(profil: &Profil) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for regel in &profil.regeln {
        let fail = |what: &str| Err(format!("{PROFILE_FILE}: Regel {}: {what}", regel.id));
        if !ids.insert(&regel.id) {
            return fail("id doppelt");
        }
        match (regel.klasse, regel.art) {
            (Klasse::Maschinell, None) => return fail("klasse maschinell braucht eine art"),
            (Klasse::Urteil, _) if regel.text.is_none() => return fail("klasse urteil braucht text"),
            _ => {}
        }
        let needs = |ok: bool, what: &str| if ok { Ok(()) } else { fail(what) };
        match regel.art {
            Some(Art::Seitenumfang) => needs(regel.min.is_some() && regel.max.is_some(), "seitenumfang braucht min und max")?,
            Some(Art::Beschriftung) => needs(regel.element.is_some(), "beschriftung braucht element")?,
            Some(Art::Textbaustein) => needs(regel.wortlaut.is_some(), "textbaustein braucht wortlaut")?,
            Some(Art::Stilregel) => needs(regel.vale.is_some(), "stilregel braucht vale")?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fhdw() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../typst/fhdw-arbeit/regelprofil-fhdw.yaml")).unwrap()
    }

    #[test]
    fn fhdw_profile_is_valid_and_unconfirmed() {
        let profil = parse(&fhdw()).unwrap();
        assert!(!profil.bestaetigt, "erst nach Freigabe durch Pascal");
        assert!(profil.regeln.iter().all(|r| !r.fundstelle.is_empty()), "jede Regel mit Fundstelle");
        assert!(profil.regeln.iter().any(|r| r.art == Some(Art::Seitenumfang)));
    }

    #[test]
    fn unknown_art_is_a_validation_error() {
        let text = fhdw().replacen("art: gliederungstiefe", "art: gliederungstife", 1);
        let error = parse(&text).unwrap_err();
        assert!(error.contains("gliederungstife"), "{error}");
    }

    #[test]
    fn unknown_field_and_missing_parts_are_errors() {
        assert!(parse(&fhdw().replacen("    max: 3\n", "    maxi: 3\n", 1)).is_err());
        let text = "profil: X\nquelle: Y\nbestaetigt: false\nregeln:\n  - id: a\n    klasse: maschinell\n    fundstelle: S. 1\n";
        assert!(parse(text).unwrap_err().contains("braucht eine art"));
    }

    #[test]
    fn vocabulary_matches_schema_file() {
        let schema: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../typst/fhdw-arbeit/regelprofil.schema.json")).unwrap(),
        )
        .unwrap();
        let arten: Vec<&str> =
            schema["$defs"]["regel"]["properties"]["art"]["enum"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(arten, Art::ALL);
    }
}
