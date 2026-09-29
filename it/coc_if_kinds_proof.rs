//! The equivalence proof for the walker's two generalisations (plan
//! v2.30 step 6 — cognitive.rs's then, the emitter's alternative
//! class and the core's else rule since step 7b ③; the booklet's §11
//! "mechanism extension (d)"): the
//! else bonus off an if's `alternative` FIELD was `alternative.kind ==
//! "block"` and became "anything there but the next if or a flat
//! branch node", and the walker knew an if by `kind.starts_with("if")`
//! and now reads the exact `if_kinds` table. Both were changed so that
//! Java's single-statement else and R's expression else score, and
//! the booklet promised the launch languages move by nothing. The
//! metric batteries show that on their rows; this leg shows it on the
//! GRAMMARS — every kind each pinned crate's node-types.json can put
//! in an if's `alternative`, and every kind spelled `if…` — so the
//! claim holds for every tree the parser can build, not for the trees
//! a table happened to spell. The new languages' partitions are pinned
//! beside them: what the generalisation buys, counted.
//!
//! Supertypes (Java's `statement`, `expression`) are expanded through
//! their subtypes, as the parser produces the concrete kind.

use codeeraser::scan::lang::Lang;
use codeeraser::scan::spec::{Kinds, spec};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Every judged language that parses, with its crate's node-types.
const GRAMMARS: &[(Lang, &str)] = &[
    (Lang::Python, tree_sitter_python::NODE_TYPES),
    (
        Lang::TypeScript,
        tree_sitter_typescript::TYPESCRIPT_NODE_TYPES,
    ),
    (Lang::Tsx, tree_sitter_typescript::TSX_NODE_TYPES),
    (Lang::Rust, tree_sitter_rust::NODE_TYPES),
    (Lang::Go, tree_sitter_go::NODE_TYPES),
    (Lang::Haskell, tree_sitter_haskell::NODE_TYPES),
    (Lang::Java, tree_sitter_java::NODE_TYPES),
    (Lang::C, tree_sitter_c::NODE_TYPES),
    (Lang::Cpp, tree_sitter_cpp::NODE_TYPES),
    (Lang::Lua, tree_sitter_lua::NODE_TYPES),
    (Lang::R, tree_sitter_r::NODE_TYPES),
];

/// The languages the generalisation must not move: everything judged
/// before plan v2.30.
const LAUNCH: &[Lang] = &[
    Lang::Python,
    Lang::TypeScript,
    Lang::Tsx,
    Lang::Rust,
    Lang::Go,
    Lang::Haskell,
];

/// Per language: the concrete kinds an if's `alternative` can hold,
/// how many of them the present rule pays the else +1 for, and how
/// many the retired `== "block"` rule paid. The launch rows agree
/// column for column (and kind for kind, asserted below); Java and R
/// are the gain — a single-statement else and an expression else the
/// block rule never saw — while C, C++ and Lua hang an else node in
/// the field and pay it by the flat rule, as TypeScript does.
const PARTITION: &str = "\
python 2 0 0
typescript 1 0 0
tsx 1 0 0
rust 1 0 0
go 2 1 1
haskell 0 0 0
java 28 27 1
c 1 0 0
cpp 1 0 0
lua 2 0 0
r 29 28 0";

struct Grammar {
    by: BTreeMap<String, Value>,
    supers: BTreeMap<String, Vec<String>>,
}

/// The kinds a `types` list names.
fn types(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t["type"].as_str())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

impl Grammar {
    fn parse(json: &str) -> Grammar {
        let nodes: Vec<Value> = serde_json::from_str(json).expect("node-types.json");
        let (mut by, mut supers) = (BTreeMap::new(), BTreeMap::new());
        for n in nodes {
            let kind = n["type"].as_str().expect("type").to_string();
            if let Some(subs) = n.get("subtypes") {
                supers.insert(kind.clone(), types(subs));
            }
            by.insert(kind, n);
        }
        Grammar { by, supers }
    }

    /// The concrete kinds a type list can produce: a supertype stands
    /// for each of its subtypes, recursively.
    fn expand(&self, kinds: &[String]) -> BTreeSet<String> {
        let (mut out, mut seen) = (BTreeSet::new(), BTreeSet::new());
        let mut stack = kinds.to_vec();
        while let Some(k) = stack.pop() {
            match self.supers.get(&k) {
                Some(subs) if seen.insert(k.clone()) => stack.extend(subs.iter().cloned()),
                Some(_) => {}
                None => {
                    out.insert(k);
                }
            }
        }
        out
    }

    /// Named concrete kinds spelled `if…` — what the retired prefix
    /// test read as an if.
    fn if_prefixed(&self) -> BTreeSet<String> {
        self.by
            .values()
            .filter(|n| n["named"] == true && n.get("subtypes").is_none())
            .filter_map(|n| n["type"].as_str())
            .filter(|k| k.starts_with("if"))
            .map(String::from)
            .collect()
    }

    /// The concrete kinds a node's `alternative` field can hold; None
    /// when the kind has no such field.
    fn alternatives(&self, kind: &str) -> Option<BTreeSet<String>> {
        let field = self.by.get(kind)?.get("fields")?.get("alternative")?;
        Some(self.expand(&types(&field["types"])))
    }

    /// Every concrete kind that can be a direct child of `kind`, in
    /// a field or not.
    fn children(&self, kind: &str) -> BTreeSet<String> {
        let node = &self.by[kind];
        let mut raw = Vec::new();
        if let Some(c) = node.get("children") {
            raw.extend(types(&c["types"]));
        }
        if let Some(fields) = node.get("fields").and_then(Value::as_object) {
            for f in fields.values() {
                raw.extend(types(&f["types"]));
            }
        }
        self.expand(&raw)
    }
}

/// The kinds a kind table names (an entry's first token; the rest are
/// its body positions).
fn kinds_of(table: Kinds) -> BTreeSet<&'static str> {
    table.iter().filter_map(|e| e.split(' ').next()).collect()
}

/// One grammar beside the two kind tables the walker consults for
/// its language.
struct Site {
    lang: Lang,
    g: Grammar,
    ifk: BTreeSet<&'static str>,
    flat: BTreeSet<&'static str>,
}

impl Site {
    /// Every concrete kind an if of this language can hold in its
    /// `alternative` field.
    fn alternatives(&self) -> BTreeSet<String> {
        self.ifk
            .iter()
            .filter_map(|k| self.g.alternatives(k))
            .flatten()
            .collect()
    }
}

/// Each grammar in GRAMMARS, parsed, with its tables.
fn sites() -> Vec<Site> {
    GRAMMARS
        .iter()
        .map(|&(lang, json)| {
            let sp = spec(lang);
            Site {
                lang,
                g: Grammar::parse(json),
                ifk: kinds_of(sp.if_kinds),
                flat: kinds_of(sp.coc_flat_kinds),
            }
        })
        .collect()
}

fn name(lang: Lang) -> String {
    format!("{lang:?}").to_lowercase()
}

/// Every kind the prefix test read as an if that the exact table does
/// not name is inert at each site the walker consults the table: it
/// has no `alternative` field (the else bonus), it never sits in an
/// if's `alternative` (the else-if test), and it is never a direct
/// child of a flat branch node (the yielded +1) — so the exact table
/// is the prefix test with the one accident removed (Python's
/// comprehension `if_clause`), and the table names nothing the prefix
/// would not have read.
#[test]
fn the_exact_if_table_is_the_prefix_test_with_its_accidents_removed() {
    for s in sites() {
        let prefixed = s.g.if_prefixed();
        for k in &s.ifk {
            assert!(
                prefixed.contains(*k),
                "{}: if_kinds names {k}, no `if…` kind",
                name(s.lang)
            );
        }
        let in_alternatives = s.alternatives();
        let under_flat: BTreeSet<String> = s.flat.iter().flat_map(|k| s.g.children(k)).collect();
        for k in prefixed.iter().filter(|k| !s.ifk.contains(k.as_str())) {
            assert!(
                s.g.alternatives(k).is_none()
                    && !in_alternatives.contains(k)
                    && !under_flat.contains(k),
                "{}: {k} is spelled `if…` and is not inert",
                name(s.lang)
            );
        }
    }
}

/// The retired rule (`alternative.kind == "block"`) and the present
/// one (neither the next if nor a flat branch node) agree kind for
/// kind over every alternative a launch grammar can produce; the
/// PARTITION table records what each language's alternatives are and
/// how the two rules count them.
#[test]
fn the_field_else_bonus_moves_no_launch_language() {
    let mut rows = Vec::new();
    for s in sites() {
        let alternatives = s.alternatives();
        let present = |k: &str| !s.ifk.contains(k) && !s.flat.contains(k);
        let retired = |k: &str| k == "block";
        if LAUNCH.contains(&s.lang) {
            for k in &alternatives {
                assert_eq!(
                    present(k),
                    retired(k),
                    "{}: the rules part on {k}",
                    name(s.lang)
                );
            }
        }
        rows.push(format!(
            "{} {} {} {}",
            name(s.lang),
            alternatives.len(),
            alternatives.iter().filter(|k| present(k)).count(),
            alternatives.iter().filter(|k| retired(k)).count()
        ));
    }
    assert_eq!(rows.join("\n"), PARTITION);
}
