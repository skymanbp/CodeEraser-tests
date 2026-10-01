//! The slot tables against their grammars and the flow probes (plan
//! v2.31 step 7, step-7 ruling 2): every kind and field a table names
//! is its grammar's; every named node of the language's flow probe
//! (scripts/tsprobe/snippets/flow.*) that answers "other" is a kind the
//! table names other — an omission is named, never read as a
//! decision; the table's kind sets are pairwise disjoint; and a table
//! names its own statement forms and containers exactly when its
//! language has no flow table (Haskell, step-7 ruling 8).

use super::*;
use std::collections::BTreeSet;

/// The flow probe of each table's language, by its extension.
const PROBES: &str =
    "python py|typescript ts|tsx tsx|rust rs|go go|c c|cpp cpp|java java|lua lua|r R|haskell hs";

fn probe(lang: Lang) -> String {
    let ext = PROBES
        .split('|')
        .filter_map(|row| row.split_once(' '))
        .find_map(|(name, ext)| (name == lang.name()).then_some(ext))
        .unwrap_or_else(|| panic!("{lang:?}: no probe"));
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../scripts/tsprobe/snippets")
        .join(format!("flow.{ext}"));
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

fn kind_sets(s: &SlotSpec) -> [(&'static str, &Vec<String>); 3] {
    [
        ("expr", &s.expr_kinds),
        ("type", &s.type_kinds),
        ("other", &s.other_kinds),
    ]
}

/// Leg 1: every kind (named) and field of every table is its grammar's.
#[test]
fn every_kind_and_field_is_in_the_grammar() {
    for (lang, _) in TABLES {
        let s = slot_spec(lang).expect("a slot table");
        let g = lang.grammar().expect("a grammar");
        for (what, kinds) in kind_sets(s) {
            for k in kinds {
                assert!(
                    g.id_for_node_kind(k, true) != 0,
                    "{lang:?}: {what} kind `{k}`"
                );
            }
        }
        for k in s.stmt_kinds.iter().chain(&s.container_kinds) {
            assert!(
                g.id_for_node_kind(k, true) != 0,
                "{lang:?}: statement or container kind `{k}`"
            );
        }
        for (k, f) in &s.name_fields {
            assert!(
                g.id_for_node_kind(k, true) != 0,
                "{lang:?}: name kind `{k}`"
            );
            assert!(g.field_id_for_name(f).is_some(), "{lang:?}: field `{f}`");
        }
    }
}

/// Leg 2 (coverage): over the language's flow probe every named node
/// that answers 4 is of a kind the table names other.
#[test]
fn every_other_answer_on_the_probe_is_named_other() {
    let mut missing = BTreeSet::new();
    for (lang, _) in TABLES {
        let (s, c) = (slot_spec(lang).unwrap(), classes(lang).unwrap());
        let text = probe(lang);
        let tree = crate::scan::ast::parse_lang(&text, lang).expect("parses");
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            stack.extend(crate::scan::ast::children(n));
            if n.is_named() && slot_of(c, n) == 4 && !s.other_kinds.iter().any(|k| k == n.kind()) {
                let parent = n.parent().map_or("-", |p| p.kind());
                missing.insert(format!("{} {} < {parent}", lang.name(), n.kind()));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "unnamed other:\n{}",
        missing.into_iter().collect::<Vec<_>>().join("\n")
    );
}

/// Leg 3: the kind sets are pairwise disjoint, none holds a statement
/// form, no name pair repeats, and the statement and container lists
/// are the table's own exactly when no flow table gives them.
#[test]
fn the_sets_are_disjoint() {
    for (lang, _) in TABLES {
        let s = slot_spec(lang).unwrap();
        let c = classes(lang).unwrap();
        let empty = s.stmt_kinds.is_empty() && s.container_kinds.is_empty();
        assert_eq!(
            empty,
            crate::flow::spec::spec(lang).is_some(),
            "{lang:?}: statement forms and containers come from the flow table or from this one"
        );
        let mut seen = BTreeSet::new();
        for (what, kinds) in kind_sets(s) {
            for k in kinds {
                assert!(seen.insert(k.as_str()), "{lang:?}: `{k}` twice ({what})");
                assert!(
                    !c.statement.contains(k.as_str()),
                    "{lang:?}: {what} `{k}` is a statement form"
                );
            }
        }
        let pairs: BTreeSet<_> = s.name_fields.iter().collect();
        assert_eq!(
            pairs.len(),
            s.name_fields.len(),
            "{lang:?}: a name pair twice"
        );
    }
}
