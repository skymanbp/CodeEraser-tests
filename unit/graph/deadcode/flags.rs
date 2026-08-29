use super::*;
use std::collections::BTreeSet;

fn no_targets() -> Declared {
    Declared::gather(Path::new("."), &BTreeSet::new(), &BTreeSet::new())
}

/// The allow-claim role (batch-7 slice 3), table-driven — the
/// docdup discipline transplanted: only a why-bearing marker
/// claims; a bare marker and an absent file claim nothing.
#[test]
fn allow_claim_requires_the_why_tail() {
    let cases = [
        (
            "a.py",
            Some("# ce:allow(deadcode) -- loader-invoked\n"),
            true,
        ),
        ("b.py", Some("# ce:allow(deadcode)\n"), false),
        ("missing.py", None, false),
    ];
    let root = crate::testutil::scratch("dc-allow");
    let cfg = crate::config::Config::default();
    let none = no_targets();
    for (name, text, want) in cases {
        if let Some(text) = text {
            std::fs::write(root.join(name), text).unwrap();
        }
        assert_eq!(allow_claim(&root, name), want, "{name}");
        let r = roles_of(&root, name, &cfg, &none);
        assert_eq!(r & ROLE_ALLOW != 0, want, "{name}");
    }
    std::fs::remove_dir_all(&root).ok();
}
