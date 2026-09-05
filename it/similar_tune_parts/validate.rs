//! Independent product replay checks: a tuning mirror must earn its baseline.
use super::data::{self, Pool};
use super::ranking::{Frame, Ranked};
use crate::similar_replay::Measured;

pub fn baseline(m: &Measured, p: &Pool<'_>, f: &Frame, ranks: &[Ranked]) {
    for r in ranks {
        assert_eq!(r.score, f.base[r.doc].unwrap_or(0), "BM25 mirror score");
    }
    let mut want: Vec<_> = ranks.iter().map(|r| r.doc).collect();
    want.sort_by(|a, b| {
        f.base[*b]
            .unwrap_or(0)
            .cmp(&f.base[*a].unwrap_or(0))
            .then_with(|| super::ranking::identity(m, *a).cmp(&super::ranking::identity(m, *b)))
    });
    assert_eq!(
        want,
        ranks.iter().map(|r| r.doc).collect::<Vec<_>>(),
        "identity tie order"
    );
    for (doc, c) in &p.candidates {
        let e = data::evidence(m, p.query, *doc);
        if m.name != "self" {
            assert_eq!(e.hits, c.hits, "fixture evidence");
            assert_eq!(e.shape, c.shape_equal, "fixture shape");
            for (frozen, live) in [
                (&c.bare, &m.ranked[p.query].0),
                (&c.widened, &m.ranked[p.query].1),
            ] {
                if let Some(placement) = frozen {
                    assert_eq!(live[placement.rank - 1].doc, *doc, "fixture rank");
                    assert_eq!(
                        live[placement.rank - 1].score,
                        placement.score,
                        "fixture score"
                    );
                } else {
                    assert!(live.iter().all(|h| h.doc != *doc));
                }
            }
        }
    }
}

pub fn widened(m: &Measured, query: usize, ranks: &[Ranked]) {
    let mut q = m.corpus.query_of(query);
    m.table.expand(&mut q);
    let live = m.corpus.top_k(&q, m.corpus.docs.len(), Some(query));
    for r in ranks {
        let want = live.iter().find(|h| h.doc == r.doc).map_or(0, |h| h.score);
        assert_eq!(r.score, i128::from(want), "PPMI mirror score");
    }
}
