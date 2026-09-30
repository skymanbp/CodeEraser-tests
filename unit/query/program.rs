use super::*;
use crate::query::PRELUDE;

fn query(text: &str) -> Program {
    Program::lex(PRELUDE, None, Some(text)).unwrap()
}

/// The query's tokens as `kind:value` words, one line per table.
fn query_tokens(p: &Program) -> String {
    p.tokens
        .iter()
        .filter(|t| t.src == Source::Query)
        .map(|t| format!("{}:{}", t.kind, t.value))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_prelude_is_eight_clauses_over_six_reserved_predicates() {
    let p = Program::lex(PRELUDE, None, None).unwrap();
    assert_eq!((p.clauses, p.prelude_clauses, p.vars.len()), (8, 8, 8));
    assert_eq!(
        p.preds,
        ["entry", "reach", "dead", "depends", "same_dir", "dir_ref"]
    );
    assert!(p.tokens.iter().all(|t| t.src == Source::Prelude));
    // the schema tables a bare `dead(F)` needs: file, in_dir, role, ref
    let referenced: Vec<u32> = p.referenced().into_iter().collect();
    assert_eq!(referenced, [1, 2, 7, 9]);
    // the two liveness-inert reference kinds are name constants
    for word in ["asset", "refdef"] {
        assert_eq!(
            p.names.get(&legend::sym(word)).map(String::as_str),
            Some(word)
        );
    }
    assert_eq!(p.locate(0), "prelude 8:1");
    assert_eq!(p.locate(p.tokens.len()), "end of program");
}

#[test]
fn variables_number_per_clause_by_first_appearance_and_every_underscore_is_fresh() {
    let p = query("?- ref(N, M, _, _), reach(M), lines(N, _x).");
    assert_eq!(
        query_tokens(&p),
        "16:0 0:9 13:0 1:0 11:0 1:1 11:0 6:2 11:0 6:3 14:0 11:0 0:1001 13:0 1:1 14:0 11:0 0:8 13:0 1:0 11:0 6:4 14:0 12:0"
    );
    assert_eq!(p.vars[8], ["N", "M", "_", "_", "_"]);
    assert_eq!(
        (p.var_name(8, 1), p.var_name(8, 9)),
        ("M".to_string(), "V9".to_string())
    );
    assert_eq!(p.clauses, 9);
}

#[test]
fn a_lowercase_word_is_a_predicate_before_a_paren_and_a_name_elsewhere() {
    let rules = "big(F) :- lines(F, N), N > 300.\n";
    let p = Program::lex(
        PRELUDE,
        Some(rules),
        Some("?- big(F), node(F, file), file(F), lang(F, rust)."),
    )
    .unwrap();
    assert_eq!(p.preds.last().map(String::as_str), Some("big"));
    let q: Vec<&Token> = p.tokens.iter().filter(|t| t.src == Source::Query).collect();
    assert_eq!(
        (q[1].kind, q[1].value, q[1].spell.as_str()),
        (0, 1006, "big")
    );
    // `file` twice: the name constant, then the predicate
    let files: Vec<(i64, i128)> = q
        .iter()
        .filter(|t| t.spell == "file")
        .map(|t| (t.kind, t.value))
        .collect();
    assert_eq!(files, [(4, i128::from(legend::sym("file"))), (0, 1)]);
    assert_eq!(
        p.names.get(&legend::sym("rust")).map(String::as_str),
        Some("rust")
    );
    assert_eq!(
        (p.pred_name(1006), p.pred_name(9), p.pred_name(1999)),
        ("big".into(), "ref".into(), "pred#1999".into())
    );
    assert_eq!((p.clauses, p.prelude_clauses), (10, 8));
    // the three sources ride in wire order
    let order: Vec<usize> = p.tokens.iter().map(|t| t.src as usize).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn a_string_is_a_set_only_in_the_set_position_and_in_is_sugar_for_it() {
    let p = query("?- set(\"a/**\", F), in(G, \"b/**\"), mention(\"Foo\", F).");
    assert_eq!(p.sets, ["a/**", "b/**"]);
    let q: Vec<(i64, i128, &str)> = p
        .tokens
        .iter()
        .filter(|t| t.src == Source::Query)
        .map(|t| (t.kind, t.value, t.spell.as_str()))
        .collect();
    let foo = i128::from(legend::sym("Foo"));
    assert_eq!(
        q,
        [
            (16, 0, "?-"),
            (0, 26, "set"),
            (13, 0, "("),
            (3, 0, "a/**"),
            (11, 0, ","),
            (1, 0, "F"),
            (14, 0, ")"),
            (11, 0, ","),
            (0, 26, "in"),
            (13, 0, "("),
            (3, 1, "b/**"),
            (11, 0, ","),
            (1, 1, "G"),
            (14, 0, ")"),
            (11, 0, ","),
            (0, 24, "mention"),
            (13, 0, "("),
            (4, foo, "Foo"),
            (11, 0, ","),
            (1, 0, "F"),
            (14, 0, ")"),
            (12, 0, "."),
        ]
    );
    assert_eq!(
        p.names.get(&legend::sym("Foo")).map(String::as_str),
        Some("Foo")
    );
}

#[test]
fn a_minus_is_a_sign_only_where_no_operand_precedes_it() {
    let p = query("?- lines(F, N), M = N - -3, M > -300, K = (N) - 1.");
    let ops: Vec<(i64, i128)> = p
        .tokens
        .iter()
        .filter(|t| t.src == Source::Query && (t.kind == 2 || t.spell == "-"))
        .map(|t| (t.kind, t.value))
        .collect();
    assert_eq!(ops, [(25, 0), (2, -3), (2, -300), (25, 0), (2, 1)]);
    assert_eq!(num(-3), serde_json::json!(-3));
    assert_eq!(num(5), serde_json::json!(5u64));
    assert_eq!(num(i128::from(u64::MAX)), serde_json::Value::from(u64::MAX));
}

#[test]
fn a_lexical_fault_is_pinned_where_the_item_began_and_stops_the_program() {
    let at = |q: &str| Program::lex(PRELUDE, None, Some(q)).unwrap_err();
    let f = at("?- mention(\"x, F).");
    assert_eq!(
        (f.src, f.line, f.col, f.what.as_str()),
        (Source::Query, 1, 12, "unterminated string")
    );
    assert_eq!(f.describe(), "query 1:12: unterminated string");
    let f = at("?- lines(F,\n  99999999999999999999).");
    assert_eq!(
        (f.line, f.col, f.what.as_str()),
        (2, 3, "integer out of range")
    );
    assert_eq!(
        at("?- lines(F, -9223372036854775809).").what,
        "integer out of range"
    );
    let floor = query("?- lines(F, -9223372036854775808).");
    assert_eq!(
        floor.tokens.iter().find(|t| t.kind == 2).map(|t| t.value),
        Some(i128::from(i64::MIN))
    );
    let f = at("?- file(F)).");
    assert_eq!((f.line, f.col, f.what.as_str()), (1, 11, "unbalanced `)`"));
    let f = at("?- file(F) & x.");
    assert_eq!(
        (f.line, f.col, f.what.as_str()),
        (1, 12, "unexpected character")
    );
    let f = at("?- in(F, G).");
    assert_eq!((f.line, f.col), (1, 10));
    assert!(f.what.starts_with("in(File, \"glob\")"), "{}", f.what);
    // a fault in the rules file names that source; the query is never reached
    let f = Program::lex(PRELUDE, Some("big(F) :- lines(F, \"x).\n"), Some("?- @.")).unwrap_err();
    assert_eq!((f.src, f.line, f.col), (Source::Rules, 1, 20));
}

#[test]
fn keywords_are_never_predicates_and_an_unfinished_clause_still_seats_its_variables() {
    let p = query("?- N = count(F : file(F)), not dead(F), N >= 2");
    assert_eq!(
        query_tokens(&p),
        "16:0 1:0 18:0 29:0 13:0 1:1 33:0 0:1 13:0 1:1 14:0 14:0 11:0 15:0 0:1002 13:0 1:1 14:0 11:0 1:0 23:0 2:2"
    );
    assert_eq!(
        (p.clauses, p.vars[8].clone()),
        (9, vec!["N".to_string(), "F".to_string()])
    );
    assert_eq!(p.wire().len(), p.tokens.len());
    assert_eq!(p.wire()[p.tokens.len() - 1], serde_json::json!([2, 2]));
}
