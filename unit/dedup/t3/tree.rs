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
    let built = file_trees(text, Lang::Rust, &spans);
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
    let built = file_trees(text, Lang::Rust, &[(1, 2)]);
    assert!(matches!(built[0], Built::Forest(2)));
}
