//! The dead-file exam, generation 2 (lane v235_licence, part 4B),
//! sections 3 and 4 and the record: the two readings and reading C, the
//! bars and their four outcomes, the keyed audit draw, the record's bytes
//! and the registry. Section 2's definitions are fixed in
//! eval_deadfile_gen2.rs; the report and the run's rules in
//! eval_deadfile_gen2_run.rs.

use crate::common::repo_root;
use crate::eval_deadfile_gen2_parts::audit::{
    Row, drawn, names_component, reading_c, reading_c_twice,
};
use crate::eval_deadfile_gen2_parts::model::{Event, Fate, Kind, State};
use crate::eval_deadfile_gen2_parts::readings::{
    Outcome, Reading, Table, blocks, outcome, sentence, share, tally,
};
use crate::eval_deadfile_gen2_parts::{
    MIN_FIRST_PARENT, REGISTRY_SCHEMA, corpora, pinned, product_commit, registry,
};
use crate::eval_support::content_sha;

/// `(fates, (false, denominator) under A, under B)`.
#[test]
fn the_two_readings() {
    let f = [
        Fate::ReReferred,
        Fate::KeptUp { focused: true },
        Fate::KeptUp { focused: false },
        Fate::RemovedUntouched,
        Fate::Undetermined,
    ];
    assert_eq!(share(Reading::A, &f), (4, 5));
    assert_eq!(share(Reading::B, &f), (2, 3));
    // reading C (G1), exact: B is 2 / 3, the one undetermined added at an
    // audited share of 1 / 2 gives (2 * 2 + 1 * 1) / ((3 + 1) * 2) = 5 / 8
    assert_eq!(reading_c(&f, 1, 2), Some((5, 8)));
    assert_eq!(reading_c(&f, 2, 2), Some((6, 8)));
    assert_eq!(
        reading_c(&f, 0, 0),
        None,
        "undetermined and nothing audited"
    );
    let decided = &f[..4];
    assert_eq!(reading_c(decided, 0, 0), Some(share(Reading::B, decided)));
    // three audited, one in use, one unsure: unsure counted in use, then not
    assert_eq!(reading_c_twice(&f, 1, 1, 3), [Some((8, 12)), Some((7, 12))]);
}

/// A table of `corpus state kind removed false` rows, `;` between rows:
/// `false` re-referred fates and `removed` removed-untouched ones.
fn table(spec: &str) -> Table {
    use {Kind::*, State::*};
    let mut t = Table::new();
    for row in spec.split(';') {
        let w: Vec<&str> = row.split_whitespace().collect();
        let [corpus, state, kind, removed, bad] = w[..] else {
            panic!("table row {row:?}")
        };
        let state = if state == "orphaned" { Orphaned } else { Born };
        let kind = if kind == "document" { Document } else { Code };
        let n = |s: &str| s.parse::<usize>().expect("count");
        let ev = |path: String| Event {
            at: 1,
            path,
            state,
            kind,
        };
        let evs: Vec<(Event, Fate)> = (0..n(removed))
            .map(|i| (ev(format!("r{i}")), Fate::RemovedUntouched))
            .chain((0..n(bad)).map(|i| (ev(format!("b{i}")), Fate::ReReferred)))
            .collect();
        tally(corpus, &evs, &mut t);
    }
    t
}

/// `rows => state outcome`, one case a line; `#` lines say why.
const BARS: &str = "
# 2 false in 100 misses the 1.0 % bar
p orphaned code 98 2 => orphaned Listed
# 99 events: not judged
p born code 99 0 => born NotJudged
# pooled 2 / 300 passes, but one corpus of 30 events at 2 false (6.7 %) is above 5.0 %
p orphaned code 270 0; q orphaned code 28 2 => orphaned Listed
# the same corpus with 29 events is below the corpus floor: blocks
p orphaned code 271 0; q orphaned code 27 2 => orphaned FailsByDefault
# a kind with 100 events of its own answers the bars by itself
p orphaned code 400 0; q orphaned document 98 2 => orphaned Listed";

/// 1 false in 100: at most 1.0 %, blocks under both readings.
#[test]
fn one_false_in_a_hundred_blocks() {
    let t = table("p orphaned code 99 1");
    assert_eq!(
        (
            blocks(&t, State::Orphaned, Reading::A),
            outcome(&t, State::Orphaned)
        ),
        (Some(true), Outcome::FailsByDefault)
    );
}

/// Every row of `BARS` reads the outcome it names.
#[test]
fn the_bars_read_as_their_table_says() {
    use State::*;
    let cases = BARS.lines().filter_map(|l| l.split_once(" => "));
    let got: Vec<String> = cases
        .clone()
        .map(|(rows, want)| {
            let state = if want.starts_with("orphaned") {
                Orphaned
            } else {
                Born
            };
            format!("{rows} => {}", want.split(' ').next().unwrap_or(""))
                + &format!(" {:?}", outcome(&table(rows), state))
        })
        .collect();
    let want: Vec<String> = cases
        .map(|(rows, want)| format!("{rows} => {want}"))
        .collect();
    assert_eq!(got, want);
}

/// A and B disagree: an undetermined counts false under A only, which is
/// the owner's question.
#[test]
fn an_undetermined_parts_the_readings() {
    use {Kind::*, State::*};
    let mut t = table("p born code 100 0");
    let ev = Event {
        at: 1,
        path: "u".into(),
        state: Born,
        kind: Code,
    };
    tally(
        "p",
        &[
            (ev.clone(), Fate::Undetermined),
            (ev.clone(), Fate::Undetermined),
        ],
        &mut t,
    );
    assert_eq!(outcome(&t, Born), Outcome::OwnerQuestion);
}

/// Each of the four outcomes has its sentence, led by the state.
#[test]
fn the_four_outcomes_each_have_a_sentence() {
    for o in [
        Outcome::FailsByDefault,
        Outcome::Listed,
        Outcome::OwnerQuestion,
        Outcome::NotJudged,
    ] {
        assert!(sentence("orphaned", o).starts_with("orphaned "), "{o:?}");
    }
}

#[test]
fn the_audit_draws_by_key_and_reads_whole_components() {
    let rows: Vec<Row> = (0..70)
        .map(|i| Row {
            corpus: "p".into(),
            commit: format!("c{i}"),
            path: "a.py".into(),
            state: State::Born,
        })
        .collect();
    let picked = drawn(&rows, State::Born);
    assert_eq!(picked.len(), 60);
    assert_eq!(drawn(&rows, State::Orphaned), Vec::<Row>::new());
    let first = rows
        .iter()
        .min_by_key(|r| crate::eval_deadfile_gen2_parts::audit::key(r))
        .expect("row");
    assert_eq!(&picked[0], first);
    assert!(names_component(
        "sphinx reads docs/conf.py at build",
        "docs/conf.py"
    ));
    assert!(
        !names_component("a myconf.py loader", "docs/conf.py"),
        "N8: no substring match"
    );
}

#[test]
fn the_record_is_the_main_sessions_bytes() {
    for (rel, sha) in pinned() {
        let text =
            std::fs::read_to_string(repo_root().join("contracts/eval").join(rel)).expect(rel);
        assert_eq!(content_sha(&text), sha, "{rel}: not the copied bytes");
    }
}

#[test]
fn the_registry_holds_section_one_before_any_count() {
    let reg = registry();
    assert_eq!(reg["schema"], REGISTRY_SCHEMA);
    let rows = reg["corpora"].as_array().expect("corpora");
    assert_eq!(rows.len(), corpora().len());
    for (row, (lang, first, replacement)) in rows.iter().zip(corpora()) {
        assert_eq!(
            (
                row["language"].as_str(),
                row["first"].as_str(),
                row["replacement"].as_str()
            ),
            (Some(lang), Some(first), Some(replacement))
        );
        let chosen = row["chosen"].as_str().expect("chosen");
        assert!(
            chosen == first || (chosen == replacement && row["replaced_because"].is_string()),
            "{lang}"
        );
        let tip = row["tip"].as_str().expect("tip");
        assert!(oid(tip), "{lang}: {tip}");
        let n = row["first_parent_commits"].as_u64().expect("count");
        assert!(
            chosen != first || n >= MIN_FIRST_PARENT,
            "{lang}: {n} < {MIN_FIRST_PARENT}"
        );
    }
    if let Some(c) = product_commit(&reg) {
        assert!(oid(c), "{c}");
    }
}

/// A full commit id: forty hex digits.
fn oid(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
