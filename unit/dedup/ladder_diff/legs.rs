//! The random-tree driver the legs share (common.rs re-exports it): a
//! seeded leg over a tree builder, one leg per rung from a language's
//! `LEGS` table, and the scratch root the instruments write under.

use super::beside::{compare_declared, compare_forced, compare_private};
use super::common::{Mark, Tally, Tree};
use super::rng::Rng;
use crate::graph::ladder::Site;
use crate::scan::lang::Lang;
use std::path::{Path, PathBuf};

/// A random leg: `CE_LADDER_DIFF_N` seeded trees, each built by `tree`
/// in a fresh directory, both sides on its sites (and on its forced-
/// include arcs when `forced`, on its R package code, cabal mains and
/// crate roots when it declares a DESCRIPTION, a .cabal or a Cargo.toml,
/// on its cabal and Cargo privacy when it declares a .cabal or a
/// Cargo.toml); the tally printed, any mismatch fails.
pub(super) fn leg(name: &str, salt: u64, forced: bool, tree: impl FnMut(&mut Rng, &Path) -> Tree) {
    run(name, salt, (forced, None), tree);
}

/// `leg` with a named divergence's mark (`Tally::marks`); the marked
/// mismatches it saw.
fn run(
    name: &str,
    salt: u64,
    (forced, mark): (bool, Option<Mark>),
    mut tree: impl FnMut(&mut Rng, &Path) -> Tree,
) -> usize {
    let mut rng = Rng::ladder(salt);
    let mut tally = Tally::default();
    tally.mark = mark;
    let n = std::env::var("CE_LADDER_DIFF_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);
    for i in 0..n {
        let root = scratch().join(format!("{name}-{i}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("scratch dir");
        let (world, owned) = tree(&mut rng, &root);
        let sites: Vec<(Lang, Site)> = owned
            .iter()
            .zip(0..)
            .map(|((lang, kind, from, spec), at)| {
                let line = world.lines.get(at).copied().unwrap_or(1);
                let site = Site {
                    kind,
                    from,
                    spec,
                    line,
                };
                (*lang, site)
            })
            .collect();
        world.judge(&root, &sites, &mut tally);
        let ctx = || world.describe(&root);
        if forced {
            compare_forced(&root, &world.files, &mut tally, &ctx);
        }
        let named = |suffix| world.configs.iter().any(|c| c.ends_with(suffix));
        if named("DESCRIPTION") || named(".cabal") || named("Cargo.toml") {
            compare_declared(&root, &world.files, &mut tally, &ctx);
        }
        if named(".cabal") || named("Cargo.toml") {
            compare_private(&root, &world.files, &mut tally, &ctx);
        }
    }
    tally.report(&format!("{name} random"));
    tally.marked.1
}

/// A language's legs, one per rung, from its `<lang> LEGS` table (rows
/// `name salt rung`): each a seeded leg over `tree` aimed at its rung.
pub(super) fn legs(lang: &str, tree: fn(&mut Rng, &Path, usize) -> Tree) {
    legs_marked(lang, tree, None);
}

/// `legs` under a named divergence's mark: across the legs, at least one
/// marked site must still mismatch, else the divergence's note is stale.
pub(super) fn legs_marked(
    lang: &str,
    tree: fn(&mut Rng, &Path, usize) -> Tree,
    mark: Option<Mark>,
) {
    let mut marked = 0;
    for row in super::tables::t(&format!("{lang} LEGS")).split('|') {
        let cells: Vec<&str> = row.split(' ').collect();
        let [name, salt, rung] = cells[..] else {
            panic!("a leg row: {row}");
        };
        let rung = rung.parse().expect("a rung");
        let salt = salt.parse().expect("a salt");
        marked += run(name, salt, (false, mark), |rng, root| tree(rng, root, rung));
    }
    assert!(
        mark.is_none() || marked > 0,
        "{lang}: no marked site diverges any more: the named divergence's note is stale"
    );
}

/// The scratch root the instruments write under.
pub(super) fn scratch() -> PathBuf {
    std::env::var("CE_LADDER_DIFF_SCRATCH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("ce-ladder-diff"))
}
