//! The seeded stream every random leg draws from, and the draws the
//! legs share (table words, paths, a site's own file).

use crate::graph::roots::join_dir;
use std::collections::BTreeSet;

/// xorshift64*: the seeded stream every random leg draws from.
pub(super) struct Rng(u64);

impl Rng {
    /// The leg's stream: the seed mixed with the leg's salt. The state
    /// only has to be nonzero (xorshift stays at zero); `.max(1)` keeps
    /// every other state as it is. (Until plan v2.33 W2-text stage E this
    /// was `| 1`, which folded each even state onto the odd one above it:
    /// seeds 2 and 3 drew the same trees on every leg.)
    pub(super) fn new(salt: u64) -> Rng {
        let seed = std::env::var("CE_LADDER_DIFF_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0x5eed_2033_u64);
        Rng((seed ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15)).max(1))
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    pub(super) fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// One word of a table (its words joined by `|`, an empty word
    /// allowed).
    pub(super) fn pick(&mut self, table: &'static str) -> &'static str {
        let words: Vec<&'static str> = table.split('|').collect();
        words[self.below(words.len())]
    }

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

    pub(super) fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
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
