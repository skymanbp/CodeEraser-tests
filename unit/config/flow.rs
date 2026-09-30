use super::*;
use crate::config::{Config, TIERS, canonical};

/// Every tier reads itself back and is its own knob: the default is
/// silence to the digest, each other tier moves it under `flow`, no
/// two alike and none like the tombstone class's same tier. A tier
/// outside the four is refused by name (the load throat reads the
/// same `tier_fault`; flow_face.rs drives it through `ce flow`).
#[test]
fn every_tier_is_its_own_knob_and_a_typo_is_refused() {
    assert_eq!(FlowCfg::default().tier(), FLOW_DEFAULT);
    let mut digests = std::collections::BTreeSet::new();
    for t in TIERS {
        let cfg: Config = toml::from_str(&format!("[flow]\ntier = {t:?}\n")).expect(t);
        assert_eq!(cfg.flow.tier(), t);
        let Some(d) = cfg.knobs_digest() else {
            assert_eq!(t, FLOW_DEFAULT, "only the default is silence");
            continue;
        };
        assert!(digests.insert(d), "{t} moves the digest on its own");
        assert_eq!(canonical(&cfg), serde_json::json!({"flow": {"tier": t}}));
        let tomb: Config = toml::from_str(&format!("[tombstone]\ntier = {t:?}\n")).expect(t);
        assert_ne!(tomb.knobs_digest(), Some(d), "two classes, two knobs");
    }
    assert_eq!(digests.len(), TIERS.len() - 1);
    let fault = crate::config::tier::tier_fault("[flow]", Some("Deny")).expect("a typo");
    assert!(
        fault.contains("[flow] tier \"Deny\"") && fault.contains("observe | warn | ask | deny"),
        "{fault}"
    );
    assert!(toml::from_str::<Config>("[flow]\nmode = \"deny\"\n").is_err());
}
