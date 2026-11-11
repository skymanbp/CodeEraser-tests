//! K1's constant-like tables (BARS.md §2 rule 5) per language over the
//! product's grammars, with the expression forms addenda 2 and 3 rule:
//! read-through wrappers (8), dotted names (10), R's other literals
//! (11), the template string (12), the regular-expression literal (7)
//! and the forms ruling 15 names active. One line per (table, role):
//!   literal / string / number / name / function: constant-like kinds
//!   subst: a child that makes a string a template with a substitution
//!   template: a template-string kind (ruling 12 when no substitution)
//!   dotted / container / pair: constant-like when every part is
//!   key: a pair's bare-word key or an object's method (never evaluated)
//!   unary / binary: operator kinds; active: kinds known active
//!   through: ruling 8; r-dotted: ruling 10; r-literal: ruling 11;
//!   regex: ruling 7; closed: ruling 15

use super::k1::{Cx, Judged, constant};
use std::collections::BTreeMap;
use std::sync::OnceLock;
use tree_sitter::Node;

const TABLES: &str = "
py literal integer float true false none
py string string
py subst interpolation
py number integer float
py name identifier
py dotted attribute
py function lambda
py container list tuple set dictionary concatenated_string
py pair pair
py unary unary_operator not_operator
py binary binary_operator boolean_operator comparison_operator
py active call await
py through parenthesized_expression
py closed subscript list_comprehension dictionary_comprehension set_comprehension generator_expression conditional_expression list_splat dictionary_splat
ts literal number true false null undefined
ts string string template_string
ts subst template_substitution
ts template template_string
ts number number
ts name identifier this
ts dotted member_expression
ts function arrow_function function_expression generator_function class
ts container array object
ts pair pair
ts key property_identifier shorthand_property_identifier method_definition
ts unary unary_expression
ts binary binary_expression
ts active call_expression new_expression await_expression
ts through parenthesized_expression as_expression satisfies_expression type_assertion non_null_expression
ts regex regex
ts closed subscript_expression spread_element ternary_expression jsx_element jsx_self_closing_element jsx_fragment
lua literal number true false nil
lua string string
lua number number
lua name identifier
lua dotted dot_index_expression
lua function function_definition
lua container table_constructor
lua pair field
lua unary unary_expression
lua binary binary_expression
lua active function_call
lua through parenthesized_expression
lua closed bracket_index_expression
r literal integer float complex true false null
r string string
r number integer float complex
r name identifier
r function function_definition
r unary unary_operator
r binary binary_operator
r active call
r through parenthesized_expression
r r-literal na inf nan
r r-dotted extract_operator namespace_operator
r closed subset subset2 if_statement
c literal number_literal char_literal true false null nullptr
c string string_literal concatenated_string raw_string_literal
c number number_literal
c name identifier qualified_identifier
c dotted field_expression
c function lambda_expression
c container initializer_list
c pair initializer_pair
c key field_designator subscript_designator field_identifier
c unary unary_expression
c binary binary_expression
c active call_expression new_expression
c through parenthesized_expression
c closed subscript_expression conditional_expression
";

/// A language's constant-like table: role → kinds.
pub struct Expr {
    roles: BTreeMap<&'static str, Vec<&'static str>>,
}

impl Expr {
    /// Whether `kind` plays `role` in this language.
    pub fn is(&self, role: &str, kind: &str) -> bool {
        self.roles.get(role).is_some_and(|ks| ks.contains(&kind))
    }

    /// The role a kind plays, first in the evaluator's order.
    pub fn role_of(&self, kind: &str) -> Option<&'static str> {
        ROLE_ORDER.iter().copied().find(|r| self.is(r, kind))
    }
}

/// The order a kind's role is looked up in (a template string is a
/// string first; the string rule reads the template row itself).
pub const ROLE_ORDER: [&str; 17] = [
    "string",
    "literal",
    "name",
    "function",
    "key",
    "dotted",
    "container",
    "pair",
    "through",
    "regex",
    "closed",
    "r-literal",
    "r-dotted",
    "unary",
    "binary",
    "active",
    "number",
];

/// The table of a language family (`py`, `ts`, `lua`, `r`, `c`).
pub fn expr(family: &str) -> &'static Expr {
    static ALL: OnceLock<BTreeMap<&'static str, Expr>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        let mut out: BTreeMap<&'static str, Expr> = BTreeMap::new();
        for words in TABLES
            .lines()
            .map(|l| l.split_whitespace().collect::<Vec<_>>())
        {
            if let [fam, role, kinds @ ..] = words.as_slice() {
                let e = out.entry(fam).or_insert_with(|| Expr {
                    roles: BTreeMap::new(),
                });
                e.roles.insert(role, kinds.to_vec());
            }
        }
        out
    });
    &all[family]
}

/// Ruling 10 in R: `a$b`, `a@b` (lhs a name or such a chain, rhs a
/// name) and `pkg::name`, `pkg:::name` (both sides names).
pub fn r_dotted(n: Node) -> bool {
    let side = |f: &str| n.child_by_field_name(f);
    let name = |c: Option<Node>| c.is_some_and(|c| c.kind() == "identifier");
    match n.kind() {
        "namespace_operator" => name(side("lhs")) && name(side("rhs")),
        "extract_operator" => {
            let lhs = side("lhs");
            let chain = lhs.is_some_and(|l| l.kind() == "extract_operator" && r_dotted(l));
            (name(lhs) || chain) && name(side("rhs"))
        }
        _ => false,
    }
}

/// Ruling 15's bare name as a statement, and any other bare constant
/// (no rule places it: undecided).
pub fn bare(cx: &mut Cx, e: &Expr, n: Node) -> Judged {
    if e.is("name", n.kind()) {
        return cx.active(15);
    }
    match constant(cx, e, n) {
        Judged::Inert => cx.undecided(n, &format!("bare {}", n.kind())),
        active => active,
    }
}
