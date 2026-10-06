//! The comparisons beside the sites: the C forced-include arcs (the
//! deadcode wire's unit → header arcs), the R package code, the cabal
//! mains and the Cargo crate roots (the declared-target pass's
//! `packages`, plan v2.33 W2-text stage B, `mains`, stage D, and
//! `crates`, stage F) and the Haskell and Rust files a cabal or a
//! Cargo.toml keeps private (the mounts table's `private`, stages D and
//! F), each answered by the frozen oracle and by the core over one file
//! set.

use super::common::Tally;
use crate::graph::ladder;
use crate::graph::oracle_cfg::cabal_mains;
use crate::graph::oracle_cfg::rust_targets::RustTargets;
use crate::graph::resolve::Manifests;
use crate::graph::{cabal, cargo, roots};
use crate::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The forced-include arcs both ways over one file set.
pub(super) fn compare_forced(
    root: &Path,
    files: &BTreeSet<String>,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let ids: BTreeMap<(&str, &str), usize> = files
        .iter()
        .enumerate()
        .map(|(i, f)| ((f.as_str(), ""), i))
        .collect();
    let (mut want, mut got) = (BTreeSet::new(), BTreeSet::new());
    ladder::frozen::forced_wire(root, files, &ids, &mut want);
    if let Err(e) = crate::graph::resolve::forced_wire(root, files, &ids, &mut got) {
        return tally.miss(format!("forced: core refused: {e}\n{}", ctx()));
    }
    tally.forced.0 += 1;
    tally.forced.1 += want.len();
    tally.agree("forced", &got, &want, ctx);
}

/// The R package code, the cabal mains and the Cargo crate roots both
/// ways over one file set, over the manifests the declared-target pass
/// finds: each R file's nearest DESCRIPTION, each walked directory's
/// nearest .cabal and nearest Cargo.toml.
pub(super) fn compare_declared(
    root: &Path,
    files: &BTreeSet<String>,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let descriptions: BTreeSet<String> = files
        .iter()
        .filter(|f| Lang::from_path(Path::new(f)) == Some(Lang::R))
        .filter_map(|f| roots::nearest_up(root, &roots::parent_dir(f), "DESCRIPTION"))
        .collect();
    let cabals: BTreeSet<String> = files
        .iter()
        .filter_map(|f| cabal::nearest(root, &roots::parent_dir(f)))
        .collect();
    let manifests: BTreeSet<String> = files
        .iter()
        .filter_map(|f| roots::nearest_up(root, &roots::parent_dir(f), "Cargo.toml"))
        .collect();
    let packages = ladder::frozen::packages(root, &descriptions, files);
    let mains = cabal_mains::mains(root, &cabals, files);
    let crates: BTreeSet<String> = manifests
        .iter()
        .filter_map(|m| cargo::package(root, m))
        .flat_map(|p| p.crate_roots(files))
        .collect();
    let found = Manifests {
        descriptions: &descriptions,
        cabals: &cabals,
        cargo: &manifests,
    };
    let got = match crate::graph::resolve::declared(root, files, &found) {
        Ok(got) => got,
        Err(e) => return tally.miss(format!("declared: core refused: {e}\n{}", ctx())),
    };
    tally.agree("packages", &got.packages, &packages, ctx);
    tally.agree("mains", &got.mains, &mains, ctx);
    tally.agree("crates", &got.crates, &crates, ctx);
    tally.packages.0 += 1;
    tally.packages.1 += packages.len();
    tally.mains.0 += 1;
    tally.mains.1 += mains.len();
    tally.crates.0 += 1;
    tally.crates.1 += crates.len();
}

/// The Haskell files their nearest cabal keeps private and the Rust files
/// their nearest Cargo.toml keeps private, both ways, every walked
/// Haskell and Rust file asked (the mounts table's bit 1).
pub(super) fn compare_private(
    root: &Path,
    files: &BTreeSet<String>,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let owners: BTreeMap<String, String> = files
        .iter()
        .filter(|f| Lang::judged_path(Path::new(f)) == Some(Lang::Haskell))
        .filter_map(|f| Some((f.clone(), cabal::nearest(root, &roots::parent_dir(f))?)))
        .collect();
    let rust: BTreeMap<String, String> = files
        .iter()
        .filter(|f| Lang::judged_path(Path::new(f)) == Some(Lang::Rust))
        .filter_map(|f| {
            let manifest = roots::nearest_up(root, &roots::parent_dir(f), "Cargo.toml")?;
            Some((f.clone(), manifest))
        })
        .collect();
    let mut want: BTreeSet<String> = owners
        .iter()
        .filter(|(f, c)| cabal::parse(root, c).is_some_and(|c| c.keeps_private(f)))
        .map(|(f, _)| f.clone())
        .collect();
    want.extend(
        rust.iter()
            .filter(|(f, m)| RustTargets::of(cargo::package(root, m), files).keeps(f))
            .map(|(f, _)| f.clone()),
    );
    let got = match crate::graph::resolve::private(root, files, &owners, &rust) {
        Ok(got) => got,
        Err(e) => return tally.miss(format!("private: core refused: {e}\n{}", ctx())),
    };
    tally.agree("private", &got, &want, ctx);
    tally.private.0 += owners.len() + rust.len();
    tally.private.1 += want.len();
}
