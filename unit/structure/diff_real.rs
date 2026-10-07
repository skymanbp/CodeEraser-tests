//! The real-tree half of plan v2.33 W1 item 3's structure and arch
//! differential: every directory in `CE_I3_REAL_ROOTS` (`;`-separated,
//! each a committed copy — the leg refreshes its index), measured as `ce
//! structure --deep --days 30` and `ce arch --impact` measure it — the
//! walk's judged paths, the snapshot's clone blocks and graph wire, the
//! `[structure] layout`, the real line counts — then the core's built
//! tables held to the frozen readers (oracle/, c2abca5d) in the frozen
//! judge's order: staleness, redundancy, references, layout. An
//! instrument: `CE_CORE_BIN=… CE_I3_REAL_ROOTS=… cargo test --lib --
//! --ignored structure::oracle::diff_real`.

use crate::corelink::Link;
use crate::graph::deadcode::{Advisory, wire_of};
use crate::structure::oracle::diff_gen::{Case, plain};
use crate::structure::oracle::diff_rows::{answered, request, with};
use crate::structure::oracle::{diff_arch, diff_hold, rows, tree};
use crate::structure::rows as live;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

/// The frozen structure tables for a real root, in the frozen order.
fn frozen(
    link: &mut Link,
    root: &Path,
    core: &str,
    c: &Case,
    found: &crate::dedup::pairs::Blocks,
) -> Value {
    let t = tree::build(&c.walked);
    let optional = rows::stale_doc_rows(root, &c.w, &t, 30).and_then(|(docs, edges)| {
        let red = rows::redundancy_rows(root, core, &c.w, found, &t)?;
        Ok(json!({"staleDocRows": plain(docs), "staleEdgeRows": plain(edges), "redundancy": plain(red)}))
    });
    with(link, c, optional)
}

const KEYS: [&str; 8] = [
    "patternShapes",
    "conventions",
    "fileRefs",
    "dirEdges",
    "declared",
    "staleDocRows",
    "staleEdgeRows",
    "redundancy",
];

/// The live facts for the same root, answered by the core.
fn live(
    link: &mut Link,
    root: &Path,
    core: &str,
    c: &Case,
    found: &crate::dedup::pairs::Blocks,
) -> Value {
    let facts = live::stale_docs(root, &c.w, 30)
        .and_then(|s| Ok((s, live::redundancy(root, core, &c.w, found)?)));
    match facts {
        Err(e) => json!({"fault": e.to_string()}),
        Ok((stale, red)) => answered(link, &request(c, Some(stale), Some(red)), &KEYS),
    }
}

/// A root measured once: its case and clone blocks, and each measured
/// file's lines.
fn measured(root: &Path) -> Result<(Case, crate::dedup::pairs::Blocks, Vec<i64>)> {
    let (_config, files) = crate::scan::measure(root)?;
    let (found, idx, db) = crate::dedup::snapshot(root, None)?;
    let w = wire_of(root, &idx, &db, Advisory::No)?;
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let paths: Vec<String> = crate::graph::deadcode::measured_nodes(&w)
        .iter()
        .map(|m| m.1.to_string())
        .collect();
    let mut lines = Vec::new();
    for p in &paths {
        let bytes = crate::scan::walk::read_surviving(&root.join(p))?;
        lines.push(bytes.map_or(0, |b| crate::scan::metrics::size::total_lines(&b) as i64));
    }
    let c = Case {
        walked: crate::structure::judge::judged_paths(&files),
        layout: cfg.structure.layout.clone(),
        focus: paths.into_iter().take(2).collect(),
        w,
    };
    Ok((c, found, lines))
}

#[test]
#[ignore = "instrument: the item-3 differential over real trees"]
fn the_cores_tables_answer_what_the_frozen_rust_built_on_real_trees() {
    let (core, mut link, roots) = diff_hold::real_roots();
    let mut bad = Vec::new();
    for root in &roots {
        let (c, found, lines) = measured(root).expect("a measured root");
        let want = frozen(&mut link, root, &core, &c, &found);
        let got = live(&mut link, root, &core, &c, &found);
        let (awant, agot) = (
            diff_arch::expected(&c, &lines),
            diff_arch::answered(&mut link, &c, &lines),
        );
        let dirs = want["dirs"].as_array().map_or(0, Vec::len);
        println!(
            "{}: {} walked, {} dirs, structure {}, arch {}",
            root.display(),
            c.walked.len(),
            dirs,
            want == got,
            awant == agot
        );
        if want != got || awant != agot {
            bad.push(format!(
                "{}\n  want {want}\n  got  {got}\n  arch want {awant}\n  arch got  {agot}",
                root.display()
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
