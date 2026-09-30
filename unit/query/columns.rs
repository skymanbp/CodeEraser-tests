use super::*;
use crate::query::PRELUDE;

fn program(rules: Option<&str>, query: Option<&str>) -> Program {
    Program::lex(PRELUDE, rules, query).unwrap()
}

fn names(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_string()).collect()
}

#[test]
fn a_query_binds_its_columns_in_the_safety_walk_order() {
    let p = program(
        None,
        Some(
            "?- ref(N, M, K, _), not same_dir(N, M), C = count(X : depends(M, X)), C > 2, \
             lang(N, L), ref(N, M, K, R).",
        ),
    );
    assert_eq!(
        goals(&p),
        [GoalHead {
            clause: 8,
            kind: "query",
            name: None,
            columns: names(&["N", "M", "K", "C", "L", "R"]),
        }]
    );
}

#[test]
fn an_assertion_names_its_head_and_spells_its_terms_as_written() {
    let rules = "assert layered(A, B, _) :- dir_ref(A, B), dir_name(A, x).\n\
                 big(F) :- lines(F, N), N > 300.\n";
    let p = program(Some(rules), Some("?- big(F)."));
    let g = goals(&p);
    assert_eq!(g.len(), 2);
    assert_eq!(
        g[0],
        GoalHead {
            clause: 8,
            kind: "assert",
            name: Some("layered".into()),
            columns: names(&["A", "B", "_"]),
        }
    );
    assert_eq!(
        (g[1].clause, g[1].kind, g[1].columns.clone()),
        (10, "query", names(&["F"]))
    );
    assert!(goals(&program(None, None)).is_empty());
}

#[test]
fn a_variable_bound_twice_is_one_column_and_a_comparison_binds_none() {
    let p = program(
        None,
        Some("?- file(F), N > 1, lines(F, N), file(F), G = F."),
    );
    assert_eq!(goals(&p)[0].columns, names(&["F", "N", "G"]));
}
