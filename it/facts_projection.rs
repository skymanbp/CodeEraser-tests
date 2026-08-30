//! ADR-009 gate 3 (plan v2.21 S3): contracts/docs-facts.json is the
//! registry's projection — blessed locally, byte-compared everywhere
//! else, so a version, revision, floor or budget that moved without
//! its documents is a red CI leg naming the id.

use crate::common::{assert_matches_golden, repo_root};
use crate::facts;

#[test]
fn the_projection_is_current() {
    assert_matches_golden(
        &facts::projection(),
        &repo_root().join("contracts/docs-facts.json"),
    );
}

#[test]
fn the_projection_is_self_describing() {
    let doc: serde_json::Value = serde_json::from_str(&facts::projection()).expect("json");
    assert_eq!(doc["schema"], facts::PROJECTION_SCHEMA);
    assert_eq!(
        doc["facts"].as_array().expect("rows").len(),
        facts::registry().len()
    );
    assert_eq!(doc["scraped"], facts::scraped_count());
}
