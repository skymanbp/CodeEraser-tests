//! The how-page constant chips (`<ul class="consts">` per numbered
//! family on site/how and, from family 16, site/how/analysis, in
//! both languages): the two languages carry the same families
//! and values, and every chip not on the allowlist resolves to exactly
//! one source constant with the same number. The source harvest and
//! the page parser live in docs_consts_parts; this half binds and judges.

use crate::common::repo_root;
use crate::docs_consts_parts::page::{Families, families};
use crate::docs_consts_parts::{
    Def, default_impls_in, defs_in, first_number, haskell_line, normalize, numbers, package_defs,
    rust_line, value_for,
};
use std::collections::BTreeSet;
use std::path::Path;

/// Chips the gate cannot bind, each for a reason that is a PROPERTY
/// of the chip and not of the parser (v2.24 emptied the label cases —
/// see `label_binding`; v2.25 bound the six that named a constant
/// under a display name: `kgram` / `window` are `Params` defaults,
/// `scale` is `structScale`, `row + knob cap` is `trendRowCap`, `feed
/// schema` is `OBSERVE_SCHEMA`, and each family's `schema` chip is
/// that family's `SCHEMA_ID`, routed by file in `collision`). Three
/// kinds and nothing else:
///
/// - prose or external facts with no source constant at all:
///   `guarantee t`, `near-miss band`, `legs cross at`, `tsconfig
///   extends`, `precision gate`, `knob codes`, `full-scale grid`,
///   `zone_tiers`, `FPR gate`, `wire`, and `since proto` — the erase
///   family's wire BIRTH version, owned by VERSIONING.md's 2.16.0
///   entry rather than by any binding;
/// - ratio chips naming a pair of constants through a slash rather
///   than a value: `rewriteNum/Den`, `tolNum/tolDen`;
/// - values that are not literals: `destFloor` is derived (`go 1`),
///   and `minPoints` / `declineFloorMicro` live as rows of
///   `CE.Trend.Cost.knobTable`, not as named bindings.
fn allowlist() -> BTreeSet<&'static str> {
    "guarantee t\nnear-miss band\nlegs cross at\ntsconfig extends\nprecision gate\nrewriteNum/Den\ntolNum/tolDen\nknob codes\ndestFloor\nfull-scale grid\ndeclineFloorMicro\nzone_tiers\nFPR gate\nwire\nminPoints\nsince proto"
        .lines()
        .collect()
}

/// Chip label → the source constant(s) it names. A chip's label is a
/// DISPLAY name and a constant's is a source name; the gate assumed
/// they were one vocabulary, so every chip whose label adds the role
/// letter the docs call it by (`sizeHard H`) resolved to nothing and
/// was allowlisted to keep the gate green — leaving the hard line
/// printed on four surfaces with no executor at all, the same
/// lying-generator class the v1.4.1 audit chased. Absent = the label
/// IS the binding, which is the ordinary case.
fn label_binding(name: &str) -> Vec<&str> {
    match name {
        "sizeHard H" => vec!["sizeHard"],
        "seamHard H" => vec!["seamHard"],
        "seamSoft S" => vec!["seamSoft"],
        "sizeCeil fallback" => vec!["sizeCeil"],
        "[softMin, softMax]" => vec!["softMin", "softMax"],
        "file_lines_warn S" => vec!["Thresholds::file_lines_warn"],
        "file_lines_fail H" => vec!["Thresholds::file_lines_fail"],
        // v2.25: display names over a source constant (each was
        // allowlisted as "prose" while a binding sat one hop away)
        "kgram" => vec!["Params::kgram"],
        "window" => vec!["Params::window"],
        "scale" => vec!["structScale"],
        "row + knob cap" => vec!["trendRowCap"],
        "feed schema" => vec!["OBSERVE_SCHEMA"],
        _ => vec![name],
    }
}

/// Every family's `schema` chip is its own report's schema id; the
/// name is shared across the tree, so the file decides — one row per
/// family, `<family> <file> [binding]`, the binding SCHEMA_ID unless
/// named (a table, not a match arm per family: the arms had taken
/// `collision` past the cyclomatic line). The documents the core lays
/// out (plan v2.32 steps 3 and 4) bind theirs in the core.
const SCHEMA_FILES: &str = "\
01 cli/src/dedup/mod.rs
02 cli/src/dedup/t3/mod.rs
10 cli/src/trend/report.rs
07 core/app/CE/Join/Document.hs schemaId
16 core/app/CE/Query/Document.hs querySchemaId
17 core/app/CE/Flow/Document.hs schemaId
18 core/app/CE/Merge/Document.hs schemaId
19 core/app/CE/Arch/Document.hs schemaId";

/// Collision routing: (owning file, source binding) for the chip
/// names that resolve differently per family; None file = global.
fn collision<'a>(family: &str, name: &'a str) -> (Option<&'static str>, &'a str) {
    if name == "schema" {
        let row = SCHEMA_FILES
            .lines()
            .find_map(|row| row.strip_prefix(family)?.strip_prefix(' '));
        if let Some(row) = row {
            let (file, binding) = row.split_once(' ').unwrap_or((row, "SCHEMA_ID"));
            return (Some(file), binding);
        }
    }
    match (family, name) {
        ("04", "violCost") => (Some("core/app/CE/Structure/Cost.hs"), "structViolCost"),
        ("05", "violCost") => (Some("core/app/CE/Verdict/Cost.hs"), name),
        ("05" | "06", "sccFloor") => (Some("core/app/CE/Graph/Cost.hs"), name),
        ("06" | "07", "entryMask") => (Some("core/app/CE/Graph/Cost.hs"), name),
        ("12", "classes") => (Some("cli/src/erase/model.rs"), "CLASS_NAMES.len"),
        ("12", "reason codes") => (Some("cli/src/erase/model.rs"), "REASON_NAMES.len"),
        _ => (None, name),
    }
}

fn defs_for<'a>(defs: &'a [Def], family: &str, name: &str) -> Vec<&'a Def> {
    let (mapped, wanted) = collision(family, name);
    defs.iter()
        .filter(|d| d.name == wanted)
        .filter(|d| mapped.is_none_or(|p| d.file.to_string_lossy().replace('\\', "/").ends_with(p)))
        .collect()
}

fn assert_sources(root: &Path, families: &Families) {
    let mut defs = defs_in(root, "cli/src", "rs", rust_line);
    defs.extend(defs_in(root, "core/app", "hs", haskell_line));
    defs.extend(default_impls_in(root, "cli/src"));
    defs.extend(package_defs());
    let allow = allowlist();
    for (family, chips) in families {
        for chip in chips {
            if allow.contains(chip.name.as_str()) {
                continue;
            }
            let wanted = label_binding(&chip.name);
            let documented = numbers(&chip.value);
            assert!(
                documented.len() >= wanted.len(),
                "family {family} chip {}: names {} constants but shows {} numbers ({})",
                chip.name,
                wanted.len(),
                documented.len(),
                chip.value
            );
            for (want, shown) in wanted.iter().zip(&documented) {
                assert_source(&defs, family, &chip.name, want, shown);
            }
        }
    }
}

/// One (chip half → source constant) binding: exactly one definition,
/// a numeric reading of it, and the number the page shows.
fn assert_source(defs: &[Def], family: &str, chip: &str, want: &str, shown: &str) {
    let matches = defs_for(defs, family, want);
    assert_eq!(
        matches.len(),
        1,
        "family {family} chip {chip}: expected exactly one source constant for {want}, found {}",
        matches.len()
    );
    let mut seen = BTreeSet::new();
    let source = value_for(matches[0], defs, &mut seen).unwrap_or_else(|| {
        panic!(
            "family {family} chip {chip}: {want} has no numeric value ({})",
            matches[0].value
        )
    });
    assert_eq!(
        normalize(source),
        normalize(shown.to_string()),
        "family {family} chip {chip}: source {} ({want}) != documented {shown}",
        matches[0].value
    );
}

/// K46: the producer's cut of the `unmentioned` table and the core's
/// soft cap are ONE number, pinned source to source (neither is a
/// page chip): a smaller Rust value would truncate silently, a larger
/// one would resurrect a table the core drops.
#[test]
fn the_unmentioned_soft_cap_is_one_number_on_both_sides() {
    let root = repo_root();
    let one = |defs: Vec<Def>, name: &str| {
        let found: Vec<&Def> = defs.iter().filter(|d| d.name == name).collect();
        assert_eq!(found.len(), 1, "{name}: exactly one definition");
        normalize(first_number(&found[0].value).expect("numeric value"))
    };
    assert_eq!(
        one(
            defs_in(&root, "cli/src", "rs", rust_line),
            "UNMENTIONED_SOFT_CAP"
        ),
        one(
            defs_in(&root, "core/app", "hs", haskell_line),
            "unmentionedCap"
        )
    );
}

#[test]
fn how_page_constant_chips_are_locked_and_resolvable() {
    let root = repo_root();
    let en = families(&root, false);
    let zh = families(&root, true);
    assert_eq!(
        en.keys().collect::<BTreeSet<_>>(),
        zh.keys().collect::<BTreeSet<_>>()
    );
    for family in en.keys() {
        assert_eq!(
            en[family].len(),
            zh[family].len(),
            "family {family}: chip count drift"
        );
        for (i, (a, b)) in en[family].iter().zip(&zh[family]).enumerate() {
            // the VALUE is the leading numeric token; trailing prose is translated
            assert!(
                first_number(&a.value) == first_number(&b.value),
                "family {family} chip {i}: value drift ({} vs {})",
                a.value,
                b.value
            );
        }
    }
    let total: usize = en.values().map(Vec::len).sum();
    assert!(total > 80, "only {total} chips harvested; parser is broken");
    assert_sources(&root, &en);
}
