//! Full retrieval has no truth outside the pool: export identities, never infer labels.
use super::data::Pool;
use super::ranking::Ranked;
use crate::eval_support::content_sha;
use crate::similar_replay::Measured;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Novel(BTreeMap<usize, Value>);

impl Novel {
    pub fn add(
        &mut self,
        m: &Measured,
        p: &Pool<'_>,
        config: &str,
        ranked: &[Ranked],
    ) -> [usize; 2] {
        let mut count = [0; 2];
        for (rank, h) in ranked.iter().enumerate() {
            if p.candidates.iter().any(|(doc, _)| *doc == h.doc) {
                continue;
            }
            let d = &m.corpus.docs[h.doc];
            let stale = p
                .row
                .candidates
                .iter()
                .any(|c| c.id.path == d.path && c.id.key == d.bag.key && c.id.nth == d.bag.nth);
            count[usize::from(stale)] += 1;
            let v = self.0.entry(h.doc).or_insert_with(|| {
                json!({
                    "path": d.path, "key": d.bag.key, "nth": d.bag.nth,
                    "status": if stale { "stale_label" } else { "unlabelled" },
                    "start_line": d.bag.start_line, "end_line": d.bag.end_line,
                    "sha": content_sha(&m.texts[&d.path]), "placements": [],
                })
            });
            v["placements"]
                .as_array_mut()
                .expect("placements")
                .push(json!({
                    "config": config, "rank": rank + 1, "score": h.score.to_string(),
                }));
        }
        count
    }

    pub fn row(&self, p: &Pool<'_>) -> Value {
        json!({"query": p.row.id, "rank": p.row.rank, "corpus": p.row.corpus,
            "requires_arbitration": self.0.values().collect::<Vec<_>>()})
    }
}
