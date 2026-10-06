use super::*;

// The verdict over these facts is the core's (CE.Scan.Cost.conforms,
// its own battery); this side owes the five shape facts, pinned here as
// `language name: style upper under test` per row (snake for Python,
// mixed caps otherwise; the language code leads the row). Leading
// underscores are trimmed before the shape reads; Go's
// toolchain-mandated underscore families carry the test fact (go vet's
// rule: a lowercase boundary is no test name), and the fact is stated
// under every language — the core gates the exemption on Go's own code.
// A sentinel name carries no convention.
const ROWS: &str = "\
python load_config: 1 0 1 0
python __init__: 1 0 1 0
python loadConfig: 1 1 0 0
go loadConfig: 2 1 0 0
go ServeHTTP: 2 1 0 0
go load_config: 2 0 1 0
go ExampleParse_errors: 2 1 1 1
go TestServer_Start: 2 1 1 1
go Example_errors: 2 1 1 1
go Testing_helper: 2 1 1 0
typescript TestServer_Start: 2 1 1 1
typescript (anonymous): 0 0 0 0
typescript (non-utf8): 0 0 0 0
typescript \"my_key\": 0 0 0 0
typescript [dynamic_key]: 0 0 0 0";

/// One row read: the language and its style, the name, the facts.
fn read(row: &str) -> (Lang, NameStyle, &str, Vec<i64>) {
    let (head, want) = row.split_once(": ").expect("a row");
    let (lang, name) = head.split_once(' ').expect("a language and a name");
    let (lang, style) = match lang {
        "python" => (Lang::Python, NameStyle::Snake),
        "go" => (Lang::Go, NameStyle::MixedCaps),
        _ => (Lang::TypeScript, NameStyle::MixedCaps),
    };
    let want = std::iter::once(lang as i64)
        .chain(want.split(' ').map(|w| w.parse().expect("a fact")))
        .collect();
    (lang, style, name, want)
}

#[test]
fn every_row_states_its_shape_facts() {
    for (lang, style, name, want) in ROWS.lines().map(read) {
        assert_eq!(facts(lang, style, name).to_vec(), want, "{name}");
    }
}
