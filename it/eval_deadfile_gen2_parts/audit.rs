//! BARS-GEN2 section 3's audit of the undetermined, its selection and
//! reading C. The selection is a pure function of the events: per state
//! the AUDIT_ROWS undetermined events lowest by
//! `sha256("ce-deadfile-gen2-v1|" + corpus + "|" + commit + "|" + path)`,
//! all of them when fewer. Reading C is reading B with the undetermined
//! added at the audited `in_use` share. The auditors' answers are read
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

/// Reading C: reading B's `(false, denominator)` with every
/// undetermined event added to the denominator and, at the audited share
/// `in_use / audited` (floored to whole events), to the false count.
pub fn reading_c(fates: &[Fate], in_use: usize, audited: usize) -> (usize, usize) {
    let (k, n) = share(Reading::B, fates);
    let undetermined = fates.iter().filter(|f| **f == Fate::Undetermined).count();
    let in_use = if audited == 0 {
        0
    } else {
        undetermined * in_use / audited
    };
    (k + in_use, n + undetermined)
}
