//! `n` lines of Rust that bait nothing: no duplicate run for the
//! probe to find, no unit for the scanner to measure — the zone rule
//! counts lines and nothing else, so the corpus of a zone e2e is
//! deliberately inert. Its own module because a private copy inside
//! the gate would be one more shape for the clone ratchet to match
//! against guard_budget_parity's.

pub fn filler(n: usize) -> String {
    "// filler\n".repeat(n)
}
