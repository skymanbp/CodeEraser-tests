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

/// The differential gates' other side: the real core over the real wire.
pub(crate) fn core_link() -> crate::corelink::Link {
    let core = crate::daemon::judge::core_bin().expect("a core");
    crate::corelink::Link::open(&core).expect("open core").0
}

/// The differentials' fixed-seed stream (crate::testutil::Draw).
pub(crate) struct Lcg(crate::testutil::Draw);

impl Lcg {
    pub(crate) fn new(seed: u64) -> Lcg {
        Lcg(crate::testutil::Draw::new(seed, 0))
    }

    pub(crate) fn next(&mut self, n: usize) -> usize {
        self.0.below(n)
    }

    pub(crate) fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        self.0.pick_from(xs)
    }
}
