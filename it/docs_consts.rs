//! The how-page constant chips (`<ul class="consts">` per numbered
//! family on site/how ×2): the two languages carry the same families
//! and values, and every chip not on the allowlist resolves to exactly
//! one source constant with the same number. The source harvest lives
//! in docs_consts_parts; this half parses the pages and judges.

use crate::common::repo_root;
use crate::docs_consts_parts::{
    Def, defs_in, first_number, haskell_line, normalize, rust_line, value_for,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct Chip {
    name: String,
    value: String,
}

type Families = BTreeMap<String, Vec<Chip>>;

fn page(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn between<'a>(text: &'a str, start: usize, end: usize, what: &str) -> &'a str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("consts parser: invalid {what} range"))
}

/// Find `pat` at or after `from`, or panic naming the surface — the
/// ONE owner of the find-or-refuse idiom both parsers lean on.
fn seek(text: &str, from: usize, pat: &str, what: &str) -> usize {
    text[from..]
        .find(pat)
        .map(|i| from + i)
        .unwrap_or_else(|| panic!("{what} has no {pat}"))
}

fn parse_chips(body: &str, label: &str, family: &str) -> Vec<Chip> {
    let mut chips = Vec::new();
    let mut item = 0;
    let ctx = format!("{label} family {family}: chip");
    while let Some(rel) = body[item..].find("<li><b>") {
        let name_start = item + rel + "<li><b>".len();
        let name_end = seek(body, name_start, "</b>", &ctx);
        let value_start = name_end + "</b>".len();
        let value_end = seek(body, value_start, "</li>", &ctx);
        chips.push(Chip {
            name: between(body, name_start, name_end, "chip name").to_string(),
            value: between(body, value_start, value_end, "chip value")
                .trim()
                .to_string(),
        });
        item = value_end + "</li>".len();
    }
    chips
}

fn parse_page(text: &str, label: &str) -> Families {
    let mut families = BTreeMap::new();
    let mut cursor = 0;
    while let Some(rel) = text[cursor..].find(r#"<ul class="consts">"#) {
        let ul = cursor + rel;
        let end = seek(text, ul, "</ul>", &format!("{label}: consts block"));
        let before = &text[..ul];
        let heading = before
            .rfind(r#"<h3><span class="n">"#)
            .unwrap_or_else(|| panic!("{label}: consts block with no preceding numbered heading"));
        let n_start = heading + r#"<h3><span class="n">"#.len();
        let n_end = seek(
            text,
            n_start,
            "</span>",
            &format!("{label}: numbered heading"),
        );
        let family = between(text, n_start, n_end, "family number").to_string();
        assert!(
            !family.is_empty() && family.bytes().all(|b| b.is_ascii_digit()),
            "{label}: consts block heading is not numbered"
        );
        let body = between(
            text,
            ul + r#"<ul class="consts">"#.len(),
            end,
            "consts body",
        );
        assert!(
            families
                .insert(family.clone(), parse_chips(body, label, &family))
                .is_none(),
            "{label}: duplicate family {family}"
        );
        cursor = end + "</ul>".len();
    }
    families
}

fn allowlist() -> BTreeSet<&'static str> {
    // "since proto": the erase family's wire BIRTH version — owned by
    // VERSIONING.md's 2.16.0 entry, not by any source binding
    "kgram\nwindow\nguarantee t\nnear-miss band\nschema\nscale\nsizeHard H\n[softMin, softMax]\nsizeCeil fallback\nlegs cross at\ntsconfig extends\nprecision gate\nrewriteNum/Den\ntolNum/tolDen\nseamSoft S\nseamHard H\nseamPMax\nknob codes\ndestFloor\nrow + knob cap\nfull-scale grid\ndeclineFloorMicro\nfile_lines_warn S\nfile_lines_fail H\nzone_tiers\nFPR gate\nfeed schema\nwire\nminPoints\nsince proto"
        .lines()
        .collect()
}

/// Collision routing: (owning file, source binding) for the chip
/// names that resolve differently per family; None file = global.
fn collision<'a>(family: &str, name: &'a str) -> (Option<&'static str>, &'a str) {
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
    let allow = allowlist();
    for (family, chips) in families {
        for chip in chips {
            if allow.contains(chip.name.as_str()) {
                continue;
            }
            let matches = defs_for(&defs, family, &chip.name);
            assert_eq!(
                matches.len(),
                1,
                "family {family} chip {}: expected exactly one source constant, found {}",
                chip.name,
                matches.len()
            );
            let mut seen = BTreeSet::new();
            let source = value_for(matches[0], &defs, &mut seen).unwrap_or_else(|| {
                panic!(
                    "family {family} chip {}: source has no numeric value ({})",
                    chip.name, matches[0].value
                )
            });
            let documented = first_number(&chip.value).unwrap_or_else(|| {
                panic!(
                    "family {family} chip {}: value has no leading number ({})",
                    chip.name, chip.value
                )
            });
            assert_eq!(
                normalize(source),
                normalize(documented),
                "family {family} chip {}: source {} != documented {}",
                chip.name,
                matches[0].value,
                chip.value
            );
        }
    }
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
    let en = parse_page(&page(&root, "site/how/index.html"), "EN");
    let zh = parse_page(&page(&root, "site/zh/how/index.html"), "ZH");
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
