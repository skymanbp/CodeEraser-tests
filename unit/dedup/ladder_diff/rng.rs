//! The seeded stream every random leg draws from, and the draws the
//! legs share (table words, paths, a site's own file).

use crate::graph::roots::join_dir;
use std::collections::BTreeSet;

/// The seeded stream every random leg draws from: the crate's one
/// stream (crate::testutil::Draw, `Draw::ladder` per leg salt), with the
/// draws the ladder legs add below.
pub(super) type Rng = crate::testutil::Draw;

impl crate::testutil::Draw {
    /// Mostly (`percent` in a hundred) one of a table's first `plain`
    /// words, otherwise any of them: the odd spellings sit at its end.
    pub(super) fn mostly(
        &mut self,
        percent: usize,
        table: &'static str,
        plain: usize,
    ) -> &'static str {
        if self.chance(percent) {
            let words: Vec<&'static str> = table.split('|').take(plain).collect();
            words[self.below(words.len())]
        } else {
            self.pick(table)
        }
    }

    /// `n` paths joining a directory and a base name.
    pub(super) fn paths(
        &mut self,
        dirs: &'static str,
        bases: &'static str,
        n: usize,
    ) -> BTreeSet<String> {
        (0..n)
            .map(|_| join_dir(self.pick(dirs), self.pick(bases)))
            .collect()
    }

    /// A site's own file: one of `files`, or (one time in ten, or when
    /// there is none) a file of that name the walk does not hold.
    pub(super) fn origin(
        &mut self,
        files: &[&String],
        dirs: &'static str,
        unwalked: &str,
    ) -> String {
        if files.is_empty() || self.chance(10) {
            join_dir(self.pick(dirs), unwalked)
        } else {
            files[self.below(files.len())].clone()
        }
    }
}
