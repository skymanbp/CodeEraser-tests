//! tsconfig `extends` habitats (split from graph_ladder.rs at the E01
//! hard line, step 5b): each case builds its own tree because the chain
//! shape IS the fixture — a cycle, an array, a diamond, a long chain, a
//! missing base, a non-string entry. Every broken chain is
//! config_depth, never a guess.

use codeeraser::graph::ladder::{self, Outcome, Reason};
use codeeraser::scan::lang::Lang;

use crate::common::{fixture, no, ok, run_cases, site};

/// An extends cycle must refuse the whole tsconfig rung — config
/// beyond the modeled chain is config_depth, never a guess.
#[test]
fn extends_cycle_is_config_depth() {
    let fx = fixture(
        "ladder-cycle",
        &[
            ("a.ts", "export {};\n"),
            ("tsconfig.json", "{\"extends\": \"./other.json\"}\n"),
            ("other.json", "{\"extends\": \"./tsconfig.json\"}\n"),
        ],
    );
    assert_eq!(
        ladder::resolve(
            Lang::TypeScript,
            &site("import", "a.ts", "anything", 1),
            &fx.scope()
        ),
        Outcome::Unresolved(Reason::ConfigDepth)
    );
}

/// The extends habitat: `main/` extends an array whose two entries
/// share one base (a diamond), `deep/` runs eleven configs deep, `bad/`
/// names a missing base and `bad2/` an entry that is no string.
fn extends_tree() -> Vec<(String, String)> {
    let mut tree: Vec<(String, String)> = [
        ("main/a.ts", "export {};\n"),
        (
            "main/tsconfig.json",
            "{\"extends\": [\"./cfg/one.json\", \"./cfg/two.json\"]}\n",
        ),
        (
            "main/cfg/one.json",
            "{\"extends\": \"./shared\", \"compilerOptions\": {\"paths\": {\"@x/*\": [\"../one/*\"]}}}\n",
        ),
        (
            "main/cfg/two.json",
            "{\"extends\": \"./shared\", \"compilerOptions\": {\"paths\": {\"@x/*\": [\"../two/*\"]}}}\n",
        ),
        (
            "main/cfg/shared.json",
            "{\"compilerOptions\": {\"baseUrl\": \"../lib\"}}\n",
        ),
        ("main/one/x.ts", "export {};\n"),
        ("main/two/x.ts", "export {};\n"),
        ("main/lib/leaf.ts", "export {};\n"),
        ("deep/a.ts", "export {};\n"),
        ("deep/tsconfig.json", "{\"extends\": \"./c1.json\"}\n"),
        ("deep/far/thing.ts", "export {};\n"),
        ("bad/a.ts", "export {};\n"),
        ("bad/tsconfig.json", "{\"extends\": \"./missing.json\"}\n"),
        ("bad2/a.ts", "export {};\n"),
        ("bad2/tsconfig.json", "{\"extends\": [\"./ok.json\", 5]}\n"),
        ("bad2/ok.json", "{}\n"),
    ]
    .map(|(p, b)| (p.to_string(), b.to_string()))
    .to_vec();
    // eleven configs deep: past the cap the walk used to stop at
    for n in 1..=10 {
        let body = if n == 10 {
            "{\"compilerOptions\": {\"paths\": {\"@far/*\": [\"./far/*\"]}}}\n".to_string()
        } else {
            format!("{{\"extends\": \"./c{}.json\"}}\n", n + 1)
        };
        tree.push((format!("deep/c{n}.json"), body));
    }
    tree
}

/// `extends` as TypeScript 5 reads it (step 5b): an array's later
/// entries override its earlier ones, so the last entry is the nearest
/// base; a diamond (two entries sharing a base) is no cycle; a chain
/// has no length cap; and a base that is missing, or an entry that is
/// no string, breaks the chain — config_depth, never a guess.
#[test]
fn extends_arrays_diamond_and_long_chains() {
    let tree = extends_tree();
    let borrowed: Vec<(&str, &str)> = tree.iter().map(|(p, b)| (p.as_str(), b.as_str())).collect();
    let fx = fixture("ladder-extends", &borrowed);
    let (ts, im) = (Lang::TypeScript, "import");
    run_cases(
        &fx,
        vec![
            (ts, im, "main/a.ts", "@x/x", ok("main/two/x.ts", 3)),
            (ts, im, "main/a.ts", "leaf", ok("main/lib/leaf.ts", 3)),
            (
                ts,
                im,
                "deep/a.ts",
                "@far/thing",
                ok("deep/far/thing.ts", 3),
            ),
            (ts, im, "bad/a.ts", "anything", no(Reason::ConfigDepth)),
            (ts, im, "bad2/a.ts", "anything", no(Reason::ConfigDepth)),
        ],
    );
}
