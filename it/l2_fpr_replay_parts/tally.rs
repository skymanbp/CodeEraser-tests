//! Counts and derived rates for a complete changeset replay.

use super::Label;
use crate::common::stats::{clopper_pearson, permille, round3};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One corpus's ledger line.
#[derive(Default)]
pub struct Tally {
    pub name: String,
    pub tip: String,
    pub commits: usize,
    pub skipped: BTreeMap<&'static str, usize>,
    // Label × intercepted: the one source of every derived count.
    outcomes: [[usize; 2]; 3],
}

impl Tally {
    pub fn new(name: &str, tip: &str, commits: usize) -> Tally {
        Tally {
            name: name.to_string(),
            tip: tip.to_string(),
            commits,
            ..Self::default()
        }
    }

    /// One judged event: its label's denominator, and the two false
    /// readings / the miss the predicate produced.
    pub fn count(&mut self, label: Label, fired: bool) {
        self.outcomes[label as usize][usize::from(fired)] += 1;
    }

    fn population(&self, label: Label) -> usize {
        self.outcomes[label as usize].iter().sum()
    }

    fn caught(&self, label: Label) -> usize {
        self.outcomes[label as usize][1]
    }

    /// Field-for-field sum — the totals row is DERIVED, never typed.
    pub fn fold(name: &str, all: &[Tally]) -> Tally {
        let mut t = Tally::new(name, "", all.iter().map(|a| a.commits).sum());
        for a in all {
            for (why, n) in &a.skipped {
                *t.skipped.entry(why).or_default() += n;
            }
            for (sum, n) in t
                .outcomes
                .iter_mut()
                .flatten()
                .zip(a.outcomes.iter().flatten())
            {
                *sum += n;
            }
        }
        t
    }

    pub fn json(&self) -> Value {
        let normal = self.population(Label::Normal);
        let unreviewed = self.population(Label::Unreviewed);
        let abnormal = self.population(Label::Copy);
        let false_strict = self.caught(Label::Normal);
        let false_wide = false_strict + self.caught(Label::Unreviewed);
        let copied = self.caught(Label::Copy);
        let (slo, shi) = clopper_pearson(normal, false_strict);
        let (wlo, whi) = clopper_pearson(normal + unreviewed, false_wide);
        json!({
            "corpus": self.name, "tip": self.tip, "commits": self.commits,
            "events": normal + unreviewed + abnormal, "skipped": self.skipped,
            "normal": normal, "unreviewed": unreviewed,
            "abnormal": abnormal, "intercepts": false_wide + copied,
            "false_strict": false_strict, "false_wide": false_wide,
            "misses": abnormal - copied,
            "recall_permille": (abnormal > 0).then(|| permille(copied, abnormal)),
            "strict_permille": permille(false_strict, normal),
            "wide_permille": permille(false_wide, normal + unreviewed),
            "cp95_strict": [round3(slo), round3(shi)],
            "cp95_wide": [round3(wlo), round3(whi)],
        })
    }
}
