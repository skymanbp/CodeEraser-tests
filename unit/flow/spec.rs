//! The flow tables against their grammars (plan v2.31 step 4 A1): every
//! kind, field and token a table names is in the pinned grammar of each
//! language that reads it, the name tables are plain, the dispatch
//! answers the judged code languages and no other, and every table
//! holds containers, exits and names.

use super::*;
use crate::flow::pos::{Step, paths};
use std::collections::{BTreeMap, BTreeSet};
use toml::Value;

/// Every Lang, in code order (the dispatch leg pins the order); the
/// variants bare, so the list is no copy of a `[T::A, T::B, …]` table.
const ALL: [Lang; 22] = {
    use Lang::*;
    [
        Python,
        TypeScript,
        Tsx,
        Rust,
        Go,
        Markdown,
        Haskell,
        LangUnknown,
        JavaScript,
        Css,
        Html,
        Vue,
        Svelte,
        Shell,
        Yaml,
        C,
        Cpp,
        Lua,
        Java,
        Ruby,
        R,
        Text,
    ]
};

/// What each key's strings are — `kind`, `token`, `name`, `pos` (a
/// position: its fields and kinds), `mark` (a token, or `@kind`) or
/// `skip` — one per element of a tuple; `rows` for a row or row list.
const CATEGORIES: &str = "
kind: branch_binders interpolated_strings prototype_kinds prototype_reads type_reads head_reads block_kinds splice_kinds wrapper_kinds empty_kinds elif_kinds return_kinds throw_kinds break_kinds continue_kinds yield_kinds fallthrough_kinds int_kinds dynamic_kinds pattern_kinds pattern_idents dotted_patterns nonlocal_kinds local_only_scopes ident_kinds shorthand_kinds member_write_bases capture_kinds ref_binding_kinds kind pair strings
token: lasting_storage token op
name: noreturn return_calls dynamic_names receiver_names discard_names dispatch_calls
pos: results self_label header body cond init update target iter else then subject arms binder pattern value guard param resources item items storage alternative left right name args missing
kind pos: else_kinds finally_kinds gotos call_forms params binder_paths pattern_binders name_positions update_kinds default_arg_fields
kind pos pos: cond_wrappers labels
kind token: const_true const_false
kind name: noreturn_attrs
kind mark: address_ops field_params
kind pos token: conditional_ctx
pos token: holds
token skip: marker
skip: scoping first_write_declares forward_captures upper_pattern_paths default fallthrough mode scope body_first until always_default empty_noreturn passes_break redeclare
rows: if loops switches cases tries catches withs decls assigns macros
";

/// The strings FAMILY names that only C++ spells, as `what text…`.
const CPP_ONLY: &str = "
kind condition_clause init_statement field_initializer_list for_range_loop try_statement catch_clause throw_statement lambda_expression qualified_identifier reference_declarator structured_binding_declarator optional_parameter_declaration
field default_value
token and or
";

type Map = BTreeMap<&'static str, Vec<&'static str>>;
type Out = Vec<(&'static str, String)>;

/// The ten flow tables, in code order.
fn tables() -> impl Iterator<Item = (Lang, &'static FlowSpec)> {
    ALL.into_iter().filter_map(|l| spec(l).map(|s| (l, s)))
}

fn categories() -> Map {
    let mut map = Map::new();
    for line in CATEGORIES.lines().filter(|l| !l.is_empty()) {
        let (cats, keys) = line.split_once(": ").expect("`categories: keys`");
        for key in keys.split(' ') {
            let fresh = map.insert(key, cats.split(' ').collect()).is_none();
            assert!(fresh, "`{key}` has two categories");
        }
    }
    map
}

/// Every string of a table as (what, text): what is `kind`, `token`,
/// `field` or `name`, a position giving its fields and kinds.
fn strings_of(spec: &FlowSpec) -> Out {
    let value = Value::try_from(spec).expect("a table serializes");
    let mut out = Out::new();
    walk(&value, &["rows"], &categories(), &mut out);
    out
}

fn walk(value: &Value, cats: &[&'static str], map: &Map, out: &mut Out) {
    match value {
        Value::Table(table) => {
            for (key, v) in table {
                let cats = map.get(key.as_str());
                walk(
                    v,
                    cats.unwrap_or_else(|| panic!("`{key}`: no category")),
                    map,
                    out,
                );
            }
        }
        Value::Array(items) if cats.len() > 1 && !items.iter().all(Value::is_array) => {
            assert_eq!(items.len(), cats.len(), "a tuple of {cats:?}: {items:?}");
            for (item, cat) in items.iter().zip(cats) {
                walk(item, std::slice::from_ref(cat), map, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| walk(item, cats, map, out)),
        Value::String(text) => emit(cats[0], text, out),
        _ => {}
    }
}

fn emit(cat: &'static str, text: &str, out: &mut Out) {
    match cat {
        "kind" | "name" => out.push((cat, text.to_owned())),
        "token" if !text.is_empty() => out.push((cat, text.to_owned())),
        "pos" => out.extend(paths(text).into_iter().flatten().map(|step| match step {
            Step::Field(f) => ("field", f.to_owned()),
            Step::Kind(k) => ("kind", k.to_owned()),
        })),
        "mark" => out.push(match text.strip_prefix('@') {
            Some(kind) => ("kind", kind.to_owned()),
            None => ("token", text.to_owned()),
        }),
        "token" | "skip" => {}
        other => panic!("unknown category `{other}`"),
    }
}

fn known(grammar: &tree_sitter::Language, what: &str, text: &str) -> bool {
    match what {
        "kind" => grammar.id_for_node_kind(text, true) != 0,
        "token" => grammar.id_for_node_kind(text, false) != 0,
        "field" => grammar.field_id_for_name(text).is_some(),
        _ => true, // names: the plain-names leg
    }
}

fn cpp_only() -> Vec<(&'static str, &'static str)> {
    let lines = CPP_ONLY.lines().filter_map(|l| l.split_once(' '));
    lines
        .flat_map(|(what, texts)| texts.split(' ').map(move |t| (what, t)))
        .collect()
}

/// Leg 1: every kind, token and field of every table is its grammar's
/// — C reading FAMILY less the C++-only strings leg 1b pins.
#[test]
fn every_string_is_in_the_grammar() {
    let cpp_only = cpp_only();
    for (lang, spec) in tables() {
        let grammar = lang.grammar().expect("a flow language parses");
        for (what, text) in strings_of(spec) {
            if lang == Lang::C && cpp_only.contains(&(what, text.as_str())) {
                continue;
            }
            let at = "is not in its grammar";
            assert!(
                known(&grammar, what, &text),
                "{lang:?}: {what} `{text}` {at}"
            );
        }
    }
}

/// Leg 1b: the C++-only strings are exactly that — absent from the C
/// grammar, present in the C++ one, and still named by the table.
#[test]
fn the_cpp_only_strings_are_cpp_only() {
    let (c, cpp) = (Lang::C.grammar().unwrap(), Lang::Cpp.grammar().unwrap());
    let named = strings_of(spec(Lang::Cpp).expect("a C++ table"));
    for (what, text) in cpp_only() {
        assert!(!known(&c, what, text), "C spells {what} `{text}`");
        assert!(known(&cpp, what, text), "C++ lacks {what} `{text}`");
        let used = named.iter().any(|(w, t)| *w == what && t == text);
        assert!(used, "FAMILY no longer names {what} `{text}`");
    }
}

/// Leg 2: every name table holds plain names, each once — the
/// attribute table once per (kind, name): `noreturn` is the name both
/// in `__attribute__((noreturn))` and in `[[noreturn]]`.
#[test]
fn name_tables_are_plain_and_single() {
    for (lang, s) in tables() {
        let lists = [
            &s.noreturn,
            &s.return_calls,
            &s.dynamic_names,
            &s.receiver_names,
        ];
        for names in lists.into_iter().chain([&s.discard_names]) {
            let mut seen = BTreeSet::new();
            for n in names {
                assert!(plain(n), "{lang:?}: name `{n}`");
                assert!(seen.insert(n), "{lang:?}: `{n}` twice");
            }
        }
        let attrs: BTreeSet<_> = s.noreturn_attrs.iter().collect();
        assert_eq!(
            attrs.len(),
            s.noreturn_attrs.len(),
            "{lang:?}: an attribute twice"
        );
        assert!(attrs.iter().all(|(_, n)| plain(n)), "{lang:?}: {attrs:?}");
    }
}

fn plain(name: &str) -> bool {
    !name.is_empty() && name.is_ascii() && !name.contains(char::is_whitespace)
}

/// Leg 3: the judged languages less Markdown, Haskell and HTML have a
/// table and nothing else does; TypeScript and TSX read one (equal
/// tables: the package states each language's own); C and C++
/// part on the noreturn names alone.
#[test]
fn the_dispatch_answers_the_judged_code_languages() {
    for (code, lang) in ALL.into_iter().enumerate() {
        assert_eq!(lang as usize, code, "ALL is in code order");
        let judged = (Lang::judged_mask() >> code) & 1 == 1;
        let outside = matches!(lang, Lang::Markdown | Lang::Haskell | Lang::Html);
        assert_eq!(spec(lang).is_some(), judged && !outside, "{lang:?}");
    }
    let ts = spec(Lang::TypeScript).unwrap();
    assert_eq!(ts, spec(Lang::Tsx).unwrap(), "one TypeScript table");
    let (c, cpp) = (spec(Lang::C).unwrap(), spec(Lang::Cpp).unwrap());
    assert_ne!(c.noreturn, cpp.noreturn);
    let mut same = cpp.clone();
    same.noreturn.clone_from(&c.noreturn);
    assert_eq!(&same, c, "C and C++ part on the noreturn names alone");
}

/// Leg 4: every table holds a container, an exit and a name kind, and
/// the C family keeps a C-style for (an update, its condition optional).
#[test]
fn every_table_has_containers_exits_and_names() {
    for (lang, s) in tables() {
        assert!(!s.block_kinds.is_empty(), "{lang:?}: no container");
        let exit = !(s.return_kinds.is_empty() && s.return_calls.is_empty());
        assert!(exit, "{lang:?}: no return");
        assert!(!s.ident_kinds.is_empty(), "{lang:?}: no name kind");
    }
    let c_style = |lang| {
        spec(lang)
            .unwrap()
            .loops
            .iter()
            .any(|l| !l.update.is_empty())
    };
    assert!(c_style(Lang::C) && c_style(Lang::Cpp), "the C family's for");
}
