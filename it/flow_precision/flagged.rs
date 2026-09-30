//! The product's answer to each reviewed question (booklet §5.5
//! 「精度册」): every file a question sits in, lowered at the pinned tip
//! and judged whole by the real core over one link, one request per
//! file (flow::wire::judge). A question maps back through the pool item
//! it was drawn as (pools.rs — the same anchor, never a second
//! derivation): kind 0 is flagged when an unreachable run covers its
//! statement, kind 1 when the core names its (write, variable), kinds
//! 2 / 3 when the core names its variable under that kind. A unit the
//! lowering left out, the core refused or that is dynamic answers
//! nothing (None) and is listed with its reason.

use crate::eval_flow_parts::FlowExam;
use crate::eval_flow_parts::draw::text;
use crate::eval_flow_parts::generate::lowered;
use crate::eval_flow_parts::pools::{self, Item};
use crate::eval_lang_parts::Generated;
use crate::eval_lang_parts::generate::{blob, corpus_repo};
use codeeraser::corelink::Link;
use codeeraser::flow::lower::Lowered;
use codeeraser::flow::wire::{self, Finding};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The product's answers, one per review row in its order, and the
/// units that answered nothing, `{corpus, path, unit, reason}` each.
pub struct Answers {
    pub flagged: Vec<Option<bool>>,
    pub unjudged: Vec<Value>,
}

/// One judged file: its lowering, the core's findings by unit nth and
/// the units it refused, with the reason.
struct Judged {
    done: Lowered,
    findings: BTreeMap<usize, Vec<Finding>>,
    refused: BTreeMap<usize, String>,
}

fn judge_file(link: &mut Link, done: Lowered, at: &str) -> Judged {
    let verdict = wire::judge(link, std::slice::from_ref(&done))
        .unwrap_or_else(|e| panic!("{at}: the core answered nothing: {e}"));
    let mut findings: BTreeMap<usize, Vec<Finding>> = BTreeMap::new();
    for (_, nth, f) in verdict.findings {
        findings.entry(nth).or_default().push(f);
    }
    let refused = verdict
        .refused
        .into_iter()
        .map(|(_, n, r)| (n, r))
        .collect();
    Judged {
        done,
        findings,
        refused,
    }
}

/// Whether the item's question is flagged by the unit's findings.
fn hit(item: &Item, findings: &[Finding]) -> bool {
    findings.iter().any(|f| {
        u64::from(f.kind) == item.kind
            && match item.kind {
                0 => f.seq <= item.seq && item.seq <= f.seq_end,
                1 => f.seq == item.seq && f.v == item.v,
                _ => f.v == item.v,
            }
    })
}

/// Whether a review row names this pool item.
fn names(row: &Value, item: &Item) -> bool {
    row["kind"].as_u64() == Some(item.kind)
        && text(row, "stratum") == item.stratum.to_string()
        && row["line"].as_u64() == Some(u64::from(item.line))
        && row["nth"].as_u64() == Some(item.nth)
        && text(row, "name") == item.name
}

/// One row's answer, or why its unit answered nothing. The row's unit
/// must be the sampled one (name and lines) and the question one of its
/// pool items: otherwise the sample was drawn from another lowering.
fn answer(row: &Value, file: &Judged, at: &str) -> Result<bool, String> {
    let nth = row["unit"].as_u64().expect("unit") as usize;
    if let Some(u) = file.done.unlowered.iter().find(|u| u.nth == nth) {
        return Err(format!("unlowered: {}", u.reason));
    }
    let unit = file.done.units.iter().find(|u| u.nth == nth);
    let unit = unit.unwrap_or_else(|| panic!("{at}: no unit {nth} in the lowering"));
    let lines = json!([unit.start_line, unit.end_line]);
    assert!(
        row["unit_name"] == json!(unit.name) && row["unit_lines"] == lines,
        "{at}: unit {nth} is not the sampled one - the sample and this lowering are not one tree"
    );
    if let Some(reason) = file.refused.get(&nth) {
        return Err(format!("refused by the core: {reason}"));
    }
    if unit.dynamic {
        return Err("dynamic: the core judges none of it".to_string());
    }
    let items = pools::items(unit);
    let item = items.iter().find(|i| names(row, i));
    let item = item.unwrap_or_else(|| panic!("{at}: no pool item of unit {nth} is this question"));
    Ok(hit(
        item,
        file.findings.get(&nth).map_or(&[], Vec::as_slice),
    ))
}

/// Every review row answered by the product at the exam's pinned tips.
pub fn answers(exam: &FlowExam, review: &Value, link: &mut Link) -> Answers {
    let rows = review["rows"].as_array().expect("rows");
    let lang = exam.language();
    let mut files: BTreeMap<(String, String), Judged> = BTreeMap::new();
    let mut out = Answers {
        flagged: Vec::new(),
        unjudged: Vec::new(),
    };
    for row in rows {
        let (corpus, path) = (text(row, "corpus"), text(row, "path"));
        let at = format!("{corpus}/{path}");
        let key = (corpus.to_string(), path.to_string());
        let file = files.entry(key).or_insert_with(|| {
            let tip = exam.tip(corpus).expect("a corpus of the exam");
            let source = blob(&corpus_repo(corpus, tip), tip, path);
            judge_file(link, lowered(&source, lang), &at)
        });
        match answer(row, file, &at) {
            Ok(flag) => out.flagged.push(Some(flag)),
            Err(reason) => {
                let why =
                    json!({"corpus": corpus, "path": path, "unit": row["unit"], "reason": reason});
                if !out.unjudged.contains(&why) {
                    out.unjudged.push(why);
                }
                out.flagged.push(None);
            }
        }
    }
    out
}
