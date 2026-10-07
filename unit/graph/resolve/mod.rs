//! The configuration readers' differential gate (plan v2.33 W2-text
//! stage A; R's DESCRIPTION since stage B, the .cabal since stage D, the
//! JSONC reader, the tsconfig chain and the package.json since stage E,
//! the Cargo.toml since stage F, the Markdown rungs' lowercase, label
//! fold and target readings since stage G, the HTML rungs' reference and
//! page readings since stage H): the frozen readers
//! (unit/graph/oracle_cfg/, mounted at their old paths under cfg(test))
//! against the core's readers, asked directly through resolve/1's
//! `inspect` object (CE.Resolve.Inspect).
//! Every leg is `#[ignore]`d — it needs a core — and draws at least ten
//! thousand seeded questions (CE_RESOLVE_DIFF_SEED moves the seed;
//! CE_RESOLVE_DIFF_N the count), some of them real configuration texts
//! of the evaluation corpora mutated (a byte-order mark, CRLF, comments,
//! continuations, malformed lines, non-ASCII, empty); the character
//! classes are compared over every scalar value. A leg prints its tally
//! and fails on the first disagreement count above zero.

mod cabal;
mod cargo;
mod description;
mod draw;
mod files;
mod html;
mod md;
mod text;
mod ts;
mod ts_gen;

use crate::corelink::{Link, judged};
use draw::Draw;
use serde_json::{Value, json};

/// The questions one request carries at most.
const BATCH: usize = 500;

/// The leg's question count (at least ten thousand unless overridden).
fn count() -> usize {
    std::env::var("CE_RESOLVE_DIFF_N")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(10_000)
}

/// A word list written as one literal, its words parted by `¦` (an
/// empty word is an empty part).
fn table(words: &'static str) -> Vec<&'static str> {
    words.split('¦').collect()
}

/// A leg's questions: `count()` of them, each drawn from the leg's own
/// seeded stream.
fn questions(seed: u64, mut ask: impl FnMut(&mut Draw, usize) -> Value) -> Vec<Value> {
    let mut d = Draw::seeded(seed);
    (0..count()).map(|i| ask(&mut d, i)).collect()
}

/// The frozen reader's answer to each question against the core's, under
/// the `inspect` key `leg`.
fn check(leg: &str, questions: &[Value], oracle: impl FnMut(&Value) -> Value) {
    let want: Vec<Value> = questions.iter().map(oracle).collect();
    agree(leg, questions, &want, &inspect(&mut link(), leg, questions));
}

/// The core's answers to one `inspect` key, the questions sent in
/// batches, answers in question order.
fn inspect(link: &mut Link, key: &str, questions: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for batch in questions.chunks(BATCH) {
        let body = json!({ "inspect": { key: batch } });
        let reply = judged::ask(link, "resolve/1", "9.0.0", "resolve", body)
            .unwrap_or_else(|e| panic!("resolve/1 inspect {key}: {e}"));
        let answers = reply["inspected"][key]
            .as_array()
            .unwrap_or_else(|| panic!("inspect {key}: no answer in {reply}"))
            .clone();
        assert_eq!(
            answers.len(),
            batch.len(),
            "inspect {key}: one answer per question"
        );
        out.extend(answers);
    }
    out
}

fn link() -> Link {
    Link::open(crate::tables::core_flag())
        .unwrap_or_else(|e| panic!("no core: {e}"))
        .0
}

/// Compare each question's oracle answer with the core's; report.
fn agree(leg: &str, questions: &[Value], want: &[Value], got: &[Value]) {
    let mut misses = 0;
    for ((q, w), g) in questions.iter().zip(want).zip(got) {
        if w != g {
            misses += 1;
            if misses <= 10 {
                println!("MISMATCH {leg}: {q}\n  oracle {w}\n  core   {g}");
            }
        }
    }
    println!(
        "== resolve_text {leg}: questions {} mismatches {misses} ==",
        questions.len()
    );
    assert_eq!(misses, 0, "{leg}: core and frozen reader disagree");
}
