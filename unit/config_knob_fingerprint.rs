use super::*;

fn with_class(name: &str, globs: &[&str], tol: Option<usize>) -> Config {
    Config {
        rules: RulesCfg {
            class: vec![ClassCfg {
                name: name.into(),
                globs: globs.iter().map(|g| (*g).to_string()).collect(),
                knobs: ClassKnobs {
                    ratchet_tolerance: tol,
                    ..ClassKnobs::default()
                },
            }],
        },
        ..Config::default()
    }
}

/// A repo that declares nothing has no fingerprint. Absence is the
/// state the fence must leave untouched, byte for byte, and it is
/// what makes the fence free for everyone who never opted in.
#[test]
fn the_shipped_default_has_no_fingerprint() {
    assert_eq!(Config::default().knobs_digest(), None);
}

/// The two knobs the adversarial review turned into a bypass. This
/// is the reason the fingerprint covers the whole config instead of
/// the class table: `viol_cost = 0` pins the score at the scale so
/// `--fail-under` can never fail, and `tol_abs` erases the
/// ratchet's tolerance. Neither touches [[rules.class]].
#[test]
fn the_score_knobs_that_bypassed_the_gates_move_it() {
    let base = Config::default().knobs_digest();
    let viol = Config {
        score: ScoreCfg {
            viol_cost: Some(0),
            ..ScoreCfg::default()
        },
        ..Config::default()
    };
    let tol = Config {
        score: ScoreCfg {
            tol_abs: Some(100_000),
            ..ScoreCfg::default()
        },
        ..Config::default()
    };
    assert_ne!(base, viol.knobs_digest(), "viol_cost");
    assert_ne!(base, tol.knobs_digest(), "tol_abs");
    assert_ne!(
        viol.knobs_digest(),
        tol.knobs_digest(),
        "and from each other"
    );
}

/// Everything a rulepack declaration can say still moves it —
/// including declaration ORDER, which is precedence, and a knob set
/// to zero, which is a claim and not an absence.
#[test]
fn the_rulepack_still_moves_it_in_every_part() {
    let a = with_class("vendored", &["vendor/**"], None).knobs_digest();
    assert!(a.is_some());
    assert_ne!(a, with_class("vendor", &["vendor/**"], None).knobs_digest());
    assert_ne!(
        a,
        with_class("vendored", &["vendor/*"], None).knobs_digest()
    );
    assert_ne!(
        a,
        with_class("vendored", &["vendor/**"], Some(0)).knobs_digest()
    );
    let two = |x: &str, y: &str| Config {
        rules: RulesCfg {
            class: [x, y]
                .iter()
                .map(|n| ClassCfg {
                    name: (*n).into(),
                    globs: vec![format!("{n}/**")],
                    knobs: ClassKnobs::default(),
                })
                .collect(),
        },
        ..Config::default()
    };
    assert_ne!(
        two("a", "b").knobs_digest(),
        two("b", "a").knobs_digest(),
        "declaration order IS precedence"
    );
}

/// An exclude glob drops files from the walk, and with them their
/// ratchet rows — the third road the review found, fenced by the
/// same scalar as the first two.
#[test]
fn an_exclude_glob_moves_it() {
    let excluded = Config {
        exclude: vec!["vendor/**".into()],
        ..Config::default()
    };
    assert_ne!(Config::default().knobs_digest(), excluded.knobs_digest());
}

/// No value can be read as structure: a name carrying the JSON the
/// encoding uses is escaped, not spliced. The class-only draft
/// separated fields with a NUL and its own test found the collision
/// immediately; serialized JSON has no such seam to exploit.
#[test]
fn a_name_cannot_impersonate_the_encoding() {
    assert_ne!(
        with_class("a", &["b"], None).knobs_digest(),
        with_class("a\",\"globs\":[\"b", &[], None).knobs_digest(),
    );
}
