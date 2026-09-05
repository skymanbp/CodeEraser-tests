//! Join immutable labels to freshly measured identities; stale text is never scored.
use crate::similar_replay::Measured;
use crate::similar_replay_parts::identity_sha;
use codeeraser::similar::{Channel, bm25};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
pub struct Identity {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub sha: String,
}

#[derive(Deserialize)]
pub struct Placement {
    pub rank: usize,
    pub score: i64,
}

#[derive(Deserialize)]
pub struct Candidate {
    #[serde(flatten)]
    pub id: Identity,
    pub truth: String,
    pub clone: bool,
    pub hits: [u32; 6],
    pub shape_equal: bool,
    pub same_file: bool,
    pub bare: Option<Placement>,
    pub widened: Option<Placement>,
}

#[derive(Deserialize)]
pub struct Row {
    #[serde(flatten)]
    pub id: Identity,
    pub corpus: String,
    pub rank: String,
    pub role_bare: bool,
    pub bag: BTreeMap<String, u32>,
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
pub struct Oracle {
    pub rows: Vec<Row>,
}

pub struct Pool<'a> {
    pub row: &'a Row,
    pub query: usize,
    pub candidates: Vec<(usize, &'a Candidate)>,
}

#[derive(Default, Serialize)]
pub struct Coverage {
    pub corpus: String,
    pub expected: [usize; 2],
    pub retained: [usize; 2],
    pub query_sha_matched: usize,
    pub partial_pools: usize,
    pub empty_pools: usize,
    pub skipped: Vec<String>,
}

pub fn pools<'a>(m: &Measured, oracle: &'a Oracle) -> (Vec<Pool<'a>>, Coverage) {
    let hashes: BTreeMap<_, _> = m
        .texts
        .iter()
        .map(|(path, text)| (path.as_str(), identity_sha(text)))
        .collect();
    let mut cov = Coverage {
        corpus: m.name.into(),
        ..Coverage::default()
    };
    let mut out = Vec::new();
    for row in oracle.rows.iter().filter(|r| r.corpus == m.name) {
        cov.expected[0] += 1;
        cov.expected[1] += row.candidates.len();
        let Some(query) = seat(&row.id, m, &hashes, &mut cov) else {
            continue;
        };
        cov.query_sha_matched += 1;
        let candidates: Vec<_> = row
            .candidates
            .iter()
            .filter_map(|c| seat(&c.id, m, &hashes, &mut cov).map(|i| (i, c)))
            .collect();
        if !candidates.is_empty() {
            cov.partial_pools += usize::from(candidates.len() != row.candidates.len());
            cov.retained[0] += 1;
            cov.retained[1] += candidates.len();
            out.push(Pool {
                row,
                query,
                candidates,
            });
        } else {
            cov.empty_pools += 1;
        }
    }
    (out, cov)
}

fn seat(
    id: &Identity,
    measured: &Measured,
    hashes: &BTreeMap<&str, String>,
    cov: &mut Coverage,
) -> Option<usize> {
    let fresh = hashes.get(id.path.as_str()) == Some(&id.sha);
    let found = measured
        .corpus
        .docs
        .iter()
        .position(|d| d.path == id.path && d.bag.key == id.key && d.bag.nth == id.nth);
    if fresh && found.is_some() {
        return found;
    }
    let why = if !fresh {
        "sha/missing-file"
    } else {
        "missing-unit"
    };
    let message = format!("{why}: {} {}#{}", id.path, id.key, id.nth);
    assert_eq!(cov.corpus, "self", "pinned fixture changed: {message}");
    cov.skipped.push(message);
    None
}

#[derive(Clone, Serialize)]
pub struct Evidence {
    pub hits: [u32; 6],
    pub shape: bool,
    pub same_file: bool,
    pub query_names: u32,
}

impl Evidence {
    pub fn role(&self) -> bool {
        bm25::role(&self.hits, self.shape)
    }
}

pub fn evidence(m: &Measured, query: usize, doc: usize) -> Evidence {
    let (q, d) = (&m.corpus.docs[query], &m.corpus.docs[doc]);
    let mut hits = [0; 6];
    let names = q.bag.channel(Channel::Name).len() as u32;
    for (t, (ch, _)) in &q.bag.terms {
        if bm25::idf_fp(m.corpus.docs.len(), m.corpus.df(*t)) == 0 {
            continue;
        }
        if d.bag.terms.contains_key(t) {
            hits[ch.index()] += 1;
        }
    }
    Evidence {
        hits,
        shape: q.bag.channel(Channel::Shape) == d.bag.channel(Channel::Shape),
        same_file: q.path == d.path,
        query_names: names,
    }
}
