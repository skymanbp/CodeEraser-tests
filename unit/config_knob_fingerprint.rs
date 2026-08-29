use super::*;
use serde_json::{Value, json};

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

fn with_score(score: ScoreCfg) -> Config {
    Config {
        score,
        ..Config::default()
    }
}

/// A repo that declares nothing has no fingerprint. Absence is the
/// state the fence must leave untouched, byte for byte, and it is
/// what makes the fence free for everyone who never opted in.
#[test]
fn the_shipped_default_has_no_fingerprint() {
    assert_eq!(Config::default().knobs_digest(), None);
    assert_eq!(canonical(&Config::default()), json!({}));
}

/// The two knobs the adversarial review turned into a bypass. This
/// is the reason the fingerprint covers the whole config instead of
/// the class table: `viol_cost = 0` pins the score at the scale so
/// `--fail-under` can never fail, and `tol_abs` erases the
/// ratchet's tolerance. Neither touches [[rules.class]].
#[test]
fn the_score_knobs_that_bypassed_the_gates_move_it() {
    let base = Config::default().knobs_digest();
    let viol = with_score(ScoreCfg {
        viol_cost: Some(0),
        ..ScoreCfg::default()
    });
    let tol = with_score(ScoreCfg {
        tol_abs: Some(100_000),
        ..ScoreCfg::default()
    });
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

/// O39, the canonical form: a knob spelled at its EFFECTIVE default
/// is the same fingerprint as silence — one row per table the
/// default comes from (the config's own, the core's score constants,
/// the trend's, the graph's, the weight every axis carries, the
/// guard's switch). Each row is a config that judges exactly as the
/// shipped default does, so each must hash as it does: to nothing.
#[test]
fn a_knob_declared_at_its_default_is_silence() {
    let rows: [(&str, Config); 6] = [
        (
            "thresholds spelled out",
            Config {
                thresholds: Thresholds {
                    file_lines_warn: 300,
                    ..Thresholds::default()
                },
                ..Config::default()
            },
        ),
        (
            "the core's score defaults spelled out",
            with_score(crate::score::knobs::core_defaults()),
        ),
        (
            "the trend's defaults spelled out",
            Config {
                trend: TrendCfg::core(),
                ..Config::default()
            },
        ),
        (
            "the graph's floor spelled out",
            Config {
                graph: GraphCfg {
                    scc_floor: Some(crate::score::knobs::CORE_SCC_FLOOR),
                    ..GraphCfg::default()
                },
                ..Config::default()
            },
        ),
        (
            "an axis weighed at the default weight",
            with_score(ScoreCfg {
                weights: [("size".to_string(), 1)].into_iter().collect(),
                ..ScoreCfg::default()
            }),
        ),
        (
            "the guard switch at its default",
            Config {
                guard: Guard {
                    mode: None,
                    zone_tiers: false,
                },
                ..Config::default()
            },
        ),
    ];
    for (why, cfg) in rows {
        assert_eq!(cfg.knobs_digest(), None, "{why}: {}", canonical(&cfg));
    }
    // the same axis at another weight, and at the default weight the
    // repo itself moved, are declarations
    let heavy = with_score(ScoreCfg {
        weights: [("size".to_string(), 2)].into_iter().collect(),
        ..ScoreCfg::default()
    });
    assert_eq!(
        canonical(&heavy),
        json!({"score": {"weights": {"size": 2}}})
    );
    let moved = with_score(ScoreCfg {
        weights: [("size".to_string(), 1)].into_iter().collect(),
        default_weight: Some(2),
        ..ScoreCfg::default()
    });
    assert_eq!(
        canonical(&moved),
        json!({"score": {"default_weight": 2, "weights": {"size": 1}}})
    );
}

/// The tree keeps exactly what differs: no `null` (an undeclared
/// option), no empty object, and inside a class the absent knobs are
/// gone while a knob spelled at the global line stays — its default
/// is inheritance, not a constant. So a new option on any table
/// never moves the digest of a repo that never declared it: the
/// counterfactual grows a serialized tree by one null leaf and one
/// empty table and prunes to the same bytes.
#[test]
fn the_tree_holds_declarations_only() {
    let classed = with_class("vendored", &["vendor/**"], Some(0));
    assert_eq!(
        canonical(&classed),
        json!({"rules": {"class": [{
            "name": "vendored", "globs": ["vendor/**"],
            "knobs": {"ratchet_tolerance": 0}
        }]}})
    );
    let spelled = Config {
        rules: RulesCfg {
            class: vec![ClassCfg {
                name: "docs".into(),
                globs: vec!["docs/**".into()],
                knobs: ClassKnobs {
                    file_lines_warn: Some(300),
                    ..ClassKnobs::default()
                },
            }],
        },
        ..Config::default()
    };
    assert_eq!(
        canonical(&spelled)["rules"]["class"][0]["knobs"],
        json!({"file_lines_warn": 300}),
        "a class line spelled at the global value shadows later global edits"
    );
    let shipped = json!({"a": {"x": 1}, "b": [1]});
    let grown = json!({"a": {"x": 1, "y_new": null}, "b": [1], "c_new": {}});
    assert_eq!(canonical::prune(grown, Some(&shipped)), None);
    let declared = json!({"a": {"x": 2, "y_new": null}, "b": [1, 2]});
    assert_eq!(
        canonical::prune(declared, Some(&shipped)),
        Some(json!({"a": {"x": 2}, "b": [1, 2]}))
    );
    // an array is one value: its own default is compared whole
    let tree: Value = canonical(&Config {
        exclude: vec!["vendor/**".into()],
        ..Config::default()
    });
    assert_eq!(tree, json!({"exclude": ["vendor/**"]}));
}
