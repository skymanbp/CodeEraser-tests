//! The differential leg of plan v2.33 W1 item 3 for the query family's
//! directory tables: the core's tree form (CE.Query.Tree), sent through
//! the live `graph::tree` and `wire::body`, against the frozen
//! `tree_tables` (this module's parent, c2abca5d) — the `in_dir`, `dir`,
//! `parent` and `dir_name` rows read back as the answers of one question
//! per table the program reads, each directory's label and every name by
//! its hash — over three seeds of `diff_gen::cases()` node universes
//! (files, walked assets, sections, packages in and outside the tree,
//! prose files), and over the real trees in `CE_I3_REAL_ROOTS`. An
//! instrument: `CE_CORE_BIN=… cargo test --lib -- --ignored
//! query::facts::graph::frozen::diff`.

use super::tree_tables;
use crate::corelink::Link;
use crate::graph::wire::{GRAN_PACKAGE, GRAN_SECTION};
use crate::query::facts::graph::{self as live, Nodes};
use crate::query::facts::{Labels, Sink};
use crate::query::legend::{KIND_ASSET, KIND_PROSE};
use crate::structure::oracle::diff_gen::{self, Draw};
use crate::structure::oracle::diff_hold;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The four tables: name, schema code, the question that reads it back.
const TABLES: &str =
    "in_dir 2 in_dir(N, D)|dir 3 dir(D)|parent 4 parent(D, P)|dir_name 5 dir_name(D, N)";

fn tables() -> impl Iterator<Item = (&'static str, u32, &'static str)> {
    TABLES.split('|').map(|t| {
        let mut w = t.splitn(3, ' ');
        let (name, code) = (w.next().unwrap_or(""), w.next().unwrap_or(""));
        (name, code.parse().expect("a code"), w.next().unwrap_or(""))
    })
}

fn sink(wanted: &BTreeSet<u32>) -> Sink {
    Sink {
        codes: tables().map(|(n, c, _)| (n.to_string(), c)).collect(),
        wanted: wanted.clone(),
        tables: wanted.iter().map(|c| (*c, BTreeSet::new())).collect(),
    }
}

/// The frozen tables, labels and names.
fn expected(nodes: &Nodes, wanted: &BTreeSet<u32>) -> Value {
    let (mut s, mut labels) = (sink(wanted), Labels::default());
    tree_tables(nodes, &mut s, &mut labels);
    let names: Vec<(u64, String)> = labels.names.into_iter().collect();
    json!({"tables": s.finish(), "dirs": labels.dirs, "names": names})
}

/// The core's: one question per table read, its answers as the table.
fn answered(link: &mut Link, nodes: &Nodes, wanted: &BTreeSet<u32>) -> Value {
    let mut s = sink(wanted);
    let tree = live::tree(nodes, &mut s).expect("a directory table read");
    let read: Vec<(u32, &str)> = tables()
        .filter(|t| wanted.contains(&t.1))
        .map(|t| (t.1, t.2))
        .collect();
    let text: String = read.iter().map(|(_, q)| format!("?- {q}.\n")).collect();
    let body = crate::query::wire::body(
        &json!(["", null, text]),
        (&s.finish(), Some(&tree)),
        false,
        false,
    );
    let reply = link.request("query", body).expect("a query reply");
    let mut got: BTreeMap<u32, BTreeSet<Vec<u64>>> =
        read.iter().map(|(c, _)| (*c, BTreeSet::new())).collect();
    for row in reply["answers"].as_array().into_iter().flatten() {
        let row: Vec<u64> = row
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_u64)
            .collect();
        let code = read.get(row[0] as usize).map_or(u32::MAX, |r| r.0);
        got.entry(code).or_default().insert(row[1..].to_vec());
    }
    let (mut dirs, mut names) = (Vec::new(), BTreeMap::new());
    for d in reply["dirs"].as_array().into_iter().flatten() {
        dirs.push(d[0].clone());
        names
            .entry(d[2].as_u64().unwrap_or(0))
            .or_insert_with(|| d[1].clone());
    }
    let names: Vec<(u64, Value)> = names.into_iter().collect();
    json!({"tables": got, "dirs": dirs, "names": names})
}

/// A node universe: files, walked assets and prose files at drawn paths,
/// sections on them, packages at their directories, at the root, or at a
/// directory no file sits in.
fn nodes(d: &mut Draw) -> Nodes {
    let mut n = Nodes {
        labels: Vec::new(),
        kinds: Vec::new(),
        paths: Vec::new(),
        by_path: BTreeMap::new(),
    };
    for _ in 0..1 + d.under(40) {
        let kind = [
            0,
            0,
            0,
            KIND_ASSET,
            KIND_PROSE,
            GRAN_SECTION as usize,
            GRAN_PACKAGE as usize,
        ][d.under(7)];
        let near = n
            .paths
            .get(d.under(n.paths.len().max(1)))
            .cloned()
            .unwrap_or_default();
        let path = match kind {
            k if k == GRAN_SECTION as usize && !near.is_empty() => near,
            k if k == GRAN_PACKAGE as usize => match d.under(4) {
                0 => String::new(),
                1 => diff_gen::path(d),
                _ => near
                    .rsplit_once('/')
                    .map_or(String::new(), |(dir, _)| dir.to_string()),
            },
            _ => diff_gen::path(d),
        };
        n.labels.push(path.clone());
        n.kinds.push(kind);
        n.paths.push(path);
    }
    n
}

fn one(link: &mut Link, d: &mut Draw) -> (Value, Value) {
    let nodes = nodes(d);
    let mut wanted: BTreeSet<u32> = (2..=5).filter(|_| d.chance(50)).collect();
    if wanted.is_empty() {
        wanted.insert(2 + d.under(4) as u32);
    }
    (expected(&nodes, &wanted), answered(link, &nodes, &wanted))
}

#[test]
#[ignore = "instrument: the item-3 differential, run with a core"]
fn the_cores_directory_tables_answer_what_the_frozen_rust_built() {
    diff_hold::every_seed(one, false);
}

#[test]
#[ignore = "instrument: the item-3 differential over real trees"]
fn the_cores_directory_tables_answer_what_the_frozen_rust_built_on_real_trees() {
    let (_core, mut link, roots) = diff_hold::real_roots();
    let all: BTreeSet<u32> = (2..=5).collect();
    for root in &roots {
        let (_found, idx, db) = crate::dedup::snapshot(root, None).expect("a snapshot");
        let w =
            crate::graph::deadcode::wire_of(root, &idx, &db, crate::graph::deadcode::Advisory::No)
                .expect("a wire");
        let nodes = Nodes::build(&w, &idx).expect("the node universe");
        let (want, got) = (expected(&nodes, &all), answered(&mut link, &nodes, &all));
        println!(
            "{}: {} nodes, {} dirs, same {}",
            root.display(),
            nodes.len(),
            want["dirs"].as_array().map_or(0, Vec::len),
            want == got
        );
        assert_eq!(want, got, "{}", root.display());
    }
}
