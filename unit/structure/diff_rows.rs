//! The differential leg of plan v2.33 W1 item 3 for the tree, the
//! always-on structure rows and the edge counts: the core's structure/1
//! path form (CE.Structure.Tree, CE.Structure.Raw), sent through the live
//! `rows::arcs` and `wire::request_body` with `inspect` set, against the
//! frozen tree.rs, edges.rs and rows.rs (oracle/, c2abca5d) — the node
//! rows, every directory's name, the shape and convention rows, the
//! reference split, the crossing dir-edge table, the declared layout, and
//! the fault naming a path the tree cannot place, and the core's refusal
//! of a layout its contract refuses (a weight 0, two keys naming one
//! directory: the frozen road sent those rows and the core refused them,
//! so the frozen half asks the same core the frozen request) — over three seeds of
//! `diff_gen::cases()` cases. An instrument (it needs a core):
//! `CE_CORE_BIN=… cargo test --lib -- --ignored structure::oracle::diff_rows`.

use crate::corelink::Link;
use crate::structure::oracle::diff_gen::{self, Case, Draw, plain};
use crate::structure::oracle::{diff_hold, rows, tree};
use crate::structure::{rows as live, wire};
use serde_json::{Value, json};

/// Each directory's path by id, the root `.` (the frozen document's
/// `names_by_id`).
pub fn names(t: &tree::Tree) -> Vec<String> {
    let mut names = vec![String::from("."); t.dirs.len()];
    for (path, &id) in &t.ids {
        if !path.is_empty() {
            names[id] = path.clone();
        }
    }
    names
}

/// The tables every request carries, as the frozen measuring side built
/// them in its order (the references, then the layout), and — with a
/// layout — what the core answered the integer request the frozen side
/// sent: its refusal, when it refused.
pub fn expected_over(link: &mut Link, c: &Case) -> Value {
    let v = expected(c);
    if v.get("fault").is_some() || c.layout.is_empty() {
        return v;
    }
    let mut body = json!({"nodes": v["tree"], "dirEdges": v["dirEdges"]});
    for k in ["patternShapes", "conventions", "fileRefs", "declared"] {
        body[k] = v[k].clone();
    }
    match link.request("structure", body) {
        Err(e) => json!({"refused": e}),
        Ok(_) => v,
    }
}

/// The frozen answer with an optional table: that table's own fault first
/// (the frozen judge read it before the references), then the common
/// tables with it.
pub fn with(link: &mut Link, c: &Case, optional: anyhow::Result<Value>) -> Value {
    let extra = match optional {
        Err(e) => return json!({"fault": e.to_string()}),
        Ok(extra) => extra,
    };
    let mut v = expected_over(link, c);
    if v.get("fault").is_none() && v.get("refused").is_none() {
        for (k, rows) in extra.as_object().expect("an object") {
            v[k] = rows.clone();
        }
    }
    v
}

/// The frozen tables alone (no layout refusal: the caller asks).
pub fn expected(c: &Case) -> Value {
    let t = tree::build(&c.walked);
    let tables = rows::ref_rows(&c.w, &t).and_then(|(refs, edges)| {
        let declared = rows::declared_rows(&c.layout, &t)?;
        Ok(json!({
            "tree": plain(rows::node_rows(&t)), "dirs": names(&t),
            "patternShapes": plain(rows::shape_rows(&t)), "conventions": plain(rows::convention_rows(&t)),
            "fileRefs": plain(refs), "dirEdges": plain(edges), "declared": plain(declared),
        }))
    });
    tables.unwrap_or_else(|e| json!({"fault": e.to_string()}))
}

/// The live request for a case, with the optional facts given.
pub fn request(
    c: &Case,
    stale: Option<live::StaleDocs>,
    redundancy: Option<live::Redundancy>,
) -> wire::Request {
    wire::Request {
        paths: c.walked.clone(),
        arcs: live::arcs(&c.w),
        layout: c.layout.iter().map(|(p, &w)| (p.clone(), w)).collect(),
        stale,
        redundancy,
        seams: None,
        knobs: Vec::new(),
    }
}

/// The core's answer to a request, `inspect` set: the fault, or `keys` of
/// the tables it built beside its tree and directory names.
pub fn answered(link: &mut Link, r: &wire::Request, keys: &[&str]) -> Value {
    let mut body = wire::request_body(r);
    body["inspect"] = json!(true);
    let reply = match link.request("structure", body) {
        Ok(reply) => reply,
        Err(e) => return json!({"refused": e}),
    };
    if let Some(fault) = reply["fault"].as_str() {
        return json!({"fault": fault});
    }
    let mut got = json!({"tree": reply["tree"], "dirs": reply["dirs"]});
    for k in keys {
        got[*k] = reply["built"][*k].clone();
    }
    got
}

const ROWS: [&str; 5] = [
    "patternShapes",
    "conventions",
    "fileRefs",
    "dirEdges",
    "declared",
];

fn one(link: &mut Link, d: &mut Draw) -> (Value, Value) {
    let c = diff_gen::case(d, None);
    (
        expected_over(link, &c),
        answered(link, &request(&c, None, None), &ROWS),
    )
}

#[test]
#[ignore = "instrument: the item-3 differential, run with a core"]
fn the_cores_tree_rows_and_edges_answer_what_the_frozen_rust_built() {
    diff_hold::every_seed(one, true);
}
