//! Part 4A's acceptance by the frozen record (lane v235_licence): the
//! eight conventions BARS section 3 passed are entries of the product
//! now, so on every corpus at its pinned tip, with the corpus's own
//! `ce.toml` and no literal added, this build's `ce deadcode` must report
//! exactly population B's `still_dead` rows — the set generation 1
//! simulated through `[graph] entry_globs` literals, read by the
//! instrument's own position reader. No new truth: the doc is frozen.
//!   CE_DEADFILE_ROOT=<clones> CE_DEADFILE_COPIES=<scratch dir> cargo test --test it -- --ignored eval_deadfile_parts::audit::entries::the_build_reports_population_b --nocapture

use super::derived::population_path;
use super::population::as_it_is;
use crate::common::ledger::out_file;
use crate::eval_deadfile_parts::corpora;
use crate::eval_support::load;
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
#[ignore = "copies the corpus clones and runs ce deadcode on the copies"]
fn the_build_reports_population_b() {
    let doc = load(&out_file("CE_DEADFILE_OUT", &population_path()));
    let rows = doc["corpora"].as_array().expect("corpora");
    assert_eq!(rows.len(), corpora().len(), "one row per corpus");
    println!("| corpus | dead before (frozen slice) | population B still_dead | this build |");
    println!("|---|---|---|---|");
    let mut wrong = Vec::new();
    for ((corpus, _), row) in corpora().into_iter().zip(rows) {
        assert_eq!(row["corpus"], corpus);
        let want: BTreeSet<String> = row["still_dead"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect();
        let (_, _, got) = as_it_is(corpus);
        println!(
            "| {corpus} | {} | {} | {} |",
            row["before"],
            want.len(),
            got.len()
        );
        if got != want {
            let extra: Vec<&String> = got.difference(&want).collect();
            let missing: Vec<&String> = want.difference(&got).collect();
            wrong.push(format!(
                "{corpus}: dead but not in B {extra:?}; in B but not dead {missing:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
