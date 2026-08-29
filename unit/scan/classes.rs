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
/// unmatched path is class 0, and a Windows-spelled glob matches
/// through the same normalization the exclude list gets.
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
        owner(
            &[("gen", &["src\\generated\\*.rs"])],
            "src/generated/api.rs"
        ),
        1
    );
    assert!(
        !Classes::compile(root, &RulesCfg::default())
            .expect("empty")
            .declared()
    );
    let err = Classes::compile(root, &rules(&[("neg", &["!src/**"])]))
        .err()
        .expect("'!' refuses");
    assert!(err.contains("without '!'"), "{err}");
}
