//! The frozen term road of the same-role advisor (plan v2.33 W6): stem.rs,
//! terms.rs, bag.rs and docs.rs in oracle/ are byte copies of
//! cli/src/similar/ at 1324c927, the last commit before the core began
//! answering the bags (bags/1). The only edits: the trailing unit-test
//! mount of stem.rs and terms.rs reads `../stem.rs` / `../terms.rs` (the
//! road's own unit tests now test the frozen road), and bag.rs's is
//! dropped (the live bag.rs mounts unit/similar/bag.rs, which spells its
//! expected terms with the frozen road). similar/mod.rs mounts this
//! module for tests only; the differential gate (unit/similar/
//! bags_diff.rs) holds the core's term road against these copies.
//!
//! Each copy is mounted here by `#[path]` (oracle/ holds no mod.rs: a
//! parent there would turn the copies' references to their old parent
//! into edges back to it — graph/frozen_cfg.rs, same reason). The copies
//! find each other as `super::stem`, `super::terms`, `super::docs`.

#[path = "oracle/docs.rs"]
pub(crate) mod docs;
#[path = "oracle/stem.rs"]
pub(crate) mod stem;
#[path = "oracle/terms.rs"]
pub(crate) mod terms;
// the copy's accessors (empty, len, is_empty, channel) are read by no gate
#[allow(dead_code)]
#[path = "oracle/bag.rs"]
pub(crate) mod bag;

/// The frozen road's term of one word / feature under a LIVE channel —
/// what the unit tests that seat hand-made bags (ppmi, rank, reader,
/// store, query, testutil) spell their expected terms with; the
/// differential gate holds the core's road to this one.
pub(crate) fn word_term(ch: crate::similar::Channel, word: &str) -> u64 {
    terms::word_term(frozen(ch), word)
}

pub(crate) fn feature_term(ch: crate::similar::Channel, feature: &[u8]) -> u64 {
    terms::feature_term(frozen(ch), feature)
}

/// The frozen road's channel of a live one (the same six, same order).
fn frozen(ch: crate::similar::Channel) -> terms::Channel {
    terms::Channel::ALL[ch.index()]
}
