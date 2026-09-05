//! Boolean evidence expressions; recall here means recall inside the labelled pool only.
use super::config;
use super::data::{Evidence, Oracle};
use super::metrics;
use serde::Serialize;

pub struct Pair {
    pub corpus: String,
    pub same: bool,
    pub evidence: Evidence,
}

pub fn frozen(oracle: &Oracle) -> Vec<Pair> {
    oracle
        .rows
        .iter()
        .flat_map(|r| {
            r.candidates.iter().map(|c| Pair {
                corpus: r.corpus.clone(),
                same: c.truth == "same_role",
                evidence: Evidence {
                    hits: c.hits,
                    shape: c.shape_equal,
                    same_file: c.same_file,
                    query_names: r.bag["N"],
                },
            })
        })
        .collect()
}

fn predicate(expr: &str, e: &Evidence) -> bool {
    let mut stack = Vec::new();
    for token in expr.split(',') {
        let value = match token {
            "&" | "|" => {
                let (b, a) = (stack.pop().expect("rhs"), stack.pop().expect("lhs"));
                if token == "&" { a && b } else { a || b }
            }
            "!" => !stack.pop().expect("operand"),
            "p" => e.shape,
            "f" => e.same_file,
            "h" => 2 * e.hits[0] >= e.query_names,
            other => {
                let ch = match &other[..1] {
                    "n" => 0,
                    "c" => 2,
                    "d" => 3,
                    _ => panic!("atom"),
                };
                e.hits[ch] >= other[1..].parse::<u32>().expect("threshold")
            }
        };
        stack.push(value);
    }
    assert_eq!(stack.len(), 1);
    stack[0]
}

#[derive(Default, Serialize)]
pub struct Confusion {
    pub tp: usize,
    pub fp: usize,
    pub fn_: usize,
    pub tn: usize,
}

fn confusion(pairs: &[Pair], expression: &str, scope: &str) -> Confusion {
    let mut c = Confusion::default();
    for p in pairs.iter().filter(|p| metrics::in_scope(&p.corpus, scope)) {
        match (predicate(expression, &p.evidence), p.same) {
            (true, true) => c.tp += 1,
            (true, false) => c.fp += 1,
            (false, true) => c.fn_ += 1,
            (false, false) => c.tn += 1,
        }
    }
    c
}

pub fn table(pairs: &[Pair]) -> serde_json::Value {
    let rules = config::rules();
    let mut out = Vec::new();
    for (name, expression) in &rules {
        let rows: Vec<_> = metrics::scopes()
            .iter()
            .map(|scope| {
                let c = confusion(pairs, expression, scope);
                let b = confusion(pairs, rules[0].1, scope);
                let precision_up = c.tp * (b.tp + b.fp) > b.tp * (c.tp + c.fp);
                let recall_up = c.tp > b.tp;
                let pareto = c.tp >= b.tp
                    && c.tp * (b.tp + b.fp) >= b.tp * (c.tp + c.fp)
                    && (precision_up || recall_up);
                serde_json::json!({"scope": scope, "counts": c,
                "precision": [c.tp, c.tp + c.fp], "pool_recall": [c.tp, c.tp + c.fn_],
                "pareto_dominates_spec": pareto,
                "strictly_dominates_spec": precision_up && recall_up})
            })
            .collect();
        out.push(serde_json::json!({"name": name, "expression": expression, "slices": rows}));
    }
    serde_json::json!(out)
}
