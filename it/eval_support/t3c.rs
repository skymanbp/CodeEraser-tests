//! t3-candidates shared surface (M5-3c): the frozen constants both
//! the candidate instrument (eval_t3_candidates.rs) and the
//! sample's integrity gate (t3_sample_verifies in
//! eval_t3_precision.rs) read. The corpus resolver (Anchored)
//! retired with the one-shot generators (git history).

use serde_json::{Value, json};

/// Frozen constants of the t3-candidates family — the numeric knobs
/// echo the ONE product binding (the core's clone limits, LSH shape and
/// hot-group cap, off the package since plan v2.33 W3)
/// so doc and code cannot disagree. `min_reported_pairs` is the
/// per-corpus T-G14 output floor (anti-silence: "report almost
/// nothing" must red), set as HALF the scout run's identical-struct
/// pair count, rounded up: pairs whose two units carry byte-equal
/// structural facts (nodes, shingle set, kind histogram) are
/// near-certain 1.0-TSED clones, and the ×0.5 margin covers the
/// residual gap between equal pre-order facts and equal tree shape — a
/// derived bound with a stated hedge, not a guess (scout 2026-08-13:
/// identical pairs 105/126/35/1234/6506).
pub fn t3c_constants(corpus: &str) -> Value {
    use codeeraser::dedup::candidates as c;
    let l = c::limits();
    let lsh = &codeeraser::tables::get().limits.candidates;
    let floor: u64 = match corpus {
        "self" => 53,
        "cobra" => 63,
        "requests" => 18,
        "ripgrep" => 617,
        "zod" => 3253,
        other => panic!("no t3-candidates floor for {other}"),
    };
    json!({
        "t3_min_nodes": l.min_unit_nodes,
        "tsed_num": l.tsed_num,
        "tsed_den": l.tsed_den,
        "minhash_perms": lsh.lsh_perms,
        "lsh_bands": lsh.lsh_bands,
        "lsh_rows": lsh.lsh_rows,
        "hot_group_cap": codeeraser::tables::get().limits.dedup.hot_cap,
        "min_reported_pairs": floor,
    })
}
