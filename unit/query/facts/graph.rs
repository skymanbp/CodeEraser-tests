// Frozen (plan v2.33 W1 item 3; test-only): the directory tree the query family's
// in_dir / dir / parent / dir_name tables came from, lines 152-195 of
// cli/src/query/facts/graph.rs at c2abca5d, byte for byte below the imports; the
// imports are the ones those lines read, `tree` the frozen one, and the mount of
// the differential leg that holds the core to it (tree_diff.rs, a child here so it
// reads the frozen function). cli/src/query/facts/graph.rs mounts this module for
// tests only.
use crate::graph::wire::GRAN_PACKAGE;
use crate::query::facts::graph::Nodes;
use crate::query::facts::{Labels, Sink};
use crate::query::legend;
use crate::structure::oracle::tree;
#[path = "tree_diff.rs"]
mod diff;
/// The directory tree over every file-shaped and section path, a
/// package seated at its own directory; root is dir 0 and reads `.`.
fn tree_tables(nodes: &Nodes, sink: &mut Sink, labels: &mut Labels) {
    let file_paths: Vec<String> = nodes
        .kinds
        .iter()
        .zip(&nodes.paths)
        .filter(|(k, _)| **k != GRAN_PACKAGE as usize)
        .map(|(_, p)| p.clone())
        .collect();
    let mut t = tree::build(&file_paths);
    for (i, kind) in nodes.kinds.iter().enumerate() {
        let dir = if *kind == GRAN_PACKAGE as usize {
            tree::dir_id(&mut t, &nodes.paths[i])
        } else {
            tree::dir_of(&t, &nodes.paths[i]).unwrap_or(0)
        };
        sink.row("in_dir", vec![i as u64, dir as u64]);
    }
    let mut by_id: Vec<String> = vec![String::new(); t.dirs.len()];
    for (path, id) in &t.ids {
        by_id[*id] = path.clone();
    }
    for (id, dir) in t.dirs.iter().enumerate() {
        sink.row("dir", vec![id as u64]);
        if id != 0 {
            sink.row("parent", vec![id as u64, dir.parent as u64]);
        }
        let name = by_id[id]
            .rsplit('/')
            .next()
            .filter(|n| !n.is_empty())
            .unwrap_or(".");
        labels
            .names
            .entry(legend::sym(name))
            .or_insert_with(|| name.to_string());
        sink.row("dir_name", vec![id as u64, legend::sym(name)]);
    }
    labels.dirs = by_id
        .into_iter()
        .map(|p| if p.is_empty() { ".".to_string() } else { p })
        .collect();
}
