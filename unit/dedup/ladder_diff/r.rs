//! R (plan v2.33 W2-text stage B): seeded trees with R files under
//! package `R/` directories and beside scripts, DESCRIPTIONs drawn as text
//! (a byte-order mark, CRLF, `Packaged:` before `Package:`, a name on a
//! continuation line, two words, none at all, a `Collate` list over
//! continuation lines with quoted and bare names) and a listed one that is
//! not on disk, declared `r` roots, and two legs — one per rung family:
//! `source` paths (beside, under the root, under the declared roots,
//! `..`, rooted, drive and URL forms) and `library` names (a carried
//! package, two DESCRIPTIONs naming one, none). Every tree's R package
//! code is compared too (beside.rs).

use super::common::{Owned, Tree, World, leg};
use super::rng::Rng;
use super::tables::t;
use crate::graph::roots::parent_dir;
use crate::scan::lang::Lang;
use std::path::Path;

// fixture: the rooted spellings (absolute, home-relative, drive, colon)
// the source rung must refuse — specifier text, never a path this test opens
const ROOTED: &str = "/x.R|\u{7e}/x.R|C:/x.R|a:b.R|/a/b.R";

/// One DESCRIPTION's text.
fn description(rng: &mut Rng) -> String {
    let mut lines: Vec<String> = Vec::new();
    if rng.chance(15) {
        lines.push("Packaged: 2024-01-01; builder".into());
    }
    let name = rng.pick(t("r PACKAGES"));
    match rng.below(10) {
        0 => {}
        1 => lines.extend(["Package:".to_string(), format!("  {name}")]),
        2 => lines.push(format!("Package: {name} two")),
        _ => lines.push(format!("Package: {name}")),
    }
    if rng.chance(70) {
        let mut head = "Collate:".to_string();
        for _ in 0..rng.below(5) {
            let name = rng.pick(t("r COLLATE"));
            let item = quoted(rng, name);
            if rng.chance(40) {
                head.push(' ');
                head.push_str(&item);
            } else {
                lines.push(std::mem::take(&mut head));
                head = format!("{}{item}", if rng.chance(70) { "    " } else { "\t" });
            }
        }
        lines.push(head);
    }
    lines.push("Version: 1.0".into());
    let mut text = lines.join(if rng.chance(30) { "\r\n" } else { "\n" });
    if rng.chance(5) {
        text.insert(0, '\u{feff}');
    }
    text
}

/// A Collate name bare, single- or double-quoted, or opening a quote it
/// never closes.
fn quoted(rng: &mut Rng, name: &str) -> String {
    match rng.below(10) {
        0..4 => name.to_string(),
        4..7 => format!("'{name}'"),
        7..9 => format!("\"{name}\""),
        _ => format!("'{name}"),
    }
}

/// What a source path is drawn against: the walked files and the
/// declared `r` roots.
struct Known {
    files: Vec<String>,
    roots: Vec<String>,
}

/// A source path: a URL, a rooted spelling, a walked file spelled from the
/// tree root, from the sourcing file's directory or from a declared root,
/// or pieces of a table joined by slashes.
fn source(rng: &mut Rng, from: &str, known: &Known) -> String {
    match rng.below(100) {
        0..5 => "https://example.org/x.R".to_string(),
        5..13 => rng.pick(ROOTED).to_string(),
        13..55 if !known.files.is_empty() => {
            let file = &known.files[rng.below(known.files.len())];
            let mut bases: Vec<String> = known.roots.clone();
            bases.push(parent_dir(from));
            let base = &bases[rng.below(bases.len())];
            let below = file.strip_prefix(&format!("{base}/"));
            below.map_or_else(|| file.clone(), str::to_string)
        }
        _ => {
            let pieces: Vec<&str> = (0..1 + rng.below(3))
                .map(|_| rng.pick(t("r SOURCE")))
                .collect();
            pieces.join("/")
        }
    }
}

/// A tree: its R files, DESCRIPTIONs at some of the package directories
/// (one listed and absent, sometimes), its declared roots, thirty sites
/// whose kinds `library_pct` weighs.
fn tree(rng: &mut Rng, root: &Path, library_pct: usize) -> Tree {
    let mut world = World::seeded(rng, (8, 25), t("r DIRS"), t("r BASES"));
    world.declare(rng, "r", t("r ROOTS"));
    world.manifests(
        (rng, root),
        (t("r DESC_DIRS"), "DESCRIPTION"),
        45,
        description,
    );
    let known = Known {
        files: world.files.iter().cloned().collect(),
        roots: world
            .search_roots
            .get("r")
            .into_iter()
            .flatten()
            .cloned()
            .collect(),
    };
    let keep = |f: &str| f.ends_with(".R") || f.ends_with(".r");
    let owned = world.sites(rng, keep, (t("r DIRS"), "zz.R"), |rng, from| {
        site(rng, from, library_pct, &known)
    });
    (world, owned)
}

fn site(rng: &mut Rng, from: String, library_pct: usize, known: &Known) -> Owned {
    let roll = rng.below(100);
    let (kind, spec) = if roll < library_pct {
        ("library", rng.pick(t("r PACKAGES")).to_string())
    } else if roll < 95 {
        ("source", source(rng, &from, known))
    } else {
        ("import", source(rng, &from, known))
    };
    (Lang::R, kind, from, spec)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn r_source_random_trees_agree() {
    leg("r-source", 5, false, |rng, root| tree(rng, root, 10));
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn r_library_random_trees_agree() {
    leg("r-library", 6, false, |rng, root| tree(rng, root, 80));
}
