//! The per-language ledger's EXECUTOR (plan v2.30 step 6 commit E;
//! booklet language-expansion.md §14 item 9): a language plan v2.30
//! adds ships its judged bit only on its own ledger — the six rows of
//! contracts/eval/fpr-lang-v1.json, one corpus per language, each read
//! at or under the §4.2 line — and docs/FPR-REPLAY.md quotes that
//! table. The rule is the booklet's; this leg is what executes it, the
//! way fpr_zone_gate executes the zone's. Plus one e2e over a synthetic
//! two-language history, so the language filter itself is tested and
//! not only its record.

use crate::common::ledger::{closes, count, intercepts, rate_pair, two_commits};
use crate::common::{chain_of, tmp};
use crate::eval_support::lang_of;
use crate::fpr_lang_replay::{Fold, GATE_PPM, LEDGER, Reading, corpora, fold, table};
use crate::fpr_replay::replay;
use codeeraser::scan::lang::Lang;
use serde_json::Value;

/// A row admits its language when the class can fire on it and the
/// row reads at or under the line. A language that never fingerprints
/// (HTML) raises no event by construction: its row must say zero, and
/// it is admitted by that name — the class has nothing to intercept.
fn admitted(lang: Lang, c: &Value) -> bool {
    if !lang.fingerprints() {
        assert_eq!(
            count(c, "events"),
            0,
            "{}: a language that never fingerprints raises no event",
            c["name"]
        );
        return true;
    }
    assert!(
        count(c, "events") > 0,
        "{}: no event in the window — a ledger read over nothing licenses nothing",
        c["name"]
    );
    count(c, "rate_ppm") <= GATE_PPM
}

/// The six rows in registry order, each coherent and each licensing
/// exactly its language's judged bit; and docs/FPR-REPLAY.md quoting
/// the table as the instrument prints it.
#[test]
fn every_new_language_ships_only_on_its_own_ledger() {
    let registry = corpora();
    let names: Vec<&str> = registry.iter().map(|(_, n)| *n).collect();
    let rows = LEDGER.rows(&names);
    LEDGER.page_quotes("docs/FPR-REPLAY.md", &table(&rows));
    for (c, (code, _)) in rows.iter().zip(registry) {
        assert_eq!(c["lang"], code, "{}: the row's language", c["name"]);
        coherent(c);
        let lang = lang_of(code);
        let judged = Lang::judged_mask() & (1 << lang as i64) != 0;
        assert_eq!(
            judged,
            admitted(lang, c),
            "{code}: the judged bit and the ledger disagree — booklet §14 item 9: a language \
             ships only on its own ledger, read at or under the §4.2 line"
        );
    }
}

/// The row's derived numbers re-derived from its frozen rows: the
/// event fold, both rates and both bounds.
fn coherent(c: &Value) {
    let cell = |r: &Value, k: &str| r[k].as_str().unwrap_or_default().to_string();
    let f = fold(intercepts(c).iter().map(|r| {
        (
            cell(r, "commit"),
            cell(r, "file"),
            r["landed"] == true,
            !r["twin_status"].is_null(),
        )
    }));
    let events = count(c, "events") as usize;
    assert!(
        f.intercepted <= events,
        "{}: more intercepted events than events",
        c["name"]
    );
    closes(
        c,
        &[
            ("intercepted", f.intercepted as u64),
            ("landed", f.landed as u64),
            ("touched", f.touched as u64),
            ("false_events", f.false_events() as u64),
        ],
    );
    closes(
        c,
        &rate_pair(f.false_events(), events, ("rate_ppm", "cp_upper_ppm")),
    );
    closes(
        c,
        &rate_pair(f.mid_state(), events, ("strict_ppm", "strict_cp_upper_ppm")),
    );
}

/// A C function past the probe's guarantee (t = 50 tokens), copied
/// whole by the second commit: the one duplicate the history holds.
const FOLD: &str = "int fold(const int *input, int count, int limit) {\n    int total = 0;\n    \
    for (int i = 0; i < count; i++) {\n        if (input[i] > limit) {\n            \
    total += input[i] * 3 + 7;\n        } else {\n            total -= input[i] / 3;\n        }\n    \
    }\n    return total;\n}\n";
/// The seed: the C function and a Lua file.
const SEED: &[(&str, &str)] = &[
    ("a.c", FOLD),
    ("b.lua", "local function grow(t)\n  return #t + 1\nend\n"),
];
/// The second commit: the C function copied whole into a new file, the
/// Lua file edited.
const THEN: &[(&str, &str)] = &[
    ("c.c", FOLD),
    ("b.lua", "local function grow(t)\n  return #t + 2\nend\n"),
];

/// A synthetic two-commit history in two languages: the second commit
/// copies the C function into a new file (a C event that intercepts
/// and lands, twin a.c) and edits the Lua file (a Lua event that
/// duplicates nothing). Named for C, the walk counts one event and
/// one row; named for Lua, one event and no row; unnamed, both.
#[test]
fn a_synthetic_history_reads_only_the_named_language() {
    let repo = two_commits("fpr-lang-e2e", SEED, THEN);
    let commits = chain_of(&repo, "HEAD", None);
    let read = |lang| {
        let (events, rows) = replay(&repo, &tmp("fpr-lang-e2e-shadow"), lang, &commits);
        Reading { events, rows }
    };
    let c = read(Some(Lang::C));
    let landed = Fold {
        intercepted: 1,
        landed: 1,
        touched: 0,
    };
    assert_eq!(
        (c.events, c.folded(), c.rows[0].twin.as_str()),
        (1, landed, "a.c"),
        "the C copy is the one C event, intercepted and landed"
    );
    let l = read(Some(Lang::Lua));
    assert_eq!(
        (l.events, l.rows.len()),
        (1, 0),
        "the Lua edit is the one Lua event and duplicates nothing"
    );
    let all = read(None);
    assert_eq!(
        (all.events, all.rows.len()),
        (2, 1),
        "unnamed, both files are events"
    );
}
