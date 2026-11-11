//! The classifiers of the dead-file exam's bar (i): K1 (BARS.md §2 with
//! the main session's rulings of BARS-ADDENDUM-2.md), K1-strict (§2
//! read with no ruling: every construct a ruling places reads active /
//! not constant-like), and K0, the lane's first classifier
//! (`universe_counts.py`, sha256 549337f8…), printed beside them and
//! deciding nothing.
//!
//! K1 reads a file's top-level statements through the product's parser
//! (`scan::ast::parse_lang`). A statement is INERT when it is a comment
//! (rule 1), an import / include / package / module header (2), a
//! declaration (3), a bare string literal (4), a binding whose every
//! right-hand side is constant-like (5), or an export around 1–5 / an
//! export list / a top-level Lua `return` of a constant-like value (6);
//! every other statement is ACTIVE. Where a ruling places a construct
//! the row records the ruling's number; a construct no rule and no
//! ruling places is UNDECIDED — the generator refuses a universe holding
//! one and the question goes to the main session. Class: `document` (a
//! doc class), else `declares` (every statement inert; an empty file
//! too), else `runs-alone`, witnessed by the first active statement's
//! kind and line.

use super::k1_lang::{Stmt, judge_statement};
use super::k1_tables::{Expr, r_dotted};
use codeeraser::scan::lang::Lang;
use std::collections::BTreeSet;
use tree_sitter::Node;

/// One construct's standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Judged {
    Inert,
    Active,
}

/// The walk's context: which reading, and what the walk met.
pub struct Cx<'a> {
    pub strict: bool,
    pub src: &'a [u8],
    pub rulings: BTreeSet<u8>,
    pub undecided: Vec<(String, usize)>,
    /// lines of the top-level fragments the parser could not place (16)
    pub fragments: Vec<usize>,
}

impl Cx<'_> {
    /// A construct ruling `n` places: the ruled reading, or active
    /// under K1-strict. The number is recorded either way.
    pub fn rule(&mut self, n: u8, ruled: impl FnOnce(&mut Self) -> Judged) -> Judged {
        self.rulings.insert(n);
        if self.strict {
            Judged::Active
        } else {
            ruled(self)
        }
    }

    /// A ruling the lane's draft placed before the question (11, 12,
    /// 13; addendum 3): the same reading under both classes.
    pub fn confirmed(&mut self, n: u8, ruled: Judged) -> Judged {
        self.rulings.insert(n);
        ruled
    }

    /// A ruling that is active under both readings (7, 9, 14, 15).
    pub fn active(&mut self, n: u8) -> Judged {
        self.rulings.insert(n);
        Judged::Active
    }

    /// A construct no rule and no ruling places: named, and read active
    /// so the walk goes on to name every other one.
    pub fn undecided(&mut self, n: Node, what: &str) -> Judged {
        self.undecided.push((what.to_string(), line(n)));
        Judged::Active
    }

    pub fn text(&self, n: Node) -> &str {
        n.utf8_text(self.src).unwrap_or("")
    }
}

/// One reading of a file: class and witness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub class: &'static str,
    pub witness: Option<(String, usize)>,
}

/// A file under both readings, with the rulings and undecided
/// constructs its K1 walk met.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classes {
    pub k1: Reading,
    pub strict: Reading,
    pub rulings: Vec<u8>,
    pub undecided: Vec<(String, usize)>,
    /// lines of the fragments ruling 16 met
    pub fragments: Vec<usize>,
}

/// 1-based line of a node.
pub fn line(n: Node) -> usize {
    n.start_position().row + 1
}

/// The named children of a node, comments left out (rule 1 inside
/// containers too).
pub fn kids(n: Node) -> Vec<Node> {
    (0..n.named_child_count())
        .filter_map(|i| n.named_child(i as u32))
        .filter(|c| !c.kind().contains("comment"))
        .collect()
}

/// Both readings of one file: `doc` is its document class (features.rs (e)).
pub fn classify(text: &str, lang: Option<Lang>, doc: bool) -> Classes {
    let (k1, cx) = reading(text, lang, doc, false);
    let (strict, _) = reading(text, lang, doc, true);
    let rulings = cx.rulings.into_iter().collect();
    Classes {
        k1,
        strict,
        rulings,
        undecided: cx.undecided,
        fragments: cx.fragments,
    }
}

fn reading(text: &str, lang: Option<Lang>, doc: bool, strict: bool) -> (Reading, Cx<'_>) {
    let mut cx = Cx {
        strict,
        src: text.as_bytes(),
        rulings: BTreeSet::new(),
        undecided: Vec::new(),
        fragments: Vec::new(),
    };
    let none = |class| Reading {
        class,
        witness: None,
    };
    if doc {
        return (none("document"), cx);
    }
    let tree = lang.and_then(|l| codeeraser::scan::ast::parse_lang(text, l).map(|t| (l, t)));
    let Some((lang, tree)) = tree else {
        cx.undecided.push(("no grammar".into(), 0));
        return (none("declares"), cx);
    };
    let witness = statements(&mut cx, lang, kids_all(tree.root_node()));
    let class = if witness.is_some() {
        "runs-alone"
    } else {
        "declares"
    };
    (Reading { class, witness }, cx)
}

/// Every named child (comments too: rule 1 judges them inert).
fn kids_all(n: Node) -> Vec<Node> {
    (0..n.named_child_count())
        .filter_map(|i| n.named_child(i as u32))
        .collect()
}

/// Judge a run of statements (the top level, or the inside of a
/// transparent construct); the first active one is the witness. Every
/// statement is judged, so every ruling met is recorded.
pub fn statements<'t>(cx: &mut Cx, lang: Lang, nodes: Vec<Node<'t>>) -> Option<(String, usize)> {
    let mut witness = None;
    for n in nodes {
        let got = match judge_statement(cx, lang, n) {
            Stmt::Judged(Judged::Inert) => None,
            Stmt::Judged(Judged::Active) => Some((n.kind().to_string(), line(n))),
            Stmt::Inside(inner) => statements(cx, lang, inner),
        };
        if witness.is_none() {
            witness = got;
        }
    }
    witness
}

/// Whether an expression is constant-like under the language's table
/// (rule 5, with rulings 7–12 and 15 where the table names them).
pub fn constant(cx: &mut Cx, e: &Expr, n: Node) -> Judged {
    let k = n.kind();
    match e.role_of(k) {
        Some("string") => string(cx, e, n),
        Some("literal" | "name" | "function" | "key") => Judged::Inert,
        Some("dotted" | "container" | "pair") => all(cx, e, kids(n)),
        // ruling 8: read through, judge what is inside
        Some("through") => cx.rule(8, |cx| all(cx, e, kids(n).into_iter().take(1).collect())),
        Some("regex") => cx.active(7),
        Some("closed") => cx.active(15),
        Some("r-literal") => cx.confirmed(11, Judged::Inert),
        Some("r-dotted") if r_dotted(n) => cx.rule(10, |_| Judged::Inert),
        Some("unary") => unary(cx, e, n),
        Some("binary") => operator(cx, e, kids(n)),
        // ruling 16's last sentence: an ERROR inside a right-hand side is
        // not on rule 5's list, so the binding is active under both
        Some("r-dotted" | "active") => Judged::Active,
        _ if k == "ERROR" => Judged::Active,
        _ => cx.undecided(n, k),
    }
}

/// Every one of the nodes constant-like (each is judged, so every
/// ruling met is recorded).
pub fn all(cx: &mut Cx, e: &Expr, ns: Vec<Node>) -> Judged {
    let mut out = Judged::Inert;
    for c in ns {
        if constant(cx, e, c) == Judged::Active {
            out = Judged::Active;
        }
    }
    out
}

/// A string: active with a substitution; a template string with none
/// is a string literal by ruling 12.
fn string(cx: &mut Cx, e: &Expr, n: Node) -> Judged {
    if kids(n).iter().any(|c| e.is("subst", c.kind())) {
        return Judged::Active;
    }
    if e.is("template", n.kind()) {
        return cx.confirmed(12, Judged::Inert);
    }
    Judged::Inert
}

/// The minus of a number is constant-like; any other operator is not
/// (ruling 9 names the operator over literals only).
fn unary(cx: &mut Cx, e: &Expr, n: Node) -> Judged {
    let op = n.child(0).map(|c| c.kind()).unwrap_or("");
    let operand = kids(n);
    if op == "-" && operand.len() == 1 && e.is("number", operand[0].kind()) {
        return Judged::Inert;
    }
    operator(cx, e, operand)
}

fn operator(cx: &mut Cx, e: &Expr, operands: Vec<Node>) -> Judged {
    let literal = |c: &Node| e.is("literal", c.kind()) || e.is("string", c.kind());
    if operands.iter().all(literal) {
        return cx.active(9);
    }
    Judged::Active
}

/// K0 (`universe_counts.py` DECLARES, ported): `document` for a doc
/// class; `declares` when the language has a set and every distinct
/// top-level kind (at least one) is in it; else `runs-alone`.
pub fn k0(lang: &str, kinds: &[String], doc: bool) -> &'static str {
    if doc {
        return "document";
    }
    let set = super::k0_sets::set(lang);
    let declares =
        set.is_some_and(|s| !kinds.is_empty() && kinds.iter().all(|k| s.contains(&k.as_str())));
    if declares { "declares" } else { "runs-alone" }
}
