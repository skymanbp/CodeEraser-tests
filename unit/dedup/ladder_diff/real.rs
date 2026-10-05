//! The real trees: each tree `CE_LADDER_DIFF_TREES` names (paths joined
//! by `;`) is walked and indexed into a fresh database the way a
//! `ce dedup` run does, and its edge sweep's sites — every site the
//! index holds — go to both sides through the store's own resolver
//! callback; the forced-include arcs, the R package code, the cabal mains
//! and the cabal privacy are compared over the walked set.

use super::beside::{compare_declared, compare_forced, compare_private};
use super::common::{Tally, compare, scratch};
use crate::config::Config;
use crate::dedup::{Params, index, walkidx};
use crate::graph::ladder::{Memo, Scope, Site};
use crate::graph::store::{CachedSite, kind_label};
use crate::graph::{resolve, wire};
use crate::scan::lang::Lang;
use std::path::Path;

/// One cached site as the dispatcher reads it, when the core holds its
/// language.
fn held(s: &CachedSite) -> Option<(Lang, Site<'_>)> {
    let lang = Lang::from_path(Path::new(&s.file))?;
    let kind = kind_label(s.kind)?;
    let line = usize::try_from(s.line).unwrap_or(1);
    let site = Site {
        kind,
        from: &s.file,
        spec: &s.spec,
        line,
    };
    resolve::in_core(lang).then_some((lang, site))
}

fn one_tree(root: &Path, db: &Path, tally: &mut Tally) {
    let config = Config::load(root).expect("config");
    let _ = std::fs::remove_file(db);
    let mut idx = index::Index::open(db, Params::default()).expect("index");
    let walked = walkidx::index_all(root, &config, &mut idx).expect("walk");
    let (m1, m2) = (Memo::default(), Memo::default());
    let scope = |memo| Scope {
        files: &walked.live,
        assets: &walked.assets,
        configs: &walked.configs,
        root,
        memo,
        crate_roots: &walked.crate_roots,
        search_roots: &walked.search_roots,
        java: &walked.java,
        lua: &walked.lua,
        includes: &walked.includes,
    };
    let (core, oracle) = (scope(&m1), scope(&m2));
    let ctx = || format!("  tree {}", root.display());
    let mut resolver = |sites: &[CachedSite]| {
        let batch: Vec<(Lang, Site)> = sites.iter().filter_map(held).collect();
        compare(&batch, &core, &oracle, tally, &ctx);
        wire::edges(sites, &core)
    };
    idx.ensure_edges_resolved(walked.resolve_key, &mut resolver)
        .expect("sweep");
    compare_forced(root, &walked.live, tally, &ctx);
    compare_declared(root, &walked.live, tally, &ctx);
    compare_private(root, &walked.live, tally, &ctx);
}

#[test]
#[ignore = "instrument: needs a core and CE_LADDER_DIFF_TREES; run with --ignored --nocapture"]
fn real_trees_agree() {
    let trees = std::env::var("CE_LADDER_DIFF_TREES").expect("CE_LADDER_DIFF_TREES");
    std::fs::create_dir_all(scratch()).expect("scratch");
    let mut tally = Tally::default();
    for (i, tree) in trees.split(';').filter(|t| !t.is_empty()).enumerate() {
        let before = tally.sites();
        one_tree(
            Path::new(tree),
            &scratch().join(format!("real-{i}.db")),
            &mut tally,
        );
        println!("tree {tree}: sites {}", tally.sites() - before);
    }
    tally.report("real trees");
}
