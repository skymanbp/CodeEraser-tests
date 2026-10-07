//! The differential leg of plan v2.33 W1 item 3 for the arch tables: the
//! core's arch/1 path form (CE.Arch.Tables), sent through the live
//! `rows::arcs` and `arch::wire::request_body`, against the frozen
//! tables.rs (../arch/oracle/tables.rs, c2abca5d) over the frozen tree —
//! the file, directory, reference and package tables, the focus, every
//! directory's path, and the fault naming an `--impact` path that is not a
//! measured file — over three seeds of `diff_gen::cases()` cases. An
//! instrument (it needs a core): `CE_CORE_BIN=… cargo test --lib --
//! --ignored structure::oracle::diff_arch`.

use crate::arch::wire::{self, Ask};
use crate::corelink::Link;
use crate::structure::oracle::diff_gen::{self, Case, Draw, plain};
use crate::structure::oracle::{diff_hold, tables, tree};
use serde_json::{Value, json};

/// The frozen face's tables over the frozen tree of the measured paths.
pub fn expected(c: &Case, lines: &[i64]) -> Value {
    let t = tree::build(&tables::measured_paths(&c.w));
    match tables::assemble(&c.w, &t, &c.focus, lines) {
        Err(e) => json!({"fault": e.to_string()}),
        Ok(t) => json!({
            "files": plain(t.files), "dirs": plain(t.dirs), "edges": plain(t.edges),
            "pkgEdges": plain(t.pkg_edges), "focus": plain(t.focus), "dirNames": t.dir_paths,
        }),
    }
}

/// The core's tables for the same case: the fault, or what it built.
pub fn answered(link: &mut Link, c: &Case, lines: &[i64]) -> Value {
    let ask = Ask {
        arcs: crate::structure::rows::arcs(&c.w),
        lines: lines.to_vec(),
        focus: c.focus.clone(),
    };
    let reply = link
        .request("arch", wire::request_body(&ask))
        .expect("an arch reply");
    if let Some(fault) = reply["fault"].as_str() {
        return json!({"fault": fault});
    }
    let mut got = reply["tables"].clone();
    got["dirNames"] = reply["dirNames"].clone();
    got
}

fn one(link: &mut Link, d: &mut Draw) -> (Value, Value) {
    let c = diff_gen::case(d, None);
    let n = crate::graph::deadcode::measured_nodes(&c.w).len();
    let lines: Vec<i64> = (0..n).map(|_| d.under(900) as i64).collect();
    (expected(&c, &lines), answered(link, &c, &lines))
}

#[test]
#[ignore = "instrument: the item-3 differential, run with a core"]
fn the_cores_arch_tables_answer_what_the_frozen_rust_built() {
    diff_hold::every_seed(one, true);
}
