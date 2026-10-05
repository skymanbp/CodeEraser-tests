//! The comparisons beside the sites: the C forced-include arcs (the
//! deadcode wire's unit → header arcs) and the R package code (the
//! declared-target pass's `packages`, plan v2.33 W2-text stage B), each
//! answered by the frozen oracle and by the core over one file set.

use super::common::Tally;
use crate::graph::ladder;
use crate::graph::roots;
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
    if got != want {
        tally.miss(format!("forced: core {got:?} oracle {want:?}\n{}", ctx()));
    }
}

/// The R package code both ways over one file set, over the DESCRIPTIONs
/// the declared-target pass finds: each R file's nearest one.
pub(super) fn compare_packages(
    root: &Path,
    files: &BTreeSet<String>,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let manifests: BTreeSet<String> = files
        .iter()
        .filter(|f| Lang::from_path(Path::new(f)) == Some(Lang::R))
        .filter_map(|f| roots::nearest_up(root, &roots::parent_dir(f), "DESCRIPTION"))
        .collect();
    let want = ladder::frozen::packages(root, &manifests, files);
    let got = match crate::graph::resolve::packages(root, files, &manifests) {
        Ok(got) => got,
        Err(e) => return tally.miss(format!("packages: core refused: {e}\n{}", ctx())),
    };
    tally.packages.0 += 1;
    tally.packages.1 += want.len();
    if got != want {
        tally.miss(format!("packages: core {got:?} oracle {want:?}\n{}", ctx()));
    }
}
