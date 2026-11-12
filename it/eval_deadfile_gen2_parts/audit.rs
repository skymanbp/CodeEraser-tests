//! BARS-GEN2 section 3's audit of the undetermined, its selection and
//! reading C. The selection is a pure function of the events: per state
//! the AUDIT_ROWS undetermined events lowest by
//! `sha256("ce-deadfile-gen2-v1|" + corpus + "|" + commit + "|" + path)`,
//! all of them when fewer. Reading C is reading B with the undetermined
//! added at the audited `in_use` share, exactly (G1). The auditors' answers are read
//! by whole path component (N8: a substring is no match), never with
//! generation 1's `named` reader.

use super::model::{Fate, State};
use super::readings::{Reading, share};
use crate::eval_support::content_sha;

pub const AUDIT_ROWS: usize = 60;
pub const SALT: &str = "ce-deadfile-gen2-v1";

/// One undetermined event, by its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub corpus: String,
    pub commit: String,
    pub path: String,
    pub state: State,
}

/// The selection key of a row.
pub fn key(r: &Row) -> String {
    content_sha(&format!("{SALT}|{}|{}|{}", r.corpus, r.commit, r.path))
}

/// Per state, the AUDIT_ROWS rows lowest by key (all when fewer), in key
/// order.
pub fn drawn(rows: &[Row], state: State) -> Vec<Row> {
    let mut mine: Vec<(String, Row)> = rows
        .iter()
        .filter(|r| r.state == state)
        .map(|r| (key(r), r.clone()))
        .collect();
    mine.sort_by(|a, b| a.0.cmp(&b.0));
    mine.into_iter().take(AUDIT_ROWS).map(|(_, r)| r).collect()
}

/// Whether an answer's free text names a path's file by a whole path
/// component: `docs/conf.py` names `conf.py`, `myconf.py` does not.
pub fn names_component(text: &str, path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    text.split(|c: char| c.is_whitespace() || "/\\`'\"()[]{},;:".contains(c))
        .any(|w| w == base || w.trim_end_matches('.') == base)
}

/// Reading C (G1): reading B with the undetermined events added at the
/// audited `in_use` share, as an exact fraction — `(k_B * audited + U *
/// in_use, (n_B + U) * audited)`, U the undetermined events, no flooring.
/// Reading B itself when there are none; `None` when there are some and
/// nothing was audited. Stated per state, pooled over the primary
/// corpora, nothing finer.
pub fn reading_c(fates: &[Fate], in_use: usize, audited: usize) -> Option<(usize, usize)> {
    let (k, n) = share(Reading::B, fates);
    let u = fates.iter().filter(|f| **f == Fate::Undetermined).count();
    match (u, audited) {
        (0, _) => Some((k, n)),
        (_, 0) => None,
        _ => Some((k * audited + u * in_use, (n + u) * audited)),
    }
}

/// Reading C twice when audited rows end `unsure`: those counted as
/// `in_use`, then not.
pub fn reading_c_twice(
    fates: &[Fate],
    in_use: usize,
    unsure: usize,
    audited: usize,
) -> [Option<(usize, usize)>; 2] {
    [
        reading_c(fates, in_use + unsure, audited),
        reading_c(fates, in_use, audited),
    ]
}
