//! The hard-budget ledger's EXECUTOR (v2.35 guard lane, GD2). The
//! frozen rows of contracts/eval/fpr-budget-v1.json re-derive every
//! number they state; docs/FPR-REPLAY.md quotes the frozen table; the
//! growth refusal's tier is pinned at its literal value, so moving it
//! means moving this gate (REVIEW-1 Q2: no implication between the
//! ledger and the tier yet — that is the owner's ruling to make with
//! this table in hand); and a synthetic history walks the instrument
//! end to end.

use crate::common::ledger::{closes, count, two_commits};
use crate::common::tmp;
use crate::fpr_budget_replay::replay;
use crate::fpr_budget_replay_parts::{LEDGER, derived, table};
use codeeraser::guard::zone::{GROWTH_TIER, GrowthTier};

/// The ledger's corpora, in order: the two standing ones, then the
/// per-language ledger's.
const NAMES: [&str; 8] = [
    "self",
    "requests",
    "lua",
    "fmt",
    "gson",
    "luarocks",
    "stringr",
    "codeeraser",
];

#[test]
fn every_frozen_budget_row_closes_against_its_own_writes() {
    for c in &LEDGER.rows(&NAMES) {
        let rows = c["rows"].as_array().expect("rows");
        let events = count(c, "events") as usize;
        assert!(events > 0 && rows.len() <= events, "{}", c["name"]);
        for r in rows {
            let (b, a, cap) = (r["before"].as_u64(), r["after"].as_u64(), r["cap"].as_u64());
            let (b, a, cap) = (b.expect("before"), a.expect("after"), cap.expect("cap"));
            let want = if a <= b {
                "carried"
            } else if b <= cap {
                "crossing"
            } else {
                "growth"
            };
            assert!(
                cap > 0 && a > cap && r["kind"] == want,
                "{}: {r}",
                c["name"]
            );
        }
        closes(c, &derived(events, rows));
    }
}

/// The rule as the pages publish it: (page, the words it must carry).
/// A page that still said "past the line" would describe a rule the
/// hook no longer runs.
const WORDING: [(&str, &str); 5] = [
    (
        "docs/FPR-REPLAY.md",
        "cap != 0 ∧ after > cap ∧ after > before",
    ),
    (
        "docs/reference/methodology/11-fpr-discipline-and-the-guard-tier-ladder.md",
        "breach ⇔ cap != 0 ∧ after > cap ∧ after > before",
    ),
    ("README.md", "A write that grows a file past"),
    ("README.zh.md", "让文件长过"),
    ("plugin/README.md", "写入让文件长过其硬线"),
];

#[test]
fn the_budget_doc_quotes_the_frozen_table_and_the_tier_is_the_class_tier() {
    LEDGER.page_quotes("docs/FPR-REPLAY.md", &table(&LEDGER.corpora()));
    let root = crate::common::repo_root();
    for (page, words) in WORDING {
        let text = crate::facts::read(&root, page);
        assert!(
            text.contains(words),
            "{page}: the published rule is `{words}`"
        );
    }
    assert_eq!(
        GROWTH_TIER,
        GrowthTier::Class,
        "a growth refusal speaks at the class tier; moving it is the owner's ruling, \
         argued from contracts/eval/fpr-budget-v1.json, and this pin moves with it"
    );
}

/// `n` lines that bait nothing (fpr_zone_gate_helpers::filler's shape).
fn lines(n: usize, tag: &str) -> String {
    (0..n).map(|i| format!("// {tag} {i}\n")).collect()
}

/// A synthetic history with a hard line at 30: a new 40-line file
/// crosses, a 35-line file grown to 36 is a growth, a 40-line file
/// rewritten at 40 is carried, one cut from 40 to 20 lands under the
/// line (no row), and a 1-line file grown to 10 is under it too.
#[test]
fn a_synthetic_history_lands_one_crossing_one_growth_one_carried() {
    let toml = "[thresholds]\nfile_lines_warn = 10\nfile_lines_fail = 30\n";
    // `file:lines:tag` — a file of `lines` bait-free lines spelled by `tag`
    let tree = |spec: &str| -> Vec<(String, String)> {
        spec.split(' ')
            .map(|w| {
                let f: Vec<&str> = w.split(':').collect();
                (
                    f[0].to_string(),
                    lines(f[1].parse().expect("a count"), f[2]),
                )
            })
            .collect()
    };
    let mut seed = tree("b.rs:35:b c.rs:40:c d.rs:40:d e.rs:1:e");
    seed.push(("ce.toml".into(), toml.into()));
    let then = tree("a.rs:40:a b.rs:36:b c.rs:40:c2 d.rs:20:d e.rs:10:e");
    fn refs(v: &[(String, String)]) -> Vec<(&str, &str)> {
        v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
    }
    let repo = two_commits("fpr-budget-e2e", &refs(&seed), &refs(&then));
    let commits = crate::common::chain_of(&repo, "HEAD", None);
    let c = replay(&repo, &tmp("fpr-budget-e2e-shadow"), "e2e", &commits);
    let seen: Vec<(&str, &str, usize, usize)> = c
        .rows
        .iter()
        .map(|r| (r.rel.as_str(), r.kind, r.before, r.after))
        .collect();
    assert_eq!(
        (c.commits, c.events),
        (1, 5),
        "one write per changed in-scope file"
    );
    assert_eq!(
        seen,
        [
            ("a.rs", "crossing", 0, 40),
            ("b.rs", "growth", 35, 36),
            ("c.rs", "carried", 40, 40)
        ]
    );
}
