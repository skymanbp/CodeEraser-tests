//! Offline pool re-ranking; run explicitly in release, with the core binary available.
//! cargo test --release -j 8 --test it -- --ignored similar_tune::similar_tune --nocapture
//! Outputs go only to cli/target/similar-tune (gitignored). No blessing path exists.

#[test]
#[ignore]
fn similar_tune() {
    crate::similar_tune_parts::run();
}
