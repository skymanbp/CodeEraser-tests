//! ce.toml is DECLARATIVE, so a declaration that silently does
//! nothing is exactly the failure this repo exists to fight. Two
//! such holes pinned here: a ce.toml glob the dialect would misread
//! (refused by name at the load throat since plan v2.18 step #14;
//! before that a Windows-spelled exclude compiled to a pattern
//! matching nothing), and a threshold ladder
//! one reader refused while the other judged with an unreachable
//! warn arm.

use crate::common;
use codeeraser::config::Config;
use codeeraser::scan::walk;
use std::path::Path;

/// Write `toml` as the fixture's ce.toml and load it — the one
/// load stanza both load-throat batteries share.
fn load_toml(dir: &Path, toml: &str) -> Result<Config, String> {
    std::fs::write(dir.join("ce.toml"), toml).expect("ce.toml");
    Config::load(dir)
}

/// ce.toml globs compile at the load throat in the ONE dialect (plan
/// v2.18 step #14, O42 + O20): '\\' is an escape there, never a
/// separator — the former `\\`→`/` rewrite turned an escaped
/// metacharacter into a directory pattern and hid the Windows
/// spelling that matched nothing — so a backslash refuses by name
/// with the fix in the message, a `#` (a comment) and a `!` refuse
/// likewise, and the three readers share the verdict. A class glob
/// `src/` now means the directory's files (it matched nothing).
#[test]
fn globs_compile_at_the_load_throat_in_one_dialect() {
    let fx = common::fixture(
        "config-contract-globs",
        &[
            ("src/gen/api.rs", "fn generated() {}\n"),
            ("src/keep.rs", "fn kept() {}\n"),
            ("a[1].rs", "fn bracketed() {}\n"),
            ("a1.rs", "fn plain() {}\n"),
        ],
    );
    let load = |toml: &str| load_toml(&fx.dir, toml);
    let kept = |cfg: &Config| -> Vec<String> {
        let mut v: Vec<String> = walk::collect(&fx.dir, &cfg.exclude)
            .expect("walk")
            .iter()
            .map(|w| walk::rel_str(&fx.dir, &w.path))
            .collect();
        v.sort();
        v
    };
    for (toml, names) in [
        ("exclude = [\"src\\\\gen\\\\*.rs\"]\n", "escape"),
        (
            "[[rules.class]]\nname = \"g\"\nglobs = [\"src\\\\gen\"]\n",
            "escape",
        ),
        ("[graph]\nentry_globs = [\"#root.ts\"]\n", "comment"),
        ("[graph]\nentry_globs = [\"!x.ts\"]\n", "'!'"),
    ] {
        let err = load(toml).expect_err(toml);
        assert!(err.contains(names), "{toml}: {err}");
    }
    let cfg = load("exclude = [\"src/gen/*.rs\"]\n").expect("the one spelling loads");
    assert_eq!(kept(&cfg), ["a1.rs", "a[1].rs", "ce.toml", "src/keep.rs"]);
    // a literal metacharacter is a class: `[[]` is one `[`, and the
    // plain sibling the misread pattern would have taken stays
    let cfg = load("exclude = [\"a[[]1].rs\"]\n").expect("class spelling loads");
    assert_eq!(
        kept(&cfg),
        ["a1.rs", "ce.toml", "src/gen/api.rs", "src/keep.rs"]
    );
    let cfg = load("[[rules.class]]\nname = \"src\"\nglobs = [\"src/\"]\n").expect("dir class");
    let classes =
        codeeraser::scan::classes::Classes::compile(&fx.dir, &cfg.rules).expect("compile");
    assert_eq!(
        classes.class_of("src/keep.rs"),
        1,
        "`src/` owns the directory's files"
    );
    assert_eq!(classes.class_of("a1.rs"), 0, "and nothing outside it");
}

/// One config, two readers: scan/wire.rs::grade_rows refused
/// `fail < warn` while the report.rs mirror judged on happily, so
/// `ce scan` exited 2 on a ce.toml the MCP scan tool served a full
/// report from — a report whose warn arm could never fire.
#[test]
fn incoherent_threshold_ladder_is_refused_by_both_readers() {
    let fx = common::fixture(
        "config-contract-ladder",
        &[
            ("ce.toml", "[thresholds]\nfile_lines_warn = 800\n"),
            ("src/a.rs", "fn a() {}\n"),
        ],
    );
    let err = Config::load(&fx.dir).expect_err("fail 750 sits below warn 800");
    assert!(err.contains("file_lines_warn"), "{err}");
    // the shared measurement walk (score/structure reuse) dies on
    // the same config, not just the wire path — measure() is where
    // every scan surface loads ce.toml (batch-7 slice 8)
    let Err(err) = codeeraser::scan::measure(&fx.dir) else {
        panic!("the measurement walk must refuse the same config the wire path does");
    };
    assert!(err.to_string().contains("file_lines_warn"), "{err}");
    // fail 0 = "no hard line" stays legal at any warn line
    std::fs::write(
        fx.dir.join("ce.toml"),
        "[thresholds]\nfile_lines_warn = 800\nfile_lines_fail = 0\n",
    )
    .expect("ce.toml");
    Config::load(&fx.dir).expect("no hard line is coherent");
}

/// The rulepack's load throat (plan v2.13 ①): a class ladder is
/// judged on its EFFECTIVE lines through the same predicate the
/// global table answers to (C8), the fence refuses a 65th class, a
/// twice-declared name refuses, and serde names a missing key — every
/// refusal loud, never a class that silently matches nothing.
#[test]
fn rulepack_classes_refuse_at_the_load_throat() {
    let fx = common::fixture("config-contract-classes", &[("src/a.rs", "fn a() {}\n")]);
    let load = |toml: &str| load_toml(&fx.dir, toml);
    let class = |name: &str, knobs: &str| {
        format!(
            "[[rules.class]]\nname = \"{name}\"\nglobs = [\"cli/tests/**\"]\n[rules.class.knobs]\n{knobs}\n"
        )
    };
    // C8: a class warn line above the INHERITED global hard line
    let err = load(&class("tests", "file_lines_warn = 800")).expect_err("800 > the global 750");
    assert!(
        err.contains("tests") && err.contains("file_lines_warn"),
        "{err}"
    );
    // the same class with its own hard line climbs, and loads
    let cfg = load(&class(
        "tests",
        "file_lines_warn = 800\nfile_lines_fail = 900",
    ))
    .expect("coherent");
    assert_eq!(
        cfg.rules.class[0]
            .effective(&cfg.thresholds)
            .file_lines_fail,
        900
    );
    let err = load(&(class("tests", "") + &class("tests", ""))).expect_err("declared twice");
    assert!(err.contains("declared twice"), "{err}");
    let err = load("[[rules.class]]\nglobs = [\"x/**\"]\n").expect_err("name is required");
    assert!(err.contains("name"), "{err}");
    let err = load("[[rules.class]]\nname = \"empty\"\nglobs = []\n").expect_err("no globs");
    assert!(err.contains("no globs"), "{err}");
    let many: String = (0..65).map(|i| class(&format!("c{i}"), "")).collect();
    let err = load(&many).expect_err("the fence");
    assert!(err.contains("fence"), "{err}");
    assert!(
        load(&many[..many.len() - class("c64", "").len()]).is_ok(),
        "64 classes load"
    );
}

/// O39: the digest a fixed declaration hashes to is a COMPATIBILITY
/// SURFACE — it sits in downstream repos' committed baselines and is
/// compared for equality across ce upgrades — so one literal is
/// frozen here (the GRAPH_REV / DOCDUP_REV discipline). Moving it is
/// a deliberate act with a CHANGELOG line and a named re-pin in every
/// fenced repo, never a side effect of a serialization change. The
/// rows pin the canonical form's laws through the real load throat:
/// comments and key order do not move it; a knob spelled at its
/// effective default is silence; a changed threshold, a changed class
/// glob and a changed exclude each move it and apart; a Windows
/// spelling of a glob is refused by name at load (O42), never a
/// second digest of the same intent.
#[test]
fn the_digest_of_a_fixed_declaration_is_frozen() {
    const FROZEN: u64 = 13_320_460_457_564_820_659;
    let fx = common::fixture("config-contract-digest", &[("src/a.rs", "fn a() {}\n")]);
    let digest = |toml: &str| load_toml(&fx.dir, toml).expect("loads").knobs_digest();
    // (text, what it hashes to): silent = the shipped default's
    // nothing, frozen = the literal, moved = something else
    let rows = [
        ("# nothing declared\n", "silent"),
        (
            "[trend]\nmin_points = 3\n[thresholds]\nfile_lines_warn = 300\n[graph]\nscc_floor = 2\n[score]\nviol_cost = 10\n",
            "silent",
        ),
        (
            "[dedup]\nbudget = 182\n\n[thresholds]\nfile_lines_warn = 250\n",
            "frozen",
        ),
        (
            "[thresholds]\n# spelled the other way round, commented\nfile_lines_warn = 250 # the line\n[dedup]\nbudget = 182\n[score]\ntol_abs = 10\n",
            "frozen",
        ),
        (
            "[dedup]\nbudget = 182\n[thresholds]\nfile_lines_warn = 251\n",
            "moved",
        ),
    ];
    for (i, (toml, want)) in rows.iter().enumerate() {
        let got = digest(toml);
        let ok = match *want {
            "silent" => got.is_none(),
            "frozen" => got == Some(FROZEN),
            _ => got.is_some() && got != Some(FROZEN),
        };
        assert!(ok, "row {i} ({want}): {got:?} for {toml:?}");
    }
    let classed = |glob: &str| {
        digest(&format!(
            "[[rules.class]]\nname = \"gen\"\nglobs = [\"{glob}\"]\n"
        ))
    };
    assert_ne!(
        classed("src/gen/**"),
        classed("src/gen/*"),
        "a class glob moves it"
    );
    assert_ne!(
        classed("src/gen/**"),
        digest("exclude = [\"src/gen/**\"]\n"),
        "and apart from an exclude"
    );
    let err = load_toml(&fx.dir, "exclude = [\"src\\\\gen\\\\*.rs\"]\n").expect_err("refused");
    assert!(
        err.contains("escape"),
        "a Windows spelling is refused by name, not fingerprinted: {err}"
    );
}
