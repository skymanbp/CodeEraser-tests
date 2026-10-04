//! The answer shapes a judged row carries and the batched ladder call
//! that produces them (moved out of score.rs, plan v2.33 wave W2a):
//! the four answer fields, the three shapes a row may hold, and one
//! resolve_all call per batch of sites.

use crate::common::reason_name;
use crate::eval_support::lang_of;
use codeeraser::graph::ladder::{self, Outcome, Scope, Site};
use codeeraser::graph::sites::RawSite;
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};

/// An outcome as the four fields every judged row and gap site carries:
/// the in-corpus answer (a package at the tree root is "."; a section
/// is `path#slug`, its slugless degrade the file), whether External
/// answered, the rung, and the refusal reason.
pub(super) fn answer(out: &Outcome) -> Value {
    let (answered, external, rung, reason) = match out {
        Outcome::Resolved { path, rung }
        | Outcome::ResolvedVia { path, rung }
        | Outcome::ResolvedInert { path, rung } => (Some(path.clone()), false, Some(*rung), None),
        Outcome::ResolvedPackage { dir, rung } => {
            let dir = if dir.is_empty() { "." } else { dir };
            (Some(dir.to_string()), false, Some(*rung), None)
        }
        Outcome::ResolvedSection { path, slug, rung } => {
            let at = slug
                .as_ref()
                .map_or_else(|| path.clone(), |s| format!("{path}#{s}"));
            (Some(at), false, Some(*rung), None)
        }
        Outcome::External { rung } => (None, true, Some(*rung), None),
        Outcome::Unresolved(r) => (None, false, None, Some(reason_name(*r))),
    };
    json!({"answered": answered, "external": external, "rung": rung, "reason": reason})
}

/// Which of the three answer shapes a row or gap site holds: an
/// in-corpus answer with its rung, External with its rung, a refusal
/// with its reason — anything else is a cooked row.
pub(super) fn shape_of(a: &Value) -> Option<&'static str> {
    match (&a["answered"], &a["external"], &a["rung"], &a["reason"]) {
        (Value::String(_), Value::Bool(false), Value::Number(_), Value::Null) => Some("answered"),
        (Value::Null, Value::Bool(true), Value::Number(_), Value::Null) => Some("external"),
        (Value::Null, Value::Bool(false), Value::Null, Value::String(_)) => Some("refused"),
        _ => None,
    }
}

/// The ladder's answers for sites, in one batch (the core answers four
/// languages' sites in one resolve/1 request since plan v2.33 W2a).
pub(super) fn answers(sites: &[(Lang, Site)], scope: &Scope) -> Vec<Value> {
    let batch: Vec<(Lang, &Site)> = sites.iter().map(|(l, s)| (*l, s)).collect();
    ladder::resolve_all(&batch, scope)
        .expect("the ladder answers")
        .iter()
        .map(answer)
        .collect()
}

/// Detected sites of frozen files as the ladder reads them.
pub(super) fn found<'a>(sites: &[(&'a str, &'a RawSite)], lang: &str) -> Vec<(Lang, Site<'a>)> {
    sites
        .iter()
        .map(|(path, s)| {
            let at = Site {
                kind: s.kind,
                from: path,
                spec: &s.spec,
                line: s.line,
            };
            (lang_of(lang), at)
        })
        .collect()
}
