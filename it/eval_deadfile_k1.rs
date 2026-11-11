//! The dead-file exam's classifiers on small texts (lane v235_licence,
//! phase 2A): for each language, each numbered rule of BARS.md §2 and
//! each ruling of BARS-ADDENDUM-2.md the language has a syntax for, an
//! inert and an active example. One table, one case per block:
//! `lang | k1 class | k1-strict class | rulings met | k1 witness`, then
//! the text. A witness is `kind@line` or `-`. Java has no top-level
//! form that runs (anything else is a parse error), so its rules carry
//! only inert examples.

use crate::eval_deadfile_parts::k1::classify;
use k1_cases::SCRIPTED;
use k1_cases_more::OTHERS;

// the two halves of the case table, owned by this leg alone
#[path = "eval_deadfile_parts/k1_cases.rs"]
mod k1_cases;
#[path = "eval_deadfile_parts/k1_cases_more.rs"]
mod k1_cases_more;
use codeeraser::scan::lang::Lang;

/// The cases: (header fields, text).
fn cases() -> Vec<(Vec<&'static str>, String)> {
    [SCRIPTED, OTHERS]
        .iter()
        .flat_map(|t| t.split("==== "))
        .filter(|b| !b.trim().is_empty())
        .map(|b| {
            let (head, text) = b.split_once('\n').expect("a header line");
            (head.split(" | ").collect(), text.to_string())
        })
        .collect()
}

fn lang(name: &str) -> Lang {
    *Lang::ALL.iter().find(|l| l.name() == name).expect(name)
}

#[test]
fn every_rule_and_ruling_reads_as_registered() {
    let mut bad = Vec::new();
    for (h, text) in cases() {
        let got = classify(&text, Some(lang(h[0])), false);
        let rulings = got
            .rulings
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let witness = got
            .k1
            .witness
            .as_ref()
            .map_or("-".to_string(), |(k, l)| format!("{k}@{l}"));
        let read = [
            got.k1.class,
            got.strict.class,
            if rulings.is_empty() { "-" } else { &rulings },
            &witness,
        ];
        if read != [h[1], h[2], h[3], h[4]] || !got.undecided.is_empty() {
            bad.push(format!("{} ({}): {read:?} {:?}", h[5], h[0], got.undecided));
        }
    }
    assert!(
        bad.is_empty(),
        "{} cases misread:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn a_document_is_a_document_under_both_readings() {
    let got = classify("# Title\n", Some(lang("markdown")), true);
    assert_eq!((got.k1.class, got.strict.class), ("document", "document"));
    assert!(got.k1.witness.is_none() && got.rulings.is_empty());
}

#[test]
fn a_construct_no_rule_places_is_named_not_decided() {
    // a bare number as a statement: no rule of section 2 and no ruling places it
    let got = classify("1\n", Some(lang("python")), false);
    assert_eq!(got.undecided, vec![("bare integer".to_string(), 1)]);
}
