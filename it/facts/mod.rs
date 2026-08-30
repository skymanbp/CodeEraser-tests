//! The derived-fact registry (plan v2.21, ADR-009): every fact a
//! document carries that moves with a release — versions, counts,
//! schema ids, gate floors, citation anchors — is resolved here once
//! and rendered everywhere. S1 lands only the precondition every
//! later piece writes through: ONE reader of the bless switch.
//!
//! "Bless writes; bless never decides." A regeneration run rewrites a
//! generated surface from its source; a plain run byte-compares. The
//! switch used to be read at six sites in two spellings, and nothing
//! stopped a CI leg from being handed CE_BLESS=1 and turning every
//! compare into a write. Now the sites call `blessing()`, the switch
//! is exactly "1" (any-value `is_ok()` let CE_BLESS=0 or an empty var
//! bless-and-pass — attack-review finding), and a run that is BOTH
//! blessing and on CI refuses by name. bless_guard.rs holds the census
//! (one reader, and the workflows never spell the switch).

/// CE_BLESS=1 — exactly "1". Under CI (the `CI` variable every hosted
/// runner exports) a bless is a category error, not a mode: CI only
/// compares, and a write there would pass a drift by regenerating it.
pub fn blessing() -> bool {
    let on = std::env::var("CE_BLESS").as_deref() == Ok("1");
    if on && std::env::var_os("CI").is_some() {
        panic!(
            "CE_BLESS=1 under CI: a bless writes locally and CI only compares (plan v2.21 gate 1)"
        );
    }
    on
}
