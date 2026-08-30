//! The inline chip channel: `<!--ce:ID-->…<!--/ce-->` around a value
//! in prose or a code span. Markdown and HTML both hide the comment,
//! so a chip is invisible to a reader and portable across the README,
//! the contracts, the booklets and the site. A span is read as maximal
//! runs of the id's form characters (Form::token_char, in the
//! surface's language) and must hold exactly ONE such run — that rule
//! is what makes a bless safe: the run is replaced, the surrounding
//! text (backticks, a bold marker, a `v` prefix, a measure word) is
//! untouched. Not a chip carrier: a `<title>` or an attribute value,
//! where a comment is literal text — those sites are source-literal
//! assertions (facts_registry.rs).

use super::Form;
use std::ops::Range;

pub const OPEN: &str = "<!--ce:";
pub const CLOSE: &str = "<!--/ce-->";

pub struct Chip {
    pub id: String,
    /// The bytes between the two tags.
    pub span: Range<usize>,
}

/// Every chip in `text`, in order; an open tag that never closes or a
/// chip nested in another is a refusal naming the surface.
pub fn chips(text: &str, label: &str) -> Vec<Chip> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(rel) = text[at..].find(OPEN) {
        let id_start = at + rel + OPEN.len();
        let id_end = id_start
            + text[id_start..].find("-->").unwrap_or_else(|| {
                panic!("{label}: chip open tag at byte {id_start} never closes")
            });
        let id = &text[id_start..id_end];
        let body_start = id_end + "-->".len();
        let body_end = body_start
            + text[body_start..]
                .find(CLOSE)
                .unwrap_or_else(|| panic!("{label}: chip {id} has no {CLOSE}"));
        assert!(
            !text[body_start..body_end].contains(OPEN),
            "{label}: chip {id} nests another chip"
        );
        out.push(Chip {
            id: id.to_string(),
            span: body_start..body_end,
        });
        at = body_end + CLOSE.len();
    }
    out
}

/// Maximal runs of `form` characters in `s`, as byte ranges.
fn tokens(s: &str, form: Form, zh: bool) -> Vec<Range<usize>> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, c) in s.char_indices() {
        match (start, form.token_char(zh, c)) {
            (None, true) => start = Some(i),
            (Some(b), false) => {
                runs.push(b..i);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(b) = start {
        runs.push(b..s.len());
    }
    runs
}

/// `text` with every chip span rewritten to what `render` returns for
/// its id, plus one note per value that moved (`label: id old → new`).
/// A span holding zero or several tokens of its form, or a token that
/// starts or ends on a separator, is a refusal — never a guess about
/// which run to replace.
pub fn render(
    text: &str,
    label: &str,
    zh: bool,
    render: &dyn Fn(&str) -> String,
) -> (String, Vec<String>) {
    let mut out = String::with_capacity(text.len());
    let mut notes = Vec::new();
    let mut last = 0;
    for chip in chips(text, label) {
        let body = &text[chip.span.clone()];
        let runs = tokens(body, Form::of(&chip.id), zh);
        assert_eq!(
            runs.len(),
            1,
            "{label}: chip {} span {body:?} holds {} tokens of its form, not one",
            chip.id,
            runs.len()
        );
        let have = &body[runs[0].clone()];
        assert!(
            !have.starts_with(['.', ',']) && !have.ends_with(['.', ',']),
            "{label}: chip {} span {body:?} holds a malformed token {have:?}",
            chip.id
        );
        let want = render(&chip.id);
        if have != want {
            notes.push(format!("{label}: {} {have} → {want}", chip.id));
        }
        out.push_str(&text[last..chip.span.start + runs[0].start]);
        out.push_str(&want);
        last = chip.span.start + runs[0].end;
    }
    out.push_str(&text[last..]);
    (out, notes)
}
