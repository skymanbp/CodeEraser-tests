use super::*;
use crate::dedup::unitcache;
use crate::fourclass::units;

/// The same-source counterfactual (3e): tree walk and unitsig
/// fact walk select by one predicate, so over a real source every
/// unit's built tree must carry exactly unit_facts' node count —
/// and satisfy the wire contract the core machine-checks.
#[test]
fn tree_nodes_equal_unitsig_nodes_per_unit() {
    let text = "fn a(x: i64) -> i64 {\n    let y = x + 1;\n    y * 2\n}\n\
                    \nfn b() {\n    for i in 0..3 {\n        println!(\"{i}\");\n    }\n}\n";
    let facts = unitcache::unit_facts(text, Lang::Rust);
    assert!(!facts.is_empty());
    let segs = units::segments(text, Lang::Rust);
    let spans: Vec<_> = units::with_nth(&segs)
        .iter()
        .map(|(u, _)| (u.start_line, u.end_line))
        .collect();
    let built = file_trees(text, Lang::Rust, &spans, Extras::With);
    assert_eq!(built.len(), facts.len());
    for (f, b) in facts.iter().zip(&built) {
        let Built::Tree(t) = b else {
            panic!("{}: function units are single-rooted", f.key);
        };
        assert_eq!(t.lab.len() as i64, f.nodes, "{}", f.key);
        assert_eq!(*t.lld.last().expect("nonempty"), 0, "{}: root lld", f.key);
        for (i, &l) in t.lld.iter().enumerate() {
            assert!(0 <= l && l <= i as i64, "{}: lld[{i}] = {l}", f.key);
        }
    }
}

/// A span holding two sibling items has no single root — the
/// outcome is a ledgered Forest, never a guessed root.
#[test]
fn sibling_span_is_a_forest() {
    let text = "fn a() {}\nfn b() {}\nfn c() {}\n";
    let built = file_trees(text, Lang::Rust, &[(1, 2)], Extras::With);
    assert!(matches!(built[0], Built::Forest(2)));
}

/// merge/1's columns ride the same walk: a leaf hash is the fnv1a of
/// the leaf's own text and nothing else carries one; the spans, slots
/// and hashes are one per node.
#[test]
fn the_merge_columns_ride_the_walk() {
    let text = "fn a(x: i64) -> i64 {
    let y = x + 1;
    y * 2
}
";
    let [Built::Tree(t)] = &file_trees(text, Lang::Rust, &[(1, 4)], Extras::With)[..] else {
        panic!("one tree");
    };
    let n = t.lab.len();
    assert_eq!((t.leaf.len(), t.slot.len(), t.spans.len()), (n, n, n));
    for i in 0..n {
        let (s, e) = t.spans[i];
        let own = crate::dedup::tokens::fnv1a(&text.as_bytes()[s..e]);
        let leaf = t.lld[i] == i as i64;
        assert_eq!(t.leaf[i], if leaf { own } else { 0 }, "node {i}");
        assert!(!leaf || t.leaf[i] != 0, "node {i}: a leaf hashes");
    }
    assert_eq!(
        t.spans[n - 1],
        (0, text.len() - 1),
        "the root spans the unit"
    );
}

/// merge/1's trees leave the comments out (step-7 ruling 3): over a
/// unit holding a comment, the tree without extras is the tree with them
/// less exactly the comment nodes, every other column in step.
#[test]
fn a_tree_without_extras_is_the_tree_less_its_comments() {
    let text = "fn a(x: i64) -> i64 {\n    let y = x + 1; // one\n    // two\n    y * 2\n}\n";
    let one = |extras| match &file_trees(text, Lang::Rust, &[(1, 5)], extras)[..] {
        [Built::Tree(t)] => t.clone(),
        _ => panic!("one tree"),
    };
    let (with, without) = (one(Extras::With), one(Extras::Without));
    let comment = crate::dedup::struct_fp::kind_code("line_comment");
    let kept: Vec<usize> = (0..with.lab.len())
        .filter(|&i| with.lab[i] != comment)
        .collect();
    assert_eq!(with.lab.len() - kept.len(), 2, "two comments");
    assert_eq!(without.lab.len(), kept.len());
    let at = |i: i64| kept.iter().position(|&k| k as i64 == i).map(|p| p as i64);
    for (new, &old) in kept.iter().enumerate() {
        let row = (
            with.lab[old],
            with.leaf[old],
            with.slot[old],
            with.spans[old],
        );
        let got = (
            without.lab[new],
            without.leaf[new],
            without.slot[new],
            without.spans[new],
        );
        assert_eq!(got, row, "node {old}");
        assert_eq!(Some(without.lld[new]), at(with.lld[old]), "node {old}: lld");
    }
}
