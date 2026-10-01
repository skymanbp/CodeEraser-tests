//! The blind sample of the frozen suggestion set (plan v2.31 step 7):
//! 100 suggestions, the feasible and the infeasible half each, every
//! half apportioned over the corpora by their share of it (largest
//! remainder; a short half takes what there is), each corpus's seats
//! the lowest identity hashes. The audit is blind to the verdict and
//! not to the parameterisation (step-7 ruling 2): a sampled row carries
//! every member's identity, the lines of the run it sent, its source
//! (its identity's lines, cut at 120) and, per parameter the core
//! found, every member's text at it (`param_texts`) — an auditor cannot
//! judge a merge without the holes' text — and withholds the
//! feasibility, the reason, the savings and the member kept. The texts
//! list is as long as the core's count, so the auditor sees the core's
//! parameterisation and may reject it: the count the audit gives is
//! read after seeing the core's.

use super::member::{identity, members_of};
use crate::eval_support::{identity_hash, largest_remainder};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

pub const SAMPLE_SCHEMA: &str = "ce.eval-merge-sample/1.0.0";
pub const SAMPLE: &str = super::EXAM.sample;
/// The ids' hash domain, the first generation's in every generation: a
/// question is its id, so a member set sampled by two generations is
/// one question and the two samples compare by id.
pub const DOMAIN: &str = "merge-sample-v1";
pub const HALF: u64 = 50;
const TEXT_LINES: usize = 120;

/// A frozen row's sample id: the hash of its corpus and its members'
/// identities (`at`; the run is not identity).
pub fn id_of(corpus: &str, members: &Value) -> String {
    let ats = members.as_array().expect("members").iter();
    let joined: Option<Vec<&str>> = ats.map(|m| m["at"].as_str()).collect();
    let joined = joined.expect("every member names its `at`");
    identity_hash(
        DOMAIN,
        &json!({"corpus": corpus, "members": joined.join(" ")}),
        &["corpus", "members"],
    )
}

/// The drawn (corpus, row) pairs, feasible half first, each half by id.
pub fn draw(frozen: &Value) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for feasible in [true, false] {
        let mut pools: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
        for c in frozen["corpora"].as_array().expect("corpora") {
            let name = c["name"].as_str().expect("name").to_string();
            for row in c["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .filter(|r| r["feasible"] == feasible)
            {
                pools
                    .entry(name.clone())
                    .or_default()
                    .push((id_of(&name, &row["members"]), row.clone()));
            }
        }
        let weights = pools
            .iter()
            .map(|(k, v)| (k.clone(), v.len() as u64))
            .collect();
        let total: u64 = pools.values().map(|v| v.len() as u64).sum();
        let seats = largest_remainder(&weights, HALF.min(total));
        let mut half = Vec::new();
        for (name, mut pool) in pools {
            pool.sort_by(|a, b| a.0.cmp(&b.0));
            half.extend(
                pool.into_iter()
                    .take(seats[&name] as usize)
                    .map(|(id, row)| (id, (name.clone(), row))),
            );
        }
        half.sort_by(|a, b| a.0.cmp(&b.0));
        out.extend(half.into_iter().map(|(_, pair)| pair));
    }
    out
}

/// One sampled row as the auditor reads it, from the corpus's merge
/// document and tree.
pub fn audit_row(corpus: &str, row: &Value, doc: &Value, root: &Path) -> Value {
    let group = doc["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .find(|g| json!(members_of(g)) == row["members"])
        .unwrap_or_else(|| panic!("{corpus}: a frozen row the document does not hold"));
    let members: Vec<Value> = group["members"]
        .as_array()
        .expect("members")
        .iter()
        .map(|m| json!({"at": identity(m), "run": m["run"], "source": source(root, m)}))
        .collect();
    let params: Vec<Value> = group["holes"]
        .as_array()
        .expect("holes")
        .iter()
        .map(|p| {
            let texts: Vec<&Value> = p["values"]
                .as_array()
                .expect("values")
                .iter()
                .map(|v| &v["text"])
                .collect();
            json!(texts)
        })
        .collect();
    json!({
        "id": id_of(corpus, &row["members"]), "corpus": corpus,
        "family": row["family"], "fragment": row["fragment"],
        "members": members, "param_texts": params,
    })
}

/// A member's lines of source, cut at TEXT_LINES.
fn source(root: &Path, member: &Value) -> String {
    let path = member["path"].as_str().expect("path");
    let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
    let (s, e) = (
        member["lines"][0].as_u64().expect("start") as usize,
        member["lines"][1].as_u64().expect("end") as usize,
    );
    text.lines()
        .skip(s - 1)
        .take((e + 1 - s).min(TEXT_LINES))
        .collect::<Vec<_>>()
        .join("\n")
}
