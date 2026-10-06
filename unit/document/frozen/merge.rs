//! Frozen W7 oracle (plan v2.33 W7; test-only): merge: cli/src/merge/face.rs, copied byte for
//! byte from 1324c927 (spans cli/src/merge/face.rs 202-203, 215-221, 223-243, 245-254, 256-270) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{Resolve, Why};
use crate::merge::groups;
use std::collections::BTreeMap;

#[path = "../spelled/merge.rs"]
mod spelled;

/// Every member file's text, read once.
type Texts = BTreeMap<String, String>;

/// The merge document's strings: each member's path, unit and text at
/// a hole (by its number across groups), and the reason.
struct Names<'a> {
    members: Vec<&'a groups::Member>,
    texts: Texts,
    why: Why,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let member = |k: &i128| usize::try_from(*k).ok().and_then(|k| self.members.get(k));
        match (class, ints) {
            ("why", _) => self.why.at(ints),
            ("path", [k]) => Some(member(k)?.path.clone()),
            ("unit", [k]) => member(k)?.unit.clone(),
            ("text", [k, post, post_end]) => {
                let m = member(k)?;
                let (post, post_end) = (i64::try_from(*post).ok()?, i64::try_from(*post_end).ok()?);
                Some(span_text(&self.texts[&m.path], &m.tree, post, post_end))
            }
            // a console line's text cut to the cap the core names
            ("clipped", [k, post, post_end, cap]) => {
                let text = self.resolve("text", &[*k, *post, *post_end])?;
                Some(clip(&text, usize::try_from(*cap).ok()?))
            }
            _ => None,
        }
    }
}

/// One line, at most `cap` characters, `…` when cut: the core's
/// `clipped` reference (its console names the cap).
fn clip(text: &str, cap: usize) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= cap {
        return flat;
    }
    let cut: String = flat.chars().take(cap).collect();
    format!("{cut}…")
}

/// The source text from node `post`'s start to node `post_end`'s end
/// (−1: nothing on this side).
fn span_text(
    text: &str,
    tree: &crate::dedup::t3::tree::UnitTree,
    post: i64,
    post_end: i64,
) -> String {
    let at = |p: i64| usize::try_from(p).ok().and_then(|i| tree.spans.get(i));
    at(post)
        .zip(at(post_end))
        .and_then(|(&(s, _), &(_, e))| text.get(s..e))
        .unwrap_or("")
        .to_string()
}
