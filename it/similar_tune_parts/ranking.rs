//! Re-rank finite pools and separately retrieve candidates needing new arbitration.
use super::association;
use super::config::Config;
use super::data;
use super::score;
use super::stats::Stats;
use super::translation;
use crate::similar_replay::Measured;
use codeeraser::similar::bm25::QueryTerm;

pub struct Frame {
    pub doc: usize,
    pub base: Vec<Option<i128>>,
    pub shortlist: Vec<usize>,
}

impl Frame {
    pub fn build(m: &Measured, doc: usize) -> Self {
        let hits = m
            .corpus
            .top_k(&m.corpus.query_of(doc), m.corpus.docs.len(), Some(doc));
        let mut base = vec![None; m.corpus.docs.len()];
        for h in &hits {
            base[h.doc] = Some(i128::from(h.score));
        }
        Self {
            doc,
            base,
            shortlist: hits.iter().map(|h| h.doc).collect(),
        }
    }
}

pub struct Prepared<'a> {
    pub config: &'a Config,
    pub query: Vec<QueryTerm>,
    translated: Vec<translation::Term>,
}

impl<'a> Prepared<'a> {
    pub fn build(m: &Measured, s: &Stats, f: &Frame, c: &'a Config) -> Self {
        let bare = score::query(m, s, f.doc, c);
        let query = association::expanded(m, s, f.doc, &bare, c);
        let translated = if c.text("model", "bm") == "translation" {
            translation::Kernel::build(s, c.int("mix", 4)).query(s, &query)
        } else {
            Vec::new()
        };
        Self {
            config: c,
            query,
            translated,
        }
    }

    fn score(&self, m: &Measured, s: &Stats, doc: usize) -> i128 {
        if self.translated.is_empty() {
            score::score(m, s, &self.query, doc, self.config)
        } else {
            translation::score(m, s, &self.translated, doc, self.config)
        }
    }
}

#[derive(Debug)]
pub struct Ranked {
    pub doc: usize,
    pub score: i128,
    key: [i128; 3],
}

pub fn rank(m: &Measured, s: &Stats, f: &Frame, p: &Prepared<'_>, seats: &[usize]) -> Vec<Ranked> {
    let mut out: Vec<_> = seats
        .iter()
        .copied()
        .map(|doc| {
            let score = p.score(m, s, doc);
            Ranked {
                doc,
                score,
                key: order(m, f, p.config, doc, score),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.key
            .cmp(&a.key)
            .then_with(|| identity(m, a.doc).cmp(&identity(m, b.doc)))
    });
    out
}

fn order(m: &Measured, f: &Frame, c: &Config, doc: usize, score: i128) -> [i128; 3] {
    let base = f.base[doc].unwrap_or(0);
    match c.text("order", "score") {
        "tie" => [base, score, 0],
        "band" => {
            let max = f
                .shortlist
                .first()
                .and_then(|&d| f.base[d])
                .unwrap_or(1)
                .max(1);
            [base * c.int("bands", 20) / max, score, base]
        }
        "stage" => {
            let eligible = f
                .shortlist
                .iter()
                .take(c.int("stage", 5) as usize)
                .any(|d| *d == doc);
            [i128::from(eligible), if eligible { score } else { base }, 0]
        }
        "role" | "name" | "shape" => {
            let e = data::evidence(m, f.doc, doc);
            let first = match c.text("order", "") {
                "name" => i128::from(e.hits[0]),
                "shape" => i128::from(e.shape),
                _ => i128::from(e.role()),
            };
            [first, score, 0]
        }
        _ => [score, 0, 0],
    }
}

pub fn retrieve(m: &Measured, s: &Stats, f: &Frame, p: &Prepared<'_>) -> Vec<Ranked> {
    let model = p.config.text("model", "bm");
    let all = matches!(model, "lm" | "translation") || p.config.text("idf", "rsi") != "rsi";
    let seats: Vec<_> = if all {
        (0..m.corpus.docs.len()).filter(|i| *i != f.doc).collect()
    } else {
        m.corpus
            .top_k(&p.query, m.corpus.docs.len(), Some(f.doc))
            .iter()
            .map(|h| h.doc)
            .collect()
    };
    let mut ranked = rank(m, s, f, p, &seats);
    ranked.truncate(5);
    ranked
}

pub fn identity(m: &Measured, doc: usize) -> (&str, &str, i64) {
    let d = &m.corpus.docs[doc];
    (&d.path, &d.bag.key, d.bag.nth)
}
