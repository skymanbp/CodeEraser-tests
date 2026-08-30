//! The report-id family: every value-shaped `"ce.<name>/<ver>"`
//! literal under cli/src, closed both ways against the two tables
//! below.

use super::{Fact, Form, linked, scraped};
use crate::common::files_with_ext;
use std::collections::BTreeMap;
use std::path::Path;

/// Report ids reachable through a `pub` path, by family name.
const LINKED: &[(&str, &str)] = &[
    ("baseline", codeeraser::score::baseline::SCHEMA_ID),
    ("check", codeeraser::score::model::SCHEMA_ID),
    ("clone", codeeraser::dedup::t3::SCHEMA_ID),
    ("clone-units", codeeraser::dedup::unitcache::UNITS_SCHEMA_ID),
    ("dedup", codeeraser::dedup::SCHEMA_ID),
    ("docdup", codeeraser::docdup::judge::SCHEMA_ID),
    ("doctor", codeeraser::health::doctor::SCHEMA_ID),
    ("erase-plan", codeeraser::erase::SCHEMA_ID),
    ("graph-canvas", codeeraser::graph::canvas::SCHEMA_ID),
    ("join", codeeraser::join::SCHEMA_ID),
    ("mentions", codeeraser::mention::face::SCHEMA_ID),
    ("observe", codeeraser::hookio::OBSERVE_SCHEMA),
    ("scan", codeeraser::scan::report::SCHEMA),
    ("sites", codeeraser::graph::SCHEMA_ID),
    ("structure", codeeraser::structure::judge::SCHEMA_ID),
    ("update", codeeraser::update::SCHEMA_ID),
];

/// Report ids whose const sits behind a private module — scraped,
/// each with the promotion that would link it.
const PRIVATE: &[(&str, &str)] = &[
    (
        "churn",
        "churn::report is private; promote = re-export SCHEMA beside Report",
    ),
    (
        "deadcode",
        "report::DEADCODE_SCHEMA is private; promote = pub const",
    ),
    (
        "erase-log",
        "erase::apply::LOG_SCHEMA is private; promote = re-export beside SCHEMA_ID",
    ),
    (
        "trend",
        "trend::report is private; promote = re-export SCHEMA_ID beside Report",
    ),
];

/// The report-id family: every value-shaped `"ce.<name>/<ver>"`
/// literal under cli/src, keyed by name with a `-report` suffix
/// dropped, closed BOTH ways against the two tables — a family reaches
/// the docs only by being enrolled, and an enrolled family must still
/// be spelled by the product.
pub fn facts(root: &Path) -> Vec<Fact> {
    let scanned = scan_report_ids(root);
    for (name, _) in LINKED.iter().chain(PRIVATE) {
        assert!(
            scanned.contains_key(*name),
            "{name}: enrolled, but no literal under cli/src spells it"
        );
    }
    scanned
        .iter()
        .map(|(name, (value, file))| {
            let id = format!("report:{name}#schemaver");
            if let Some((_, typed)) = LINKED.iter().find(|(n, _)| n == name) {
                assert_eq!(
                    typed, value,
                    "{file}: the scanned literal and the typed const disagree"
                );
                linked(&id, value, &format!("{file} (typed)"))
            } else if let Some((_, debt)) = PRIVATE.iter().find(|(n, _)| n == name) {
                scraped(&id, value, file, debt)
            } else {
                panic!("{file}: report id {name:?} is enrolled in neither table (facts/ver.rs)")
            }
        })
        .collect()
}

/// name → (literal, repo-relative file); a name spelled in two files
/// is a refusal (one family, one spelling).
fn scan_report_ids(root: &Path) -> BTreeMap<String, (String, String)> {
    let mut files = Vec::new();
    files_with_ext(&root.join("cli/src"), "rs", &mut files);
    files.sort();
    let mut found = BTreeMap::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        for lit in literals(&std::fs::read_to_string(&path).expect("read source")) {
            let name = lit["ce.".len()..]
                .split('/')
                .next()
                .expect("name")
                .trim_end_matches("-report")
                .to_string();
            if let Some((prev, at)) = found.insert(name.clone(), (lit.clone(), rel.clone())) {
                panic!("report id {name:?} spelled at {at} ({prev}) and {rel} ({lit})");
            }
        }
    }
    found
}

/// Every `"ce.<name>/<ver>"` string literal in `text`, in order.
fn literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("\"ce.") {
        let body = &rest[i + 1..];
        let end = body.find('"').unwrap_or(body.len());
        if Form::SchemaVer.admits(&body[..end]) {
            out.push(body[..end].to_string());
        }
        rest = &body[end..];
    }
    out
}
