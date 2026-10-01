//! The slot tables against their grammars and the flow probes (plan
//! v2.31 step 7, step-7 ruling 2; merge generation 2 ruling R8): every
//! kind, field and operator a table names is its grammar's; every named
//! node of the language's flow probe (scripts/tsprobe/snippets/flow.*)
//! that answers "other" is a kind the table names other, a part, a
//! literal's content or a target root — an omission is named, never
//! read as a decision; the table's kind sets are pairwise disjoint; a
//! table names its own statement forms and containers exactly when its
//! language has no flow table (Haskell, step-7 ruling 8); every table
//! carries its helper lines; and per table a probe pins the classes the
//! second generation moved — a target root and a bare target, a part
//! name, a literal's content, a type's own declared name.

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

/// The languages the package gives a slot table.
fn slotted() -> impl Iterator<Item = Lang> {
    Lang::ALL.into_iter().filter(|&l| slot_spec(l).is_some())
}

fn kind_sets(s: &SlotSpec) -> [(&'static str, &Vec<String>); 4] {
    [
        ("expr", &s.expr_kinds),
        ("type", &s.type_kinds),
        ("other", &s.other_kinds),
        ("part", &s.part_kinds),
    ]
}

/// Every (kind, field) pair a table names, by what it is for.
fn pair_sets(s: &SlotSpec) -> [(&'static str, &Vec<(String, String)>); 3] {
    [
        ("name", &s.name_fields),
        ("target", &s.target_fields),
        ("part", &s.part_fields),
    ]
}

/// Leg 1: every kind (named) and field of every table is its grammar's.
#[test]
fn every_kind_and_field_is_in_the_grammar() {
    for lang in slotted() {
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
        for k in s
            .stmt_kinds
            .iter()
            .chain(&s.container_kinds)
            .chain(&s.target_lists)
        {
            assert!(
                g.id_for_node_kind(k, true) != 0,
                "{lang:?}: statement, container or target-list kind `{k}`"
            );
        }
        let ops = s.target_ops.iter().map(|(k, f, _)| (k.clone(), f.clone()));
        let pairs: Vec<(&str, (String, String))> = pair_sets(s)
            .into_iter()
            .flat_map(|(what, v)| v.iter().map(move |p| (what, p.clone())))
            .chain(ops.map(|p| ("target op", p)))
            .collect();
        for (what, (k, f)) in pairs {
            assert!(
                g.id_for_node_kind(&k, true) != 0,
                "{lang:?}: {what} kind `{k}`"
            );
            assert!(
                f.is_empty() || g.field_id_for_name(&f).is_some(),
                "{lang:?}: field `{f}`"
            );
        }
        for (_, _, op) in &s.target_ops {
            assert!(
                g.id_for_node_kind(op, false) != 0,
                "{lang:?}: operator `{op}`"
            );
        }
    }
}

/// Leg 2 (coverage): over the language's flow probe every named node
/// that answers 4 is of a kind the table names other.
#[test]
fn every_other_answer_on_the_probe_is_named_other() {
    let mut missing = BTreeSet::new();
    for lang in slotted() {
        let (s, c) = (slot_spec(lang).unwrap(), classes(lang).unwrap());
        let text = probe(lang);
        let tree = crate::scan::ast::parse_lang(&text, lang).expect("parses");
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            stack.extend(crate::scan::ast::children(n));
            if n.is_named() && slot_of(c, n) == 4 && !explained(s, c, n) {
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

/// A node answering 4 the table decided: a kind named other or a
/// literal's content, a part on its parent's part field, or a target
/// root that is not a bare identifier.
fn explained(s: &SlotSpec, c: &Classes, n: Node) -> bool {
    let named = |v: &Vec<String>| v.iter().any(|k| k == n.kind());
    let part = n.parent().is_some_and(|p| {
        let f = field_of(p, n).unwrap_or("");
        c.parts.contains(&(p.kind(), f))
    });
    named(&s.other_kinds) || named(&s.part_kinds) || part || targeted(c, n)
}

/// Leg 3: the kind sets are pairwise disjoint, none holds a statement
/// form, no name pair repeats, and the statement and container lists
/// are the table's own exactly when no flow table gives them.
#[test]
fn the_sets_are_disjoint() {
    for lang in slotted() {
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

/// Leg 4 (ruling R6): every table carries its helper lines — a merged
/// function around a fragment adds a head and a closing line in a brace
/// language, a head alone where the body is indented (Python, Haskell).
#[test]
fn every_table_carries_its_helper_lines() {
    for lang in slotted() {
        let want = if matches!(lang, Lang::Python | Lang::Haskell) {
            1
        } else {
            2
        };
        assert_eq!(slot_spec(lang).unwrap().helper_lines, want, "{lang:?}");
    }
}

/// The probes of leg 5: per language a source and the classes its
/// nodes must answer, `kind@text=class` (the first node of that kind
/// spelling that text, in preorder; `\n` a newline).
const MOVED: &str = r#"python | def f(self, v):\n    self.a = v\n    x = "hi"\n    a, b = 1, 2\n    o.foo(p)\n | attribute@self.a=4 identifier@x=3 identifier@b=3 identifier@foo=4 string_content@hi=4 identifier@o=1
typescript | function f(v) { this.a = v; x = "hi"; y++; o.foo(p); }\nclass K {}\n | member_expression@this.a=4 identifier@x=3 identifier@y=3 property_identifier@foo=4 string_fragment@hi=4 type_identifier@K=3 identifier@o=1
tsx | function f(v) { this.a = v; x = "hi"; o.foo(p); }\n | member_expression@this.a=4 identifier@x=3 property_identifier@foo=4 string_fragment@hi=4
rust | struct S;\nfn f(v: i32) { self.a = v; x = "hi"; y += 1; o.foo(p); }\n | type_identifier@S=3 field_expression@self.a=4 identifier@x=3 identifier@y=3 field_identifier@foo=4 string_content@hi=4
go | package p\ntype T int\nfunc f(v int) { s.a = v; x = "hi"; y++; o.foo(p) }\n | type_identifier@T=3 selector_expression@s.a=4 identifier@x=3 identifier@y=3 field_identifier@foo=4 interpreted_string_literal_content@hi=4
c | void f(int v) { s.a = v; x = "hi"; y++; o.foo(p); }\n | field_expression@s.a=4 identifier@x=3 identifier@y=3 field_identifier@foo=4 string_content@hi=4
cpp | void f(int v) { s.a = v; x = "hi"; y++; o.foo(p); }\n | field_expression@s.a=4 identifier@x=3 identifier@y=3 field_identifier@foo=4 string_content@hi=4
java | class C { void f(int v) { this.a = v; x = "hi"; y++; o.foo(p); } }\n | field_access@this.a=4 identifier@x=3 identifier@y=3 identifier@foo=4 string_fragment@hi=4 identifier@C=3
lua | function f(v) self.a = v; x = "hi"; o:foo(p) end\n | dot_index_expression@self.a=4 identifier@x=3 identifier@foo=4 string_content@hi=4 identifier@a=4
r | f <- function(v) { x <- "hi"; 2 -> z; o$a <- v }\n | identifier@f=3 identifier@x=3 identifier@z=3 extract_operator@o$a=4 identifier@a=4 string_content@hi=4
haskell | f r = r.field\n | field_name@field=4"#;

/// Leg 5 (ruling R8): per table, the classes the second generation
/// moved, read off a parse of a probe source.
#[test]
fn the_moved_classes_answer_on_their_probes() {
    let mut wrong = Vec::new();
    for row in MOVED.lines() {
        let cols: Vec<&str> = row.split(" | ").collect();
        let lang = slotted()
            .find(|l| l.name() == cols[0])
            .unwrap_or_else(|| panic!("{}: no table", cols[0]));
        let src = cols[1].replace("\\n", "\n");
        let tree = crate::scan::ast::parse_lang(&src, lang).expect("parses");
        let c = classes(lang).unwrap();
        for want in cols[2].split_whitespace() {
            let (node, class) = want.rsplit_once('=').expect("kind@text=class");
            let (kind, text) = node.split_once('@').expect("kind@text");
            let got = first(tree.root_node(), &src, kind, text).map(|n| slot_of(c, n));
            if got != Some(class.parse().expect("a class")) {
                wrong.push(format!("{} {want}: {got:?}", cols[0]));
            }
        }
    }
    assert!(wrong.is_empty(), "moved classes:\n{}", wrong.join("\n"));
}

/// The first node of `kind` spelling `text`, in preorder.
fn first<'t>(root: Node<'t>, src: &str, kind: &str, text: &str) -> Option<Node<'t>> {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.kind() == kind && &src[n.byte_range()] == text {
            return Some(n);
        }
        stack.extend(crate::scan::ast::children(n).into_iter().rev());
    }
    None
}
