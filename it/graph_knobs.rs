//! The `[graph]` declarations a tree makes for its resolver — each a
//! knob the walk honours or refuses BY NAME, never drops (the
//! [structure] layout posture): `crate_roots` (plan v2.18 step #12)
//! for a Rust tree whose manifest lives elsewhere — the suite is a
//! slice of the `cli` package, its test binaries cargo targets only in
//! the superproject's Cargo.toml — and `search_roots` (plan v2.30 step
//! 2) for the C-family include roots. Each knob has a declared tree and
//! its undeclared control: the same files, where the knob is the only
//! difference the verdict can read. A declared crate root is
//! everything a manifest target is: the ladder mounts its `mod`
//! children in its own directory and anchors `crate::` paths there, and
//! the entry role sees a declared target; undeclared, the name
//! convention (role 0) still keeps the main, but it mounts nothing and
//! anchors nothing, so its child dies.

use crate::common;
use codeeraser::graph::deadcode;

const RUST: &str = "--- it/main.rs\nmod helper;\nuse crate::helper::h;\nfn main() {\n    h();\n}\n\
                    --- it/helper.rs\npub fn h() {}\n";
const RUST_DECLARED: &str = "--- ce.toml\n[graph]\ncrate_roots = [\"it/main.rs\"]\n";

/// `main.c` includes `lib.h`, which lives under `include/`: the
/// including file's own directory holds no such file, so the include
/// resolves only through a declared root. The unit role keeps
/// `main.c` either way (register D18); the header is kept by the
/// include edge or by nothing.
const C: &str =
    "--- src/main.c\n#include \"lib.h\"\nint main(void) { return 0; }\n--- include/lib.h\n\n";
const C_DECLARED: &str = "--- ce.toml\n[graph.search_roots]\nc = [\"include\"]\n";

/// (declared tree, its undeclared control, the one file that dies
/// undeclared) — the declared tree keeps everything and resolves at
/// least one edge; the control keeps the root by convention alone and
/// resolves nothing.
const PAIRS: [(&str, &str, &str, &str); 2] = [
    ("crate-roots", RUST, RUST_DECLARED, "it/helper.rs"),
    ("search-roots", C, C_DECLARED, "include/lib.h"),
];

fn judged(tag: &str, doc: &str) -> deadcode::Report {
    let dir = common::fixtures::doc_tree(tag, doc);
    deadcode::run(&dir, None, &common::core_bin()).expect("run")
}

fn dead_paths(report: &deadcode::Report) -> Vec<&str> {
    let mut dead: Vec<&str> = report.dead.iter().map(|d| d.path.as_str()).collect();
    dead.sort();
    dead
}

#[test]
fn a_declaration_mounts_anchors_and_keeps_what_the_control_loses() {
    for (knob, tree, declared, orphan) in PAIRS {
        let report = judged(&format!("{knob}-declared"), &format!("{tree}{declared}"));
        assert_eq!(
            dead_paths(&report),
            Vec::<&str>::new(),
            "{knob}: declared keeps all"
        );
        assert!(
            report.kept >= 1,
            "{knob}: the declared edge resolved: {report:?}"
        );
        let report = judged(&format!("{knob}-undeclared"), tree);
        assert_eq!(
            dead_paths(&report),
            [orphan],
            "{knob}: undeclared, only the root survives"
        );
        assert_eq!(report.kept, 0, "{knob}: and the edge falls out of scope");
    }
}

/// A declaration the walk cannot honour is refused by name, never
/// dropped: a crate root that is no Rust file would make a Markdown
/// page a deadcode target, a missing one would put the tree back in
/// the false-dead shape the knob exists to end (codex review of step
/// #12); a search root holding no walked file, or a key naming no
/// language, is the same silence in the other table.
#[test]
fn a_declaration_the_walk_cannot_honour_is_refused_by_name() {
    // tag | ce.toml body (`\n` spelled, one row per line) | two needles
    const ROWS: &str = "\
crate-roots-not-rust|[graph]\\ncrate_roots = [\"README.md\"]|crate_roots declares|README.md
crate-roots-missing|[graph]\\ncrate_roots = [\"it/gone.rs\"]|crate_roots declares|it/gone.rs
search-roots-missing|[graph.search_roots]\\nc = [\"inc\"]|search_roots] c declares|\"inc\"
search-roots-lang|[graph.search_roots]\\ncobol = [\"it\"]|search_roots] names|cobol";
    for row in ROWS.lines() {
        let [tag, toml, what, name]: [&str; 4] = row
            .split('|')
            .collect::<Vec<_>>()
            .try_into()
            .expect("four columns");
        let doc = format!(
            "{RUST}--- README.md\n# it\n--- ce.toml\n{}\n",
            toml.replace("\\n", "\n")
        );
        let dir = common::fixtures::doc_tree(tag, &doc);
        let err = match deadcode::run(&dir, None, &common::core_bin()) {
            Err(e) => format!("{e:#}"),
            Ok(_) => panic!("{tag}: judged on a declaration it cannot honour"),
        };
        assert!(
            err.contains(what) && err.contains(name),
            "{tag}: refused by name: {err}"
        );
    }
}
