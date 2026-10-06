//! The W7 differential's binder legs (plan v2.33 W7; mounted inside the
//! frozen binder): the faces whose strings are plain lists (`Lists`,
//! one per class), trend's `short` commit, every face's path order
//! (`ranks`) and console lines of any shape, each spelled by the core
//! and by the frozen binder.

use super::super::kit::{Leg, leg};
use super::{Lists, ranks};
use serde_json::{Value, json};

/// The plain-list faces: family, the classes it sends.
const FACES: [(&str, &[&str]); 6] = [
    ("dedup", &["path"]),
    ("clone", &["unit"]),
    ("clone-units", &["path", "key"]),
    ("scan", &["path", "fn"]),
    ("sites", &["path", "site_spec", "site_owner"]),
    (
        "erase-trail",
        &["path", "provenance", "hash", "plan", "unreadable"],
    ),
];

/// Each list's class and size: the references a face's lists answer.
fn sizes(l: &Lists) -> Vec<(&'static str, Vec<usize>)> {
    l.0.iter().map(|(c, l)| (*c, vec![l.len()])).collect()
}

/// A face's lists as the frozen face held them and as it sends them.
fn lists(leg: &mut Leg, classes: &[&'static str]) -> (Lists, Value) {
    let held: Vec<(&'static str, Vec<String>)> =
        (classes.iter()).map(|c| (*c, leg.words(9))).collect();
    let sent: serde_json::Map<String, Value> = (held.iter())
        .map(|(c, l)| (c.to_string(), json!(l)))
        .collect();
    (Lists(held), Value::Object(sent))
}

leg!(
    plain_lists_spell_as_the_frozen_binder,
    "binder lists",
    0x7701,
    |leg| {
        let (family, classes) = FACES[leg.rng.below(FACES.len())];
        let (frozen, sent) = lists(leg, classes);
        let mut drawn = sizes(&frozen);
        // deadcode's `node_name` rule reads any family's strings: none here
        drawn.push(("node_name", vec![4]));
        leg.spell_drawn(family, &sent, &frozen, &drawn);
    }
);

/// A commit as git spells it: forty hex digits.
fn commit(leg: &mut Leg) -> String {
    (0..40)
        .map(|_| "0123456789abcdef".as_bytes()[leg.rng.below(16)] as char)
        .collect()
}

leg!(
    trend_spells_its_short_commit_as_the_frozen_face,
    "binder trend",
    0x7702,
    |leg| {
        let commits = leg.many(6, commit);
        let short = commits.iter().map(|c| c[..12].to_string()).collect();
        let (sha, reason) = (leg.words(4), leg.words(4));
        let sent = json!({"commit": commits, "sha": sha, "reason": reason});
        let n = [commits.len(), sha.len()];
        let frozen = Lists(vec![
            ("commit", commits),
            ("short", short),
            ("sha", sha),
            ("reason", reason),
        ]);
        let drawn = [
            ("commit", vec![n[0]]),
            ("short", vec![n[0]]),
            ("sha", vec![n[1]]),
            ("reason", vec![n[1]]),
        ];
        leg.spell_drawn("trend", &sent, &frozen, &drawn);
    }
);

/// The table a family's request carried as its paths' order.
const RANKED: [(&str, &str); 3] = [
    ("sites", "rankFiles"),
    ("flow", "rankFiles"),
    ("join", "rankPaths"),
];

leg!(
    the_path_order_is_measured_as_the_frozen_ranks,
    "binder ranks",
    0x7703,
    |leg| {
        let (family, table) = RANKED[leg.rng.below(RANKED.len())];
        let mut paths = leg.paths(0, 12);
        if !paths.is_empty() && leg.rng.chance(30) {
            paths.push(paths[leg.rng.below(paths.len())].clone());
        }
        let want: Vec<[usize; 2]> = (ranks(paths.iter().map(String::as_str)).into_iter())
            .enumerate()
            .map(|(i, r)| [i, r])
            .collect();
        let sent = json!({"path": paths});
        let got = leg.ask(family, (&sent, &json!({})), json!({}));
        leg.table(&sent, &json!(want), &got["rows"][table]);
    }
);

/// A line of any shape: words and holes, one reference per hole give
/// or take one, now and then something that is no reference.
fn any_line(leg: &mut Leg, frozen: &Lists) -> Value {
    let pieces = 1 + leg.rng.below(4);
    let text: Vec<String> = (0..pieces).map(|_| leg.word()).collect();
    let text = text.join("{}");
    let holes = text.matches("{}").count();
    let n = match leg.rng.below(10) {
        0 => holes + 1,
        1 => holes.saturating_sub(1),
        _ => holes,
    };
    let drawn = sizes(frozen);
    let mut line = vec![json!(leg.rng.below(2)), json!(text)];
    for r in leg.references(&drawn, n) {
        line.push(match leg.rng.below(30) {
            0 => json!({"$": r, "x": 1}),
            1 => r,
            _ => json!({"$": r}),
        });
    }
    Value::Array(line)
}

leg!(
    lines_of_any_shape_bind_as_the_frozen_binder,
    "binder lines",
    0x7704,
    |leg| {
        let (frozen, sent) = lists(leg, &["path", "fn"]);
        let lines = (0..200).map(|_| any_line(leg, &frozen)).collect();
        leg.lines("scan", &sent, &frozen, lines);
    }
);
