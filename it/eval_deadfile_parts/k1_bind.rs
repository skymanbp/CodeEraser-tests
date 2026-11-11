//! K1's binding, export and return forms per language (BARS.md §2
//! rules 5 and 6, with rulings 5, 13 and 14 of addenda 2 and 3): the
//! per-language half of k1_lang.rs's statement judge.

use super::k1::{Cx, Judged, all, constant, kids};
use super::k1_lang::{Stmt, judge_statement};
use super::k1_tables::{Expr, bare, expr};
use codeeraser::scan::lang::Lang;
use tree_sitter::Node;

/// A field's value under the table, or ruling 13 when the binding has
/// no right-hand side.
fn value(cx: &mut Cx, e: &Expr, n: Node, field: &str) -> Judged {
    match n.child_by_field_name(field) {
        Some(v) => constant(cx, e, v),
        None => cx.confirmed(13, Judged::Inert),
    }
}

pub fn python(cx: &mut Cx, n: Node) -> Judged {
    if n.kind() != "expression_statement" {
        return cx.undecided(n, n.kind());
    }
    let mut out = Judged::Inert;
    for p in kids(n) {
        let j = match p.kind() {
            "string" | "concatenated_string" => constant(cx, expr("py"), p),
            "assignment" => py_binding(cx, p),
            "augmented_assignment" => cx.active(14),
            _ => bare(cx, expr("py"), p),
        };
        if j == Judged::Active {
            out = Judged::Active;
        }
    }
    out
}

fn py_binding(cx: &mut Cx, p: Node) -> Judged {
    match p.child_by_field_name("right") {
        Some(r) if r.kind() == "assignment" => py_binding(cx, r),
        Some(r) if r.kind() == "expression_list" => all(cx, expr("py"), kids(r)),
        Some(r) => constant(cx, expr("py"), r),
        None => cx.confirmed(13, Judged::Inert),
    }
}

pub fn typescript(cx: &mut Cx, n: Node) -> Judged {
    match n.kind() {
        "lexical_declaration" | "variable_declaration" => {
            let ds = kids(n)
                .into_iter()
                .filter(|d| d.kind() == "variable_declarator");
            ds.fold(Judged::Inert, |acc, d| {
                worst(acc, value(cx, expr("ts"), d, "value"))
            })
        }
        "expression_statement" => kids(n)
            .into_iter()
            .fold(Judged::Inert, |acc, p| worst(acc, ts_expression(cx, p))),
        "export_statement" => ts_export(cx, n),
        _ => cx.undecided(n, n.kind()),
    }
}

fn worst(a: Judged, b: Judged) -> Judged {
    if a == Judged::Active { a } else { b }
}

fn ts_expression(cx: &mut Cx, p: Node) -> Judged {
    match p.kind() {
        "string" => Judged::Inert,
        "internal_module" => Judged::Inert,
        "assignment_expression" => value(cx, expr("ts"), p, "right"),
        _ => bare(cx, expr("ts"), p),
    }
}

fn ts_export(cx: &mut Cx, n: Node) -> Judged {
    if let Some(d) = n.child_by_field_name("declaration") {
        return match judge_statement(cx, Lang::TypeScript, d) {
            Stmt::Judged(j) => j,
            Stmt::Inside(_) => cx.undecided(d, d.kind()),
        };
    }
    if let Some(v) = n.child_by_field_name("value") {
        return constant(cx, expr("ts"), v);
    }
    let rest: Vec<Node> = kids(n)
        .into_iter()
        .filter(|c| !matches!(c.kind(), "export_clause" | "string" | "namespace_export"))
        .collect();
    all(cx, expr("ts"), rest)
}

pub fn lua(cx: &mut Cx, n: Node) -> Judged {
    match n.kind() {
        "variable_declaration" => {
            let global = cx.text(n).starts_with("global");
            kids(n).into_iter().fold(Judged::Inert, |acc, c| {
                worst(acc, lua_assignment(cx, c, global))
            })
        }
        "implicit_variable_declaration" => cx.rule(5, |_| Judged::Inert),
        "assignment_statement" => lua_assignment(cx, n, false),
        "return_statement" if kids(n).is_empty() => cx.undecided(n, "empty return"),
        "return_statement" => kids(n).into_iter().fold(Judged::Inert, |acc, l| {
            worst(acc, all(cx, expr("lua"), kids(l)))
        }),
        _ => cx.undecided(n, n.kind()),
    }
}

/// A Lua binding: a name list alone binds no value (ruling 5 for a
/// `global`, 13 for a `local`); a list with values is rule 5.
fn lua_assignment(cx: &mut Cx, c: Node, global: bool) -> Judged {
    match c.kind() {
        "variable_list" if global => cx.rule(5, |_| Judged::Inert),
        "variable_list" => cx.confirmed(13, Judged::Inert),
        "assignment_statement" => {
            let values: Vec<Node> = kids(c)
                .into_iter()
                .filter(|k| k.kind() == "expression_list")
                .collect();
            values.into_iter().fold(Judged::Inert, |acc, l| {
                worst(acc, all(cx, expr("lua"), kids(l)))
            })
        }
        _ => cx.undecided(c, c.kind()),
    }
}

const R_BIND: [&str; 3] = ["<-", "=", "<<-"];

pub fn r(cx: &mut Cx, n: Node) -> Judged {
    match n.kind() {
        "binary_operator" => {
            let op = n
                .child_by_field_name("operator")
                .map(|o| cx.text(o).to_string());
            if op.as_deref().is_some_and(|o| R_BIND.contains(&o)) {
                value(cx, expr("r"), n, "rhs")
            } else {
                constant(cx, expr("r"), n)
            }
        }
        "string" => Judged::Inert,
        _ => bare(cx, expr("r"), n),
    }
}

/// A C / C++ declaration: a function declarator declares (rule 3); an
/// initialised declarator is rule 5; a declarator with no value binds
/// none (ruling 13).
pub fn c(cx: &mut Cx, n: Node) -> Judged {
    if n.kind() != "declaration" {
        return cx.undecided(n, n.kind());
    }
    let declarators: Vec<Node> = n
        .children_by_field_name("declarator", &mut n.walk())
        .collect();
    declarators
        .into_iter()
        .map(|d| match d.kind() {
            "init_declarator" => value(cx, expr("c"), d, "value"),
            _ if declares_function(d) => Judged::Inert,
            _ => cx.confirmed(13, Judged::Inert),
        })
        .fold(Judged::Inert, worst)
}

fn declares_function(d: Node) -> bool {
    d.kind() == "function_declarator"
        || d.child_by_field_name("declarator")
            .is_some_and(declares_function)
}
