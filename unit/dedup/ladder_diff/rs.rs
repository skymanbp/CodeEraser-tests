//! Rust (plan v2.33 W2-text stage F): seeded trees of Cargo packages — a
//! root package or a workspace root, members under crates/ and tools/,
//! manifests with names (some shared, some hyphenated, some a toolchain
//! crate's), `[lib]` and `[[bin]]` paths, dependency tables, now and then
//! a manifest that does not read or one the walk does not list — their
//! source files at the conventional places (lib and bin roots, src/bin
//! targets, tests, a build script, modules flat, as mod.rs and nested)
//! and a few outside any package, some declared crate roots (`[graph]
//! crate_roots`), and five legs, one per rung: `mod` declarations (R1),
//! `crate::` paths (R2), `self` / `super` / local-module paths (R3),
//! crate names (R4) and the re-export surface (R5). The sites are the
//! detector's own, read off the written sources (graph::sites), each at
//! its line, a planted crate's among them three times in five; a few sources are on disk only, as a site's own file the
//! walk does not hold.

use super::common::{Tree, World, put};
use super::rng::Rng;
use super::rs_gen::{manifest, source, workspace};
use super::tables::t;
use crate::graph::roots::join_dir;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

/// The packages: each directory of the table a Cargo.toml with chance,
/// most of them walked; the root one a workspace now and then. Returns
/// the directories that hold one.
fn packages(rng: &mut Rng, root: &Path, world: &mut World) -> Vec<&'static str> {
    let mut dirs = Vec::new();
    for dir in t("rs PKG_DIRS").split('|') {
        if !rng.chance(if dir.is_empty() { 75 } else { 45 }) {
            continue;
        }
        let rel = join_dir(dir, "Cargo.toml");
        let text = if dir.is_empty() && rng.chance(25) {
            workspace()
        } else {
            manifest(rng)
        };
        put(root, &rel, &text);
        if rng.chance(90) {
            world.configs.push(rel);
        }
        dirs.push(dir);
    }
    dirs
}

/// The sources: three to ten conventional files under each package
/// directory (the root's too), a few loose ones.
fn sources(rng: &mut Rng, dirs: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for dir in std::iter::once("").chain(dirs.iter().copied()) {
        for _ in 0..3 + rng.below(8) {
            out.insert(join_dir(dir, rng.pick(t("rs FILES"))));
        }
    }
    for _ in 0..rng.below(3) {
        out.insert(rng.pick(t("rs LOOSE")).to_string());
    }
    out
}

/// A planted crate under one package directory, three times in five: a
/// lib root mounting modules that re-export one another (named `pub use`,
/// renames, globs, `pub extern crate`), a bin root and an integration
/// test using them — each file one of its table's texts, written with
/// chance. Returns the planted paths and their texts.
fn plant(rng: &mut Rng, dirs: &[&'static str]) -> Vec<(String, String)> {
    if !rng.chance(60) {
        return Vec::new();
    }
    let dir = if dirs.is_empty() {
        ""
    } else {
        dirs[rng.below(dirs.len())]
    };
    let mut out = Vec::new();
    for file in t("rs PLANTS").split('|') {
        if rng.chance(85) {
            let text = rng.pick(t(&format!("rs PLANT {file}"))).replace('~', "\n");
            out.push((join_dir(dir, file), text + "\n"));
        }
    }
    out
}

/// A tree: packages, a planted crate now and then, sources written
/// (most walked), declared crate roots now and then, and every site the
/// detector reads in the sources.
fn tree(rng: &mut Rng, root: &Path, rung: usize) -> Tree {
    let mut world = World::default();
    let dirs = packages(rng, root, &mut world);
    let planted = plant(rng, &dirs);
    let mut texts: Vec<(String, String)> = sources(rng, &dirs)
        .into_iter()
        .filter(|rel| planted.iter().all(|(p, _)| p != rel))
        .map(|rel| (rel, source(rng, rung)))
        .collect();
    texts.extend(planted);
    let mut owned = Vec::new();
    for (rel, text) in texts {
        put(root, &rel, &text);
        if rng.chance(92) {
            world.files.insert(rel.clone());
        }
        for s in crate::graph::sites::detect(&text, Lang::Rust) {
            world.lines.push(s.line);
            owned.push((Lang::Rust, s.kind, rel.clone(), s.spec));
        }
    }
    let walked: Vec<&String> = world.files.iter().collect();
    if rng.chance(25) && !walked.is_empty() {
        for _ in 0..1 + rng.below(2) {
            let pick = walked[rng.below(walked.len())].clone();
            world.crate_roots.insert(pick);
        }
    }
    (world, owned)
}

/// The five legs, one per rung (the `rs LEGS` table).
#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn rs_random_trees_agree() {
    super::common::legs("rs", tree);
}

/// The surface hash the resolve key folds (ladder/rs_cst.rs
/// `pubuse_hash`, rewritten over the surface fact's reading in stage F)
/// against the frozen one (oracle/rs_reexport.rs) over ten thousand
/// drawn sources, a fifth of them cut short: equal hashes, source by
/// source.
#[test]
#[ignore = "instrument: run with --ignored --nocapture"]
fn rs_surface_hash_agrees() {
    let mut rng = Rng::new(36);
    let mut misses = 0;
    for i in 0..10_000 {
        let mut text = source(&mut rng, i % 5);
        if rng.chance(20) {
            text = text.chars().take(rng.below(text.len().max(1))).collect();
        }
        let live = crate::graph::ladder::rs_cst::pubuse_hash(&text);
        if live != crate::graph::ladder::rs_reexport::pubuse_hash(&text) {
            misses += 1;
            if misses <= 5 {
                println!("MISMATCH surface hash:\n{text}");
            }
        }
    }
    println!("== ladder_diff rs surface hash: sources 10000 mismatches {misses} ==");
    assert_eq!(misses, 0, "the live and frozen surface hashes disagree");
}
