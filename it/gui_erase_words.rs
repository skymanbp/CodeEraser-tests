//! The erase screen's provenance templates held to the console (release
//! 1.10.0, item K3; split from gui_core_words.rs at its line, whose
//! helpers it reads): the GUI writes a plan row's provenance from the
//! row's facts with its own four templates, and nothing else read those
//! beside the core's `prov_*` rows.

use crate::common::{self, tmp};
use crate::gui_core_words::{ce, copy, document, gui};
use serde_json::Value;

/// The erase screen's four provenance templates (`provDead`, `provDoc`,
/// `provLinked`, `provTwin`) are the console's `prov_*` rows: on the
/// erase fixture with linked documentation twins added (erase_e2e's
/// seed: an unreferenced file, a clone twin; the two linked pages: a
/// documentation twin whose candidate's section is linked), each
/// advisory row's GUI sentence, its markers filled with the row's facts
/// (the plan's `from`), appears in the console, in both languages.
#[test]
fn the_erase_provenance_templates_are_the_consoles() {
    let w = gui();
    let dir = tmp("gui-words-provenance");
    crate::erase_e2e::seed(&dir);
    let page = format!(
        "# Page\n\n## Install\n\n{} extra words here\n",
        crate::erase_e2e::PARA
    );
    let files = [
        (
            "links.md",
            "[a](docs/a.md#install) [b](docs/b.md#install)".to_string(),
        ),
        ("docs/a.md", page.clone()),
        ("docs/b.md", page),
    ];
    common::write_all(&dir, &files);
    common::commit_all(&dir, "linked docs");
    let plan = document(&dir, "erase");
    let rows: Vec<&Value> = plan["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|r| r["eraseable"] == false)
        .collect();
    for family in ["deadcode", "docdup", "dedup"] {
        assert!(
            rows.iter().any(|r| r["from"]["family"] == family),
            "an advisory {family} row: {plan}"
        );
    }
    assert!(
        rows.iter().any(|r| r["from"]["linked"].as_u64() > Some(0)),
        "a linked row: {plan}"
    );
    for lang in ["en", "zh"] {
        let text = ce(&dir, lang, &["erase", "."]);
        for r in &rows {
            let line = provenance(&w, lang, &r["from"]);
            assert!(text.contains(&line), "{lang}: {line} in {text}");
        }
    }
}

/// One plan row's provenance in `lang`, the GUI's way (gui/ui/erase.js
/// `provText`): its template with the row's facts in the markers.
fn provenance(w: &Value, lang: &str, f: &Value) -> String {
    let s = |v: &Value| v.as_str().map_or_else(|| v.to_string(), String::from);
    let span = [s(&f["span"][0]), s(&f["span"][1])];
    match f["family"].as_str() {
        Some("deadcode") => {
            let why = s(&w[lang]["deadWhy"][s(&f["whyCode"])]);
            copy(w, lang, "provDead", &[&s(&f["verdict"]), &why])
        }
        Some("docdup") => {
            let fill = [
                s(&f["twin"]),
                span[0].clone(),
                span[1].clone(),
                s(&f["jaccard"][0]),
                s(&f["jaccard"][1]),
                s(&f["verbatim"]),
            ];
            let linked = match f["linked"].as_u64() {
                Some(n) if n > 0 => copy(w, lang, "provLinked", &[&n.to_string()]),
                _ => String::new(),
            };
            copy(w, lang, "provDoc", &fill.each_ref().map(String::as_str)) + &linked
        }
        _ => {
            let fill = [
                s(&f["twin"]),
                span[0].clone(),
                span[1].clone(),
                s(&f["tokens"]),
                s(&f["unit"]),
            ];
            copy(w, lang, "provTwin", &fill.each_ref().map(String::as_str))
        }
    }
}
