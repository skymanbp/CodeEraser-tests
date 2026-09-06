use crate::config::{Config, UiCfg, canonical};
use crate::testutil::scratch;

/// `en`, `zh` and absence load; anything else is refused at the load
/// throat by name; an unknown key under `[ui]` is refused like any
/// table's; and none of it is a knob — the digest and the canonical
/// tree never see the table (rule 6).
#[test]
fn the_language_is_en_zh_or_refused_and_never_a_knob() {
    assert_eq!(UiCfg::default().fault(), None);
    let parse = |toml: &str| toml::from_str::<Config>(toml).expect(toml);
    for toml in ["[ui]\nlang = \"en\"\n", "[ui]\nlang = \"zh\"\n"] {
        let cfg = parse(toml);
        assert_eq!(
            (cfg.ui.fault(), cfg.knobs_digest()),
            (None, None),
            "{toml:?}"
        );
        assert_eq!(canonical(&cfg), serde_json::json!({}), "{toml:?}");
    }
    let err = toml::from_str::<Config>("[ui]\nlanguage = \"zh\"\n").unwrap_err();
    assert!(err.to_string().contains("language"), "{err}");
    let dir = scratch("ui-cfg");
    std::fs::write(dir.join("ce.toml"), "[ui]\nlang = \"fr\"\n").expect("write ce.toml");
    let fault = Config::load(&dir).expect_err("fr is not a console language");
    assert!(fault.contains("[ui] lang = \"fr\""), "{fault}");
    let _ = std::fs::remove_dir_all(&dir);
}
