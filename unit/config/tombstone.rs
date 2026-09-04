use super::*;
use crate::config::{Config, canonical};
use crate::testutil::scratch;

/// The tier is the class's own: unset reads the route default, a
/// declared tier reads itself, and a mistyped one is refused by name
/// — never rendered as something that looks armed.
#[test]
fn the_tier_is_declared_defaulted_or_refused_by_name() {
    assert_eq!(TombstoneCfg::default().tier(), "observe");
    let warn = TombstoneCfg {
        tier: Some("warn".into()),
        ..TombstoneCfg::default()
    };
    assert_eq!((warn.tier(), warn.fault()), ("warn", None));
    let typo = TombstoneCfg {
        tier: Some("Deny".into()),
        ..TombstoneCfg::default()
    };
    let fault = typo.fault().expect("a typo is refused");
    assert!(fault.contains("[tombstone] tier \"Deny\""), "{fault}");
    assert!(fault.contains("observe | warn | ask | deny"), "{fault}");
}

/// Every key is a canonical-form knob: the default tier spelled out
/// is silence, each other declaration moves the digest by name, and
/// no two move it the same way.
#[test]
fn the_table_enters_the_knob_fingerprint_like_any_other() {
    let parse = |toml: &str| toml::from_str::<Config>(toml).expect(toml);
    assert_eq!(
        parse("[tombstone]\ntier = \"observe\"\n").knobs_digest(),
        None
    );
    let moved = [
        "[tombstone]\ntier = \"warn\"\n",
        "[tombstone]\nbudget = 0\n",
        "[tombstone]\nledger = [\"docs/plan.md\"]\n",
        "[tombstone]\nterms = [\"pork\"]\n",
    ];
    let mut digests: Vec<u64> = moved
        .iter()
        .map(|t| {
            parse(t)
                .knobs_digest()
                .unwrap_or_else(|| panic!("moves: {t}"))
        })
        .collect();
    digests.sort_unstable();
    digests.dedup();
    assert_eq!(
        digests.len(),
        moved.len(),
        "each key moves the digest on its own"
    );
    assert_eq!(
        canonical(&parse(moved[1])),
        serde_json::json!({"tombstone": {"budget": 0}})
    );
}

/// An unknown key is refused like any table's; a ledger glob and the
/// tier are judged at the load throat, and each refusal names its key.
#[test]
fn an_unknown_key_a_bad_glob_or_a_bad_tier_is_refused_at_load() {
    let err = toml::from_str::<Config>("[tombstone]\nexclude = [\"x\"]\n").unwrap_err();
    assert!(err.to_string().contains("exclude"), "{err}");
    let dir = scratch("tombstone-cfg");
    let refused = |table: &str, names: &str| {
        std::fs::write(dir.join("ce.toml"), table).expect("write ce.toml");
        let fault = Config::load(&dir).expect_err(table);
        assert!(fault.contains(names), "{fault}");
    };
    refused("[tombstone]\nledger = [\"!docs/\"]\n", "[tombstone] ledger");
    refused("[tombstone]\ntier = \"Deny\"\n", "[tombstone] tier");
    let _ = std::fs::remove_dir_all(&dir);
}
