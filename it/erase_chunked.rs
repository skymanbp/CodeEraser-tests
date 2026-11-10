//! `ce erase` past erase/1's row ceiling (plan v2.35 T5): the plan's
//! rows go to the core in requests of at most `erase_rows` rows, split
//! by path the way scan/1 splits by file (erase::wire::chunks) — the
//! closure chooses among one path's rows, so a path never straddles a
//! request. Two legs: a tree over the shipped ceiling plans (it was
//! refused by name before), and on a tree under it the plan document
//! laid out after any split that holds every path is the same bytes as
//! the one-request plan.

use crate::common::{core_bin, trees};
use codeeraser::erase;
use codeeraser::erase::document::{self, Diffs, Run};

/// The plan document at request ceiling `cap`, as bytes.
fn document_at(dir: &std::path::Path, core: &str, cap: usize) -> String {
    let (plan, held) = erase::planned_within(dir, None, core, cap).expect("plan");
    let diffs = Diffs::of(dir, &plan).expect("diffs");
    let answer = document::answer(core, held, &plan, &diffs, Run::default()).expect("document");
    answer.document.to_string()
}

/// Dead scripts, a verbatim paragraph twice (a span row beside a whole
/// file) and a live import: rows of every shape over several paths.
const TREE: &str = "mixed|
@main.py
import helper
helper.run()
@helper.py
def run():
    return 1
@README.md
# fixture

[notes](notes.md)

alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty apple pear plum grape melon berry stone river cloud
@notes.md
# notes

alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty apple pear plum grape melon berry stone river cloud
@tools/one.py
print('one')
@tools/two.py
print('two')
@tools/three.py
print('three')
@tools/four.py
print('four')
";

#[test]
fn a_split_plan_is_the_one_request_plan_byte_for_byte() {
    let core = core_bin();
    let (name, _, files) = trees::trees(TREE).remove(0);
    let dir = trees::seeded(&format!("erase-chunked-{name}"), &files);
    let (plan, _) = erase::planned(&dir, None, &core).expect("plan");
    let rows = plan.rows.len();
    assert!(rows >= 5, "the fixture plans rows on several paths: {rows}");
    let whole = document_at(&dir, &core, 1 << 20);
    // the smallest ceiling that holds every path, then each one above it
    let floor = (1..=rows)
        .find(|&cap| erase::planned_within(&dir, None, &core, cap).is_ok())
        .expect("some ceiling holds every path");
    assert!(floor < rows, "the fixture splits: floor {floor} of {rows}");
    for cap in floor..=rows {
        assert_eq!(document_at(&dir, &core, cap), whole, "cap {cap}");
    }
}

#[test]
fn a_tree_over_the_shipped_ceiling_plans() {
    let core = core_bin();
    let cap = codeeraser::tables::get().limits.caps.erase_rows;
    let files: Vec<(String, String)> = (0..cap + 8)
        .map(|i| (format!("s/{i:05}.py"), format!("print({i})\n")))
        .collect();
    let dir = trees::seeded("erase-chunked-over", &files);
    let plan = erase::plan(&dir, None, &core).expect("over the ceiling plans");
    let dead = plan.rows.iter().filter(|r| r.class == "dead_file").count();
    assert!(dead > cap, "{dead} dead_file rows past the ceiling {cap}");
    assert_eq!(plan.counts.candidates, plan.rows.len());
}
