//! Fixed cohorts for pairing; selected-result slices retain the ledger's semantics.
use super::data::Pool;
use super::ranking::Ranked;
use crate::similar_replay::{CORPORA, Measured};
use serde::Serialize;

#[derive(Serialize)]
pub struct Outcome {
    pub rank: String,
    pub corpus: String,
    pub stratum: bool,
    pub same: bool,
    pub clone: bool,
    pub role: bool,
    pub hit5: bool,
    pub possible: bool,
    pub top: String,
}

pub fn outcome(m: &Measured, p: &Pool<'_>, ranked: &[Ranked]) -> Outcome {
    let top = &ranked[0];
    let label = |doc| {
        p.candidates
            .iter()
            .find(|(i, _)| *i == doc)
            .expect("labelled seat")
            .1
    };
    let c = label(top.doc);
    let d = &m.corpus.docs[top.doc];
    Outcome {
        rank: p.row.rank.clone(),
        corpus: m.name.into(),
        stratum: p.row.role_bare,
        same: c.truth == "same_role",
        clone: c.clone,
        role: super::data::evidence(m, p.query, top.doc).role(),
        hit5: ranked
            .iter()
            .take(5)
            .any(|h| label(h.doc).truth == "same_role"),
        possible: p.candidates.iter().any(|(_, c)| c.truth == "same_role"),
        top: format!(
            "{}:{} {}#{}",
            d.path, d.bag.start_line, d.bag.key, d.bag.nth
        ),
    }
}

pub type Fraction = [usize; 2];

#[derive(Default, Serialize)]
pub struct Metric {
    pub p1: Fraction,
    pub fixed_role1: Fraction,
    pub fixed_role0: Fraction,
    pub selected_role1: Fraction,
    pub selected_role0: Fraction,
    pub nonclone: Fraction,
    pub fixed_nonclone: Fraction,
    pub hit5: Fraction,
    pub ceiling: Fraction,
    pub paired: Fraction,
    pub paired_role1: Fraction,
    pub paired_hit5: Fraction,
    pub top_changes: usize,
}

fn count(frac: &mut [usize; 2], eligible: bool, hit: bool) {
    if eligible {
        frac[0] += usize::from(hit);
        frac[1] += 1;
    }
}

fn paired(pair: &mut [usize; 2], now: bool, base: bool) {
    if now != base {
        pair[usize::from(!now)] += 1;
    }
}

pub fn metric(now: &[Outcome], base: &[Outcome], scope: &str) -> Metric {
    let mut m = Metric::default();
    assert_eq!(now.len(), base.len());
    for (n, b) in now.iter().zip(base) {
        assert_eq!(n.rank, b.rank, "paired cohorts must match");
        if !in_scope(&n.corpus, scope) {
            continue;
        }
        count(&mut m.p1, true, n.same);
        count(&mut m.fixed_role1, n.stratum, n.same);
        count(&mut m.fixed_role0, !n.stratum, n.same);
        count(&mut m.selected_role1, n.role, n.same);
        count(&mut m.selected_role0, !n.role, n.same);
        count(&mut m.nonclone, !n.clone, n.same);
        count(&mut m.fixed_nonclone, !b.clone, n.same);
        count(&mut m.hit5, true, n.hit5);
        count(&mut m.ceiling, true, n.possible);
        paired(&mut m.paired, n.same, b.same);
        if n.stratum {
            paired(&mut m.paired_role1, n.same, b.same);
        }
        paired(&mut m.paired_hit5, n.hit5, b.hit5);
        m.top_changes += usize::from(n.top != b.top);
    }
    m
}

pub fn scopes() -> Vec<String> {
    std::iter::once("all".to_string())
        .chain(CORPORA.iter().map(|(n, _)| n.to_string()))
        .chain(CORPORA.iter().map(|(n, _)| format!("without_{n}")))
        .collect()
}

pub fn in_scope(corpus: &str, scope: &str) -> bool {
    scope == "all" || scope == corpus || scope.strip_prefix("without_").is_some_and(|s| s != corpus)
}

pub fn significant(now: &[Outcome], base: &[Outcome]) -> bool {
    let all = metric(now, base, "all");
    let b = metric(base, base, "all");
    all.p1[0] >= b.p1[0] + 8
        && all.fixed_role1[0] >= b.fixed_role1[0] + 8
        && all.paired[0] >= 2 * all.paired[1]
        && all.paired_role1[0] >= 2 * all.paired_role1[1]
        && CORPORA
            .iter()
            .all(|(n, _)| metric(now, base, n).p1[0] >= metric(base, base, n).p1[0])
}
