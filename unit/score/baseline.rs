use super::*;

/// The §7.2 counterfactual pair: side order must not matter, and
/// every field participates (a moved LINE changes nothing here
/// because lines are not fields).
#[test]
fn member_identity_is_order_free_and_field_sensitive() {
    let a: Side = ("a.rs".into(), "work/1".into(), 0);
    let b: Side = ("b.rs".into(), "work/1".into(), 1);
    assert_eq!(member_id("clone", &a, &b), member_id("clone", &b, &a));
    let c: Side = ("b.rs".into(), "work/1".into(), 2);
    assert_ne!(member_id("clone", &a, &b), member_id("clone", &a, &c));
    assert_ne!(member_id("clone", &a, &b), member_id("t3", &a, &b));
}

/// O31: a present file that is not a baseline document is a named
/// error, never "no baseline" — only a MISSING file is None. And O30's
/// defense in depth: a scoped caller never persists, whatever the CLI
/// above it did. A scratch anchored at itself, so path_for stays home.
#[test]
fn read_is_semantic_and_write_stays_at_the_root() {
    let dir = crate::testutil::scratch("baseline-read");
    std::fs::write(dir.join("ce.toml"), "\n").expect("anchor");
    let file = dir.join("ce-baseline.json");
    assert!(read(&dir).expect("no file").is_none(), "missing = None");
    for bad in [
        "null",
        "[]",
        "{}",
        "{\"continuous\": []}",
        "{\"discrete\": []}",
    ] {
        std::fs::write(&file, bad).expect("seed");
        let err = read(&dir).expect_err(bad).to_string();
        assert!(err.contains("not a baseline document"), "{bad}: {err}");
    }
    let valid = "{\"continuous\": [], \"discrete\": [], \"softLine\": 300}\n";
    std::fs::write(&file, valid).expect("seed");
    assert_eq!(read(&dir).expect("valid").expect("some")["softLine"], 300);

    let sub = dir.join("pkg");
    std::fs::create_dir_all(&sub).expect("mkdir");
    let err = write(&sub, &json!({"continuous": [], "discrete": []}))
        .expect_err("scoped")
        .to_string();
    assert!(err.contains("per-project"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&file).expect("bytes"),
        valid,
        "untouched"
    );
    assert!(!sub.join("ce-baseline.json").exists(), "no second floor");
    std::fs::remove_dir_all(&dir).ok();
}
