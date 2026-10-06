//! The W7 differential's seeded stream. Its own, not the ladder legs'
//! (unit/dedup/ladder_diff/rng.rs): that module is mounted under dedup
//! and keeps its items to its parent, and mounting the same file a
//! second time here is a duplicate module. Each draw hashes the seed
//! with the draw's count (std's `DefaultHasher`, fixed keys): the same
//! seed and toolchain draw the same stream.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) struct Rng {
    seed: u64,
    drawn: u64,
}

impl Rng {
    /// The leg's stream: `CE_LADDER_DIFF_SEED` (the seed every
    /// differential reads) with the leg's salt.
    pub(super) fn new(salt: u64) -> Rng {
        let seed = std::env::var("CE_LADDER_DIFF_SEED").ok();
        let seed: u64 = seed.and_then(|s| s.parse().ok()).unwrap_or(0x5eed_2033);
        Rng {
            seed: seed ^ salt.rotate_left(32),
            drawn: 0,
        }
    }

    /// One word of a table's `|`-separated words (an empty one allowed).
    pub(super) fn pick(&mut self, table: &'static str) -> &'static str {
        let k = self.below(table.split('|').count());
        table.split('|').nth(k).expect("a draw below the count")
    }

    /// True `percent` times in a hundred.
    pub(super) fn chance(&mut self, percent: usize) -> bool {
        percent > self.below(100)
    }

    /// A draw in `0..n`.
    pub(super) fn below(&mut self, n: usize) -> usize {
        let mut h = DefaultHasher::new();
        (self.seed, self.drawn).hash(&mut h);
        self.drawn += 1;
        usize::try_from(h.finish() % n as u64).expect("below a usize")
    }
}
