//! The ledger half of the FPR replay (fpr_replay.rs): what one
//! intercept row records — its reading against the parent baseline
//! and its fate once the commit is whole — and the report that folds
//! the rows into the numbers docs/FPR-REPLAY.md quotes. Split from the
//! walk when the landed measurement pushed the instrument past the
//! file budget.

use std::collections::BTreeSet;

pub mod channel;

/// What the parent baseline says about one intercept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Class {
    /// the parent shared nothing with this twin: the pair is new here
    New,
    /// the parent shared less: the edit grew the duplication
    Extension,
    /// the parent shared as much or more: the run moved, nothing grew
    Drift,
}

impl Class {
    /// Parent-shared against child-shared tokens with one twin.
    pub fn of(parent: usize, child: usize) -> Class {
        match (parent, child) {
            (0, _) => Class::New,
            (p, c) if c > p => Class::Extension,
            _ => Class::Drift,
        }
    }
}

/// One intercept row; the per-language ledger freezes it as written
/// (fpr_lang_replay.rs), the standing report prints it.
#[derive(serde::Serialize)]
pub struct Intercept {
    pub commit: String,
    #[serde(rename = "file")]
    pub rel: String,
    pub twin: String,
    pub parent: usize,
    pub child: usize,
    pub novel: usize,
    pub class: Class,
    pub refire: bool,
    pub twin_status: Option<char>,
    /// Whether the run with this twin is still in the tree once the
    /// whole commit is applied (probed again against the child state).
    /// `false` is the write-first move mid-state — split, fold, rename
    /// — the K round arbitrated pair by pair with two-file fixtures.
    pub landed: bool,
    /// The product's relocation stage credited the event at its moment
    /// in the walk (v2.35 GD4, channel.rs): the hook would not refuse it.
    pub relocated: bool,
    /// The commit as a whole did not raise the count of this row's runs
    /// (order-free, channel.rs).
    pub count_neutral: bool,
    /// The twin is a file this commit wrote EARLIER in the walk's order —
    /// two copies born in one commit, the channel the PostToolUse
    /// re-measure (GD7) makes the product see.
    pub born_together: bool,
    /// The matched runs with this twin (the channel's input; not frozen).
    #[serde(skip)]
    pub runs: Vec<Vec<u64>>,
    /// The commit's files applied before this event (not frozen).
    #[serde(skip)]
    pub applied_before: BTreeSet<String>,
}

impl Intercept {
    pub fn row(&self) -> String {
        format!(
            "{} {} -> {}  shared {}→{} novel {}  {:?} {}{} {}",
            self.commit,
            self.rel,
            self.twin,
            self.parent,
            self.child,
            self.novel,
            self.class,
            if self.refire { "refire" } else { "first" },
            self.twin_status
                .map(|s| format!(" twin:{s}"))
                .unwrap_or_default(),
            if self.landed { "landed" } else { "mid-state" },
        ) + &format!(
            "{}{}{}",
            if self.relocated { " relocated" } else { "" },
            if self.count_neutral { " count-neutral" } else { "" },
            if self.born_together { " born-together" } else { "" },
        )
    }
}

/// Unordered file pair — the key a fire is remembered under.
pub fn pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.into(), b.into())
    } else {
        (b.into(), a.into())
    }
}

pub fn report(events: usize, rows: &[Intercept]) {
    let event_of = |r: &Intercept| (r.commit.clone(), r.rel.clone());
    let hit: BTreeSet<_> = rows.iter().map(event_of).collect();
    let landed_events: BTreeSet<_> = rows.iter().filter(|r| r.landed).map(event_of).collect();
    println!(
        "replayed {events} edit events, {} intercepted, {} (event, twin) rows:",
        hit.len(),
        rows.len()
    );
    for r in rows {
        println!("  INTERCEPT {}", r.row());
    }
    let n = |pred: &dyn Fn(&Intercept) -> bool| rows.iter().filter(|r| pred(r)).count();
    let fate = |s: char| n(&|r| r.twin_status == Some(s));
    println!(
        "classes: new {} / extension {} / drift {}; first fires {} / re-fires {}; twin changed in the same commit {} (added {} / modified {} / deleted {})",
        n(&|r| r.class == Class::New),
        n(&|r| r.class == Class::Extension),
        n(&|r| r.class == Class::Drift),
        n(&|r| !r.refire),
        n(&|r| r.refire),
        n(&|r| r.twin_status.is_some()),
        fate('A'),
        fate('M'),
        fate('D'),
    );
    println!(
        "landed: {} rows still duplicate once the commit is whole ({} events); {} rows are write-first mid-states ({} with the twin changed in the same commit, {} re-fires among them; {} events raised nothing else)",
        n(&|r| r.landed),
        landed_events.len(),
        n(&|r| !r.landed),
        n(&|r| !r.landed && r.twin_status.is_some()),
        n(&|r| !r.landed && r.refire),
        hit.len() - landed_events.len(),
    );
    let events_where = |pred: &dyn Fn(&Intercept) -> bool| {
        rows.iter().filter(|r| pred(r)).map(event_of).collect::<BTreeSet<_>>().len()
    };
    println!(
        "relocation channel (v2.35 GD4): {} events the product's stage credits at their moment in the walk; {} rows count-neutral over the whole commit; {} rows born together with their twin in one commit",
        events_where(&|r| r.relocated),
        n(&|r| r.count_neutral),
        n(&|r| r.born_together),
    );
    let per500 = |k: usize| k as f64 * 500.0 / events.max(1) as f64;
    println!(
        "rate: {:.2} intercepted events per 500 strict; {:.2} landed events per 500; {:.2} mid-state rows per 500 when every write-first move counts as a false intercept (the whole-write framing) — a landed row is a verified run the parent did not have, the ledger's true-positive class",
        per500(hit.len()),
        per500(landed_events.len()),
        per500(n(&|r| !r.landed)),
    );
}
