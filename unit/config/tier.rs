use super::*;

/// The three shapes every enforcement surface must be able to
/// tell apart: declared, defaulted, and MISTYPED.
#[test]
fn an_unknown_mode_degrades_to_observe_and_names_itself() {
    let declared = Guard {
        mode: Some("warn".into()),
        zone_tiers: false,
    };
    assert_eq!(declared.tier(PROMOTED_DEFAULT), "warn");
    assert_eq!(Guard::default().tier("observe"), "observe");
    let typo = Guard {
        mode: Some("Deny".into()),
        zone_tiers: false,
    };
    let got = typo.tier(PROMOTED_DEFAULT);
    assert!(got.starts_with("observe ("), "never enforces: {got}");
    assert!(got.contains("\"Deny\""), "names the value: {got}");
}

/// A broken ce.toml renders through the SAME throat, so the feed,
/// the health line and `ce doctor` cannot drift apart again.
#[test]
fn a_broken_config_renders_through_one_throat() {
    let broken: Result<Config, String> = Err("parse ce.toml: bad line 3".into());
    let got = tier_of(&broken, PROMOTED_DEFAULT);
    assert!(got.starts_with("observe (ce.toml ERROR:"), "{got}");
    assert!(got.contains("bad line 3"), "carries the cause: {got}");
}
