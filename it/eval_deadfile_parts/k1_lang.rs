//! K1's statement rules per language (BARS.md §2 rules 1–6, with the
//! rulings of BARS-ADDENDUM-2.md): which top-level kinds are inert by
//! rules 1–3, which are control statements or calls (active), the
//! binding forms of rule 5, the export and Lua `return` forms of rule 6,
//! and the ruled statement forms — transparent conditionals and blocks
//! (1, 2), constructs that execute nothing (1, 3, 6), the hashbang (4),
//! bindings with no value (5, 13), the augmented assignment (14).

use super::k1::{Cx, Judged};
use super::k1_bind::{c, lua, python, r, typescript};
use codeeraser::scan::lang::Lang;
use tree_sitter::Node;

/// A statement's standing: judged, or transparent (ruling 1 / 2) with
/// the statements inside to judge as if at the top level.
pub enum Stmt<'t> {
    Judged(Judged),
    Inside(Vec<Node<'t>>),
}

/// (inert by rules 1–3, active: control statements and calls)
type Table = (&'static [&'static str], &'static [&'static str]);

const PY_STMT: Table = (
    &[
        "import_statement",
        "import_from_statement",
        "future_import_statement",
        "function_definition",
        "class_definition",
        "decorated_definition",
        "type_alias_statement",
    ],
    &[
        "if_statement",
        "for_statement",
        "while_statement",
        "try_statement",
        "with_statement",
        "match_statement",
    ],
);
const TS_STMT: Table = (
    &[
        "import_statement",
        "function_declaration",
        "generator_function_declaration",
        "class_declaration",
        "abstract_class_declaration",
        "interface_declaration",
        "type_alias_declaration",
        "enum_declaration",
        "ambient_declaration",
        "module",
        "internal_module",
    ],
    &[
        "if_statement",
        "for_statement",
        "for_in_statement",
        "while_statement",
        "do_statement",
        "try_statement",
        "switch_statement",
        "throw_statement",
        "labeled_statement",
    ],
);
const LUA_STMT: Table = (
    &["function_declaration"],
    &[
        "function_call",
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
        "goto_statement",
    ],
);
const R_STMT: Table = (
    &[],
    &[
        "call",
        "if_statement",
        "for_statement",
        "while_statement",
        "repeat_statement",
    ],
);
const C_STMT: Table = (
    &[
        "preproc_include",
        "preproc_def",
        "preproc_function_def",
        "function_definition",
        "type_definition",
        "struct_specifier",
        "union_specifier",
        "enum_specifier",
        "class_specifier",
        "namespace_definition",
        "template_declaration",
        "alias_declaration",
        "concept_definition",
    ],
    &["expression_statement"],
);
const JAVA_STMT: Table = (
    &[
        "package_declaration",
        "import_declaration",
        "module_declaration",
        "class_declaration",
        "interface_declaration",
        "enum_declaration",
        "record_declaration",
        "annotation_type_declaration",
    ],
    &[],
);

/// A language outside the seven: no kind is inert or active by table.
const NO_STMT: Table = (&[], &[]);

/// Ruling 1: the C / C++ conditionals and linkage blocks (transparent)
/// and the other directives (execute nothing).
const C_CONDITIONAL: [&str; 6] = [
    "preproc_if",
    "preproc_ifdef",
    "preproc_elif",
    "preproc_elifdef",
    "preproc_else",
    "linkage_specification",
];
const C_DIRECTIVE: [&str; 2] = ["preproc_call", "preproc_error"];
/// (ruling 2) bare blocks; (ruling 3) the empty statement; (ruling 4)
/// the hashbang; (ruling 6) the Lua label — in every language that has
/// the kind.
const BLOCK: [&str; 3] = ["do_statement", "statement_block", "compound_statement"];

/// One top-level statement under K1.
pub fn judge_statement<'t>(cx: &mut Cx, lang: Lang, n: Node<'t>) -> Stmt<'t> {
    let k = n.kind();
    if k.contains("comment") {
        return Stmt::Judged(Judged::Inert);
    }
    if k == "ERROR" {
        // ruling 16 (rule W): a fragment the parser could not place is
        // no witness under K1; K1-strict reads it active
        cx.fragments.push(super::k1::line(n));
        return Stmt::Judged(cx.rule(16, |_| Judged::Inert));
    }
    if let Some(s) = ruled_statement(cx, lang, n) {
        return s;
    }
    let (table, rest): (Table, fn(&mut Cx, Node) -> Judged) = match lang.name() {
        "python" => (PY_STMT, python),
        "typescript" | "tsx" => (TS_STMT, typescript),
        "lua" => (LUA_STMT, lua),
        "r" => (R_STMT, r),
        "c" | "cpp" => (C_STMT, c),
        "java" => (JAVA_STMT, |cx, n| cx.undecided(n, n.kind())),
        _ => (NO_STMT, |cx, n| cx.undecided(n, "language")),
    };
    Stmt::Judged(if table.0.contains(&k) {
        Judged::Inert
    } else if table.1.contains(&k) {
        Judged::Active
    } else {
        rest(cx, n)
    })
}

/// The statement forms rulings 1–4 and 6 place (Lua's `do` is a bare
/// block only in Lua: TypeScript's `do … while` is a loop).
fn ruled_statement<'t>(cx: &mut Cx, lang: Lang, n: Node<'t>) -> Option<Stmt<'t>> {
    let k = n.kind();
    let c_family = matches!(lang.name(), "c" | "cpp");
    let block = BLOCK.contains(&k) && (k != "do_statement" || lang.name() == "lua");
    if (c_family && C_CONDITIONAL.contains(&k)) || block {
        let n_rule = if block { 2 } else { 1 };
        let inner = inside(n);
        return Some(match cx.rule(n_rule, |_| Judged::Inert) {
            Judged::Inert => Stmt::Inside(inner),
            Judged::Active => Stmt::Judged(Judged::Active),
        });
    }
    let nothing = match k {
        _ if c_family && C_DIRECTIVE.contains(&k) => 1,
        "empty_statement" => 3,
        "hash_bang_line" => 4,
        "label_statement" => 6,
        _ => return None,
    };
    Some(Stmt::Judged(cx.rule(nothing, |_| Judged::Inert)))
}

/// The statements a transparent construct holds: its named children
/// past the selecting condition / macro name / linkage string, a nested
/// `#else` / `#elif` or body list opened in turn.
fn inside(n: Node) -> Vec<Node> {
    let skip = ["condition", "name", "value"];
    let mut out = Vec::new();
    for i in 0..n.named_child_count() {
        let Some(c) = n.named_child(i as u32) else {
            continue;
        };
        let field = n.field_name_for_named_child(i as u32);
        if field.is_some_and(|f| skip.contains(&f)) {
            continue;
        }
        if matches!(c.kind(), "declaration_list" | "block") {
            out.extend(inside(c));
        } else {
            out.push(c);
        }
    }
    out
}
