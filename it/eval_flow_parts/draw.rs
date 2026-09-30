//! The per-cell hash-ranked draw of the v2.31 flow exams: each
//! (kind, stratum) cell takes the MIN_PER_STRATUM lowest site-domain
//! ranks of its pool (the whole cell when smaller), the rows stored in
//! audit-domain order — one reading for the generator (generate.rs)
//! and the sample verifier (verify.rs). No RNG, no clock, no backups.

use super::{DOMAINS, MIN_PER_STRATUM, cells, fields};
use crate::eval_lang_parts::draw::ranked_by;
use serde_json::Value;
use std::collections::BTreeMap;

/// One sample row: the pool row plus both domain hashes.
pub fn ranked(item: &Value) -> Value {
    ranked_by(item, DOMAINS, &fields())
}

/// A row's `kind/stratum` cell.
pub fn cell_of(row: &Value) -> String {
    let stratum = text(row, "stratum").chars().next().unwrap_or('?');
    super::cell(row["kind"].as_u64().expect("kind"), stratum)
}

pub fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().expect(key)
}

/// The frozen draw: the allocation (every cell, taken from the pool)
/// and the rows in audit order.
pub fn draw(pool: &[Value]) -> (BTreeMap<String, u64>, Vec<Value>) {
    let mut ranked: Vec<Value> = pool.iter().map(ranked).collect();
    ranked.sort_by_cached_key(|r| (cell_of(r), text(r, "rank").to_string()));
    let mut taken: BTreeMap<String, u64> = cells().into_iter().map(|c| (c, 0)).collect();
    let mut rows = Vec::new();
    for row in ranked {
        let n = taken.entry(cell_of(&row)).or_insert(0);
        if *n < MIN_PER_STRATUM {
            *n += 1;
            rows.push(row);
        }
    }
    rows.sort_by_cached_key(|r| text(r, "audit").to_string());
    (taken, rows)
}
