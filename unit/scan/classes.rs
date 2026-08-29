use super::*;
use crate::config::ClassCfg;

type Decl<'a> = (&'a str, &'a [&'a str]);

fn rules(classes: &[Decl]) -> RulesCfg {
    RulesCfg {
        class: classes
            .iter()
            .map(|(name, globs)| ClassCfg {
                name: name.to_string(),
                globs: globs.iter().map(|g| g.to_string()).collect(),
                knobs: Default::default(),
            })
            .collect(),
    }
}

/// Declaration order is the tiebreak (C3): the same overlapping
/// pair flipped hands the shared path to the other class, an
/// unmatched path is class 0, a `dir/` glob owns the directory's
/// files (the one dialect, plan v2.18 step #14), and a
/// Windows-spelled glob is refused by name — never normalized.
#[test]
fn first_declared_match_owns_the_path() {
    let root = Path::new(".");
    let owner = |classes: &[Decl], path: &str| {
        Classes::compile(root, &rules(classes))
            .expect("compile")
            .class_of(path)
    };
    let (tests, cli): (Decl, Decl) = (("tests", &["cli/tests/**"]), ("cli", &["cli/**"]));
    assert_eq!(owner(&[tests, cli], "cli/tests/x.rs"), 1);
    assert_eq!(owner(&[tests, cli], "cli/src/x.rs"), 2);
    assert_eq!(owner(&[tests, cli], "core/app/X.hs"), 0);
    assert_eq!(
        owner(&[cli, tests], "cli/tests/x.rs"),
        1,
        "flipped order, flipped owner"
    );
    assert_eq!(
        owner(&[("gen", &["src/generated/"])], "src/generated/api.rs"),
        1
    );
    assert!(
        !Classes::compile(root, &RulesCfg::default())
            .expect("empty")
            .declared()
    );
    for (glob, names) in [
        ("!src/**", "without '!'"),
        ("src\\generated\\*.rs", "escape"),
    ] {
        let err = Classes::compile(root, &rules(&[("bad", &[glob])]))
            .err()
            .unwrap_or_else(|| panic!("{glob} refuses"));
        assert!(err.contains(names) && err.contains("\"bad\""), "{err}");
    }
}
