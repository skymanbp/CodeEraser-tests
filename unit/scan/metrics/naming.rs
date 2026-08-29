use super::*;

fn go(name: &str) -> [i64; 5] {
    facts(Lang::Go, NameStyle::MixedCaps, name)
}

#[test]
fn snake_rules() {
    assert!(conforms(facts(
        Lang::Python,
        NameStyle::Snake,
        "load_config"
    )));
    assert!(conforms(facts(Lang::Python, NameStyle::Snake, "__init__")));
    assert!(!conforms(facts(
        Lang::Python,
        NameStyle::Snake,
        "loadConfig"
    )));
}

#[test]
fn mixed_caps_rules() {
    assert!(conforms(go("loadConfig")));
    assert!(conforms(go("ServeHTTP")));
    assert!(!conforms(go("load_config")));
    // toolchain-mandated underscore families stay exempt — in Go
    assert!(conforms(go("ExampleParse_errors")));
    assert!(conforms(go("TestServer_Start")));
    assert!(conforms(go("Example_errors")));
    // the two dead defects: a lowercase boundary is no test name
    // (go vet's rule), and the exemption never leaves Go
    assert!(!conforms(go("Testing_helper")));
    assert!(!conforms(facts(
        Lang::TypeScript,
        NameStyle::MixedCaps,
        "TestServer_Start"
    )));
    assert!(!conforms(facts(
        Lang::Haskell,
        NameStyle::MixedCaps,
        "Testing_helper"
    )));
}

#[test]
fn sentinels_pass_as_unjudged() {
    for name in ["(anonymous)", "(non-utf8)", "\"my_key\"", "[dynamic_key]"] {
        let row = facts(Lang::TypeScript, NameStyle::MixedCaps, name);
        assert_eq!(row[1], 0, "{name}: no convention applies");
        assert!(conforms(row));
    }
}
