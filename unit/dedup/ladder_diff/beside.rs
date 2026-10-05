//! The comparisons beside the sites: the C forced-include arcs (the
//! deadcode wire's unit → header arcs), the R package code and the cabal
//! mains (the declared-target pass's `packages`, plan v2.33 W2-text stage
//! B, and `mains`, stage D) and the Haskell files a cabal keeps private
//! (the mounts table's `private`, stage D), each answered by the frozen
//! oracle and by the core over one file set.

use super::common::Tally;
use crate::graph::ladder;
use crate::graph::oracle_cfg::cabal_mains;
use crate::graph::{cabal, roots};
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

/// The R package code and the cabal mains both ways over one file set,
/// over the manifests the declared-target pass finds: each R file's
/// nearest DESCRIPTION, each walked directory's nearest .cabal.
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
    let packages = ladder::frozen::packages(root, &descriptions, files);
    let mains = cabal_mains::mains(root, &cabals, files);
    let got = match crate::graph::resolve::declared(root, files, &descriptions, &cabals) {
        Ok(got) => got,
        Err(e) => return tally.miss(format!("declared: core refused: {e}\n{}", ctx())),
    };
    tally.agree("packages", &got.packages, &packages, ctx);
    tally.agree("mains", &got.mains, &mains, ctx);
    tally.packages.0 += 1;
    tally.packages.1 += packages.len();
    tally.mains.0 += 1;
    tally.mains.1 += mains.len();
}

/// The Haskell files their nearest cabal keeps private both ways, every
/// walked Haskell file asked (the mounts table's bit 1).
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
    let want: BTreeSet<String> = owners
        .iter()
        .filter(|(f, c)| cabal::parse(root, c).is_some_and(|c| c.keeps_private(f)))
        .map(|(f, _)| f.clone())
        .collect();
    let haskell: BTreeSet<String> = owners.keys().cloned().collect();
    let got = match crate::graph::resolve::private(root, &haskell, &owners) {
        Ok(got) => got,
        Err(e) => return tally.miss(format!("private: core refused: {e}\n{}", ctx())),
    };
    tally.agree("private", &got, &want, ctx);
    tally.private.0 += owners.len();
    tally.private.1 += want.len();
}
