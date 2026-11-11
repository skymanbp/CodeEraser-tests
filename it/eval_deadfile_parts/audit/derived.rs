//! The exam's derived docs (part 3A): what needs the pinned trees is
//! read once off the corpus clones and frozen, so the CI gate
//! (eval_deadfile_review.rs) re-derives everything else from the
//! committed record without them. `#[ignore]`: each needs the clones.
//!   CE_DEADFILE_ROOT=<clones> [CE_DEADFILE_OUT=<dir>] cargo test --test it -- --ignored eval_deadfile_parts::audit::derived::deadfile_review_docs --nocapture
//!   CE_DEADFILE_ROOT=<clones> CE_DEADFILE_COPIES=<scratch dir> [CE_DEADFILE_OUT=<dir>] cargo test --test it -- --ignored eval_deadfile_parts::audit::derived::deadfile_population_doc --nocapture
//!   CE_DEADFILE_ROOT=<clones> cargo test --test it -- --ignored eval_deadfile_parts::audit::derived::deadfile_reviews_rederive --nocapture

use super::conventions::{candidates, carries, evaluate as section3};
use super::population::{Moved, line, population_b};
use super::record::convention_docs;
use super::review::{clone_facts, clone_reader};
use super::tables::{REVIEW, per_language, reviewed};
use crate::common::ledger::out_file;
use crate::eval_deadfile_parts::{classes::constants, corpora, doc_path};
use crate::eval_lang_parts::{
    freezing::frozen,
    generate::{env, freeze},
};
use serde_json::{Value, json};

pub const REVIEW_SCHEMA: &str = "ce.eval-deadfile-review/1.0.0";
pub const POPULATION_SCHEMA: &str = "ce.eval-deadfile-population-b/1.0.0";

const REVIEW_METHOD: &str = "per language, the class doc's primary rows in stored order (a reserve row \
    only where a first reading names it), each with its first and second reading from the committed \
    record, BARS section 1's check and addendum 1 B's tag of every reading read off the pinned tree, \
    the second-reading reasons, the final answer (a failing first answer replaced by a passing second \
    one; two different answers settled by the committed settlement, never by a vote) and its tag.";

const POPULATION_METHOD: &str = "BARS section 3 over the filed review docs and the committed citation \
    input gives the passing conventions; per corpus a copy of the pinned clone, ce deadcode of this \
    build with the corpus's own ce.toml (equal to the frozen slice), then with every file a passing \
    position rule matches appended to [graph] entry_globs as a literal; the run stops unless the rows \
    dropped are exactly the dead rows among the matched files and what they reach along the kept arcs.";

/// The population-B doc's path.
pub fn population_path() -> String {
    doc_path("deadfile-population", "b")
}

/// The passing §3 conventions over review docs: (name list, candidates).
pub fn passing(reviews: &[Value]) -> Vec<&'static str> {
    let rows: Vec<Value> = reviews
        .iter()
        .flat_map(|r| r["rows"].as_array().cloned().unwrap_or_default())
        .collect();
    let table = section3(&rows, &convention_docs());
    candidates()
        .into_iter()
        .filter(|c| {
            table
                .iter()
                .any(|t| t["name"] == c.name && t["passes"] == true)
        })
        .map(|c| c.name)
        .collect()
}

/// One corpus's population-B row.
fn corpus_row(corpus: &str, m: &Moved) -> Value {
    let beyond: Vec<&String> = m.reach_dead.difference(&m.matched_dead).collect();
    json!({
        "corpus": corpus, "before": m.before.len(), "after": m.after.len(),
        "matched": m.matched, "matched_dead": m.matched_dead, "reached_beyond": beyond, "still_dead": m.after,
    })
}

#[test]
#[ignore = "reads the corpus clones"]
fn deadfile_review_docs() {
    let tip = clone_reader(env("CE_DEADFILE_ROOT"));
    let (_, reviews) = reviewed(&clone_facts(&tip));
    for r in reviews {
        let lang = r["lang"].as_str().expect("lang").to_string();
        println!(
            "{lang}: {} rows, agreement {}",
            r["rows"].as_array().map_or(0, Vec::len),
            r["agreement"]["overall"]
        );
        freeze(
            &out_file("CE_DEADFILE_OUT", &doc_path(REVIEW, &lang)),
            &frozen(REVIEW_SCHEMA, REVIEW_METHOD, constants(), r),
        );
    }
}

#[test]
#[ignore = "copies the corpus clones and runs ce deadcode on the copies"]
fn deadfile_population_doc() {
    let names = passing(&per_language(REVIEW));
    let (docs, cands) = (convention_docs(), candidates());
    let pass: Vec<_> = cands.iter().filter(|c| names.contains(&c.name)).collect();
    let mut rows = Vec::new();
    for (corpus, _) in corpora() {
        let m = population_b(corpus, &|p| {
            pass.iter().any(|c| carries(c, &docs, corpus, p))
        });
        println!("{}", line(corpus, &m));
        rows.push(corpus_row(corpus, &m));
    }
    let doc = frozen(
        POPULATION_SCHEMA,
        POPULATION_METHOD,
        constants(),
        json!({"passing": names, "corpora": rows}),
    );
    freeze(&out_file("CE_DEADFILE_OUT", &population_path()), &doc);
}

/// The filed review docs, rebuilt with every check and tag read off the
/// clones again: equal, or the first difference named.
#[test]
#[ignore = "reads the corpus clones"]
fn deadfile_reviews_rederive() {
    let tip = clone_reader(env("CE_DEADFILE_ROOT"));
    let (_, rebuilt) = reviewed(&clone_facts(&tip));
    for (filed, mine) in per_language(REVIEW).iter().zip(&rebuilt) {
        for key in ["lang", "agreement", "rows"] {
            assert_eq!(
                filed[key], mine[key],
                "{}: `{key}` differs from the clones' reading",
                filed["lang"]
            );
        }
    }
    println!("{} review docs re-derived from the clones", rebuilt.len());
}
