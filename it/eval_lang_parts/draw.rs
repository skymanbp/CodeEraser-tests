//! The per-language hash-ranked draw of the v2.30 language exams (split
//! from mod.rs, which keeps the exam table and the pre-registered
//! constants the draw reads): the kind quotas, a pool row's two
//! domain hashes and the frozen draw itself — one reading for the
//! generator (generate.rs) and the sample verifier (verify.rs).

use super::{AUDIT_DOMAIN, BACKUP_PER_KIND, FIELDS, MIN_PER_KIND, SITE_DOMAIN, TOTAL};
use crate::eval_support::{identity_hash, largest_remainder};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Kind quotas from the pool's per-kind counts: each kind first takes
/// min(MIN_PER_KIND, pool), the seats left go by largest remainder
/// over what each kind has left, so no quota exceeds its pool.
pub fn quotas(pool: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let floor: BTreeMap<String, u64> = pool
        .iter()
        .map(|(k, n)| (k.clone(), (*n).min(MIN_PER_KIND)))
        .collect();
    let rest: BTreeMap<String, u64> = pool
        .iter()
        .map(|(k, n)| (k.clone(), n - floor[k]))
        .collect();
    let seats = TOTAL
        .saturating_sub(floor.values().sum())
        .min(rest.values().sum());
    let extra = largest_remainder(&rest, seats);
    floor
        .into_iter()
        .map(|(k, f)| {
            let e = extra[&k];
            (k, f + e)
        })
        .collect()
}

/// One sample row: the pool row plus both domain hashes.
pub fn ranked(site: &Value) -> Value {
    let mut row = site.clone();
    row["rank"] = json!(identity_hash(SITE_DOMAIN, site, &FIELDS));
    row["audit"] = json!(identity_hash(AUDIT_DOMAIN, site, &FIELDS));
    row
}

/// The frozen draw.
pub struct Draw {
    pub allocation: BTreeMap<String, u64>,
    pub primary: Vec<Value>,
    pub backups: Vec<Value>,
}

fn by_hash<'a>(row: &'a Value, domain: &str) -> &'a str {
    row[domain].as_str().expect("hash")
}

/// Within each kind the site-domain rank picks the quota; the primaries
/// are then stored in audit-domain order (the auditor never sees rank
/// order), and each kind's backups are the audit-domain rank over its
/// unpicked rest, stored kind by kind.
pub fn draw(pool: &[Value]) -> Draw {
    let mut by_kind: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for site in pool {
        let kind = site["kind"].as_str().expect("kind").to_string();
        by_kind.entry(kind).or_default().push(ranked(site));
    }
    let counts = by_kind
        .iter()
        .map(|(k, v)| (k.clone(), v.len() as u64))
        .collect();
    let allocation = quotas(&counts);
    let (mut primary, mut backups) = (Vec::new(), Vec::new());
    for (kind, mut rows) in by_kind {
        rows.sort_by(|a, b| by_hash(a, "rank").cmp(by_hash(b, "rank")));
        let mut rest = rows.split_off(allocation[&kind] as usize);
        rest.sort_by(|a, b| by_hash(a, "audit").cmp(by_hash(b, "audit")));
        primary.extend(rows);
        backups.extend(rest.into_iter().take(BACKUP_PER_KIND as usize));
    }
    primary.sort_by(|a, b| by_hash(a, "audit").cmp(by_hash(b, "audit")));
    Draw {
        allocation,
        primary,
        backups,
    }
}
