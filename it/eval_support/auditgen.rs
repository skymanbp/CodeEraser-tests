//! Shared audit-instrument machinery, second-family extraction (the
//! repo's own ratchet caught eval_docdup_audit re-growing
//! eval_t3_audit token for token — bite seventeen): ONE review-table
//! registry for every audit family, ONE domain-separated identity
//! hash and ONE frozen-sample envelope writer. The AuditFamily frame
//! walk and its coverage/tamper drivers retired with the one-shot
//! audit instruments (git history, 0c7c936 wave).

use serde_json::Value;
use std::collections::BTreeMap;

/// Every frozen review table of every audit family, mounted by name
/// (include_str! makes a missing corpus a compile error — a whole
/// corpus can never go silently blind, G10). Three families grew
/// three copies of this table before it became one registry.
pub const REVIEWS: [(&str, &str, &str); 15] = [
    (
        "graph",
        "cobra",
        include_str!("../eval_graph_review/cobra.json"),
    ),
    (
        "graph",
        "requests",
        include_str!("../eval_graph_review/requests.json"),
    ),
    (
        "graph",
        "ripgrep",
        include_str!("../eval_graph_review/ripgrep.json"),
    ),
    (
        "graph",
        "self",
        include_str!("../eval_graph_review/self.json"),
    ),
    (
        "graph",
        "zod",
        include_str!("../eval_graph_review/zod.json"),
    ),
    ("t3", "cobra", include_str!("../eval_t3_review/cobra.json")),
    (
        "t3",
        "requests",
        include_str!("../eval_t3_review/requests.json"),
    ),
    (
        "t3",
        "ripgrep",
        include_str!("../eval_t3_review/ripgrep.json"),
    ),
    ("t3", "self", include_str!("../eval_t3_review/self.json")),
    ("t3", "zod", include_str!("../eval_t3_review/zod.json")),
    (
        "docdup",
        "cobra",
        include_str!("../eval_docdup_review/cobra.json"),
    ),
    (
        "docdup",
        "requests",
        include_str!("../eval_docdup_review/requests.json"),
    ),
    (
        "docdup",
        "ripgrep",
        include_str!("../eval_docdup_review/ripgrep.json"),
    ),
    (
        "docdup",
        "self",
        include_str!("../eval_docdup_review/self.json"),
    ),
    (
        "docdup",
        "zod",
        include_str!("../eval_docdup_review/zod.json"),
    ),
];

/// One family's mounted ground truth for one corpus — a wrong name is
/// a loud panic, per family, as DATA.
pub fn review_of(family: &str, corpus: &str) -> Value {
    let text = REVIEWS
        .iter()
        .find(|(f, c, _)| *f == family && *c == corpus)
        .map(|(_, _, t)| *t)
        .unwrap_or_else(|| panic!("no {family} review table for {corpus}"));
    serde_json::from_str(text).unwrap_or_else(|e| panic!("{family}/{corpus}: {e}"))
}

/// Largest-remainder apportionment of `seats` over `weights`: each key
/// takes floor(seats·w/Σw), the seats left go to the largest
/// remainders, ties to the lower key — pure integer arithmetic, the
/// ONE apportion every stratified sample re-runs (t3, the language
/// exams). With seats ≤ Σw no key takes more than its weight; a zero
/// total leaves every key at 0.
pub fn largest_remainder(weights: &BTreeMap<String, u64>, seats: u64) -> BTreeMap<String, u64> {
    let total: u64 = weights.values().sum();
    let mut out: BTreeMap<String, u64> = weights.keys().map(|k| (k.clone(), 0)).collect();
    if total == 0 {
        return out;
    }
    let mut rems: Vec<(u64, &str)> = Vec::new();
    let mut used = 0;
    for (key, w) in weights {
        let share = seats * w;
        out.insert(key.clone(), share / total);
        used += share / total;
        rems.push((share % total, key.as_str()));
    }
    rems.sort_by(|x, y| (y.0, x.1).cmp(&(x.0, y.1)));
    for (_, key) in rems.iter().take((seats - used) as usize) {
        *out.get_mut(*key).expect("key") += 1;
    }
    out
}

/// Domain-separated identity hash: sha256("domain|f1|f2|…") over the
/// row's named fields in order, strings verbatim and integers in
/// decimal — the ONE derivation every sample generator and every
/// verify gate repeats (two families each grew their own).
pub fn identity_hash(domain: &str, row: &Value, fields: &[&str]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|k| match &row[*k] {
            Value::String(s) => s.clone(),
            v => v
                .as_i64()
                .unwrap_or_else(|| panic!("{k}: not a string or integer"))
                .to_string(),
        })
        .collect();
    super::content_sha(&format!("{domain}|{}", parts.join("|")))
}
