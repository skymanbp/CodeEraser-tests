//! The frozen Rust oracles of plan v2.33 W3 (user instruction
//! 2026-10-03): each algorithm the lane moved into the core, copied
//! from 093aede4 statement for statement, kept here after the
//! production copy was deleted so the differential gates can drive the
//! same seeded inputs through the old Rust and through the core over
//! the real wire. Never edit an oracle body.

pub mod docdup;
pub mod fourclass;
pub mod similar;
pub mod t3;

/// The library as similar.rs names it: the it crate mounts that oracle
/// too (the tuning instrument's formulas), with `lib` = `codeeraser`.
mod lib {
    pub(crate) use crate::similar;
}

/// The differential gates' other side: the real core over the real wire.
pub(crate) fn core_link() -> crate::corelink::Link {
    let core = crate::daemon::judge::core_bin().expect("a core");
    crate::corelink::Link::open(&core).expect("open core").0
}

/// A fixed-seed linear congruential generator (no dependency, no clock).
pub(crate) struct Lcg(pub u64);

impl Lcg {
    pub(crate) fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n.max(1) as u64) as usize
    }

    pub(crate) fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.next(xs.len())]
    }
}
