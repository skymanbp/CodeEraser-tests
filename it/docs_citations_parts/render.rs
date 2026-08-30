//! The rendering half: a citation's numbers spelled from where its
//! text resolved — the label's `file:a-b` (or bare `a`, or the first
//! segment of a comma list) and the link's `#La[-Lb]`. A page whose
//! rendering differs from what it says is drift: named on a plain run,
//! rewritten under CE_BLESS=1, the page outside each citation's span
//! byte-for-byte untouched (so a CRLF page stays CRLF).

use super::Citation;
use super::ledger::Resolved;
use std::fs;
use std::path::Path;

/// `file:a` / `file:a-b` / bare `a` / `file:a-b,c,d` with the first
/// segment re-spelled; a range stays a range only when an end is
/// known. A label naming no line is returned as written.
pub fn label(label: &str, line: usize, end: Option<usize>) -> String {
    let (head, tail) = label.rsplit_once(':').map_or(("", label), |(h, t)| (h, t));
    let (first, rest) = tail.split_once(',').map_or((tail, ""), |(f, r)| (f, r));
    if super::label_lines(label).is_none() {
        return label.to_string();
    }
    let spelled = match end {
        Some(e) if first.contains('-') => format!("{line}-{e}"),
        _ => line.to_string(),
    };
    let colon = if head.is_empty() { "" } else { ":" };
    let comma = if rest.is_empty() { "" } else { "," };
    format!("{head}{colon}{spelled}{comma}{rest}")
}

/// `target#La`, plus `-Lb` where the link carried a range end.
pub fn link(link: &str, line: usize, end: Option<usize>) -> String {
    let (target, anchor) = link.split_once("#L").expect("a citation link carries #L");
    match end {
        Some(e) if anchor.contains("-L") => format!("{target}#L{line}-L{e}"),
        _ => format!("{target}#L{line}"),
    }
}

fn spelled(c: &Citation, r: &Resolved) -> String {
    format!(
        "[{}]({})",
        label(&c.label, r.line, r.end),
        link(&c.link, r.line, r.end)
    )
}

/// The page with every resolved citation re-spelled in place.
pub fn page(text: &str, cites: &[(&Citation, &Resolved)]) -> String {
    let mut out = text.to_string();
    let mut sorted: Vec<_> = cites.iter().collect();
    sorted.sort_by_key(|(c, _)| std::cmp::Reverse(c.span.0));
    for (c, r) in sorted {
        out.replace_range(c.span.0..c.span.1, &spelled(c, r));
    }
    out
}

/// One note per citation whose spelling on the page differs from its
/// rendering.
pub fn drift(text: &str, cites: &[(&Citation, &Resolved)]) -> Vec<String> {
    cites
        .iter()
        .filter_map(|(c, r)| {
            let now = &text[c.span.0..c.span.1];
            let want = spelled(c, r);
            (now != want).then(|| format!("{}:{}: {now} -> {want}", c.citing, c.citing_line))
        })
        .collect()
}

/// Under CE_BLESS=1 the rendering is written when it differs; on a
/// plain run the drift notes are returned for the gate to fail on.
pub fn settle(
    root: &Path,
    citing: &str,
    text: &str,
    cites: &[(&Citation, &Resolved)],
) -> Vec<String> {
    let notes = drift(text, cites);
    if notes.is_empty() || !crate::facts::blessing() {
        return notes;
    }
    fs::write(root.join(citing), page(text, cites)).expect("re-render citations");
    println!("re-rendered {} citations in {citing}", notes.len());
    Vec::new()
}

#[test]
fn labels_and_links_keep_their_shape() {
    // (spelling, line, end, want) — a spelling carrying `#L` is a link
    let table: &[(&str, usize, Option<usize>, &str)] = &[
        ("Cost.hs:42-55", 44, Some(57), "Cost.hs:44-57"),
        ("Cost.hs:42", 44, Some(57), "Cost.hs:44"),
        ("141", 143, None, "143"),
        ("Split.hs:2-5,24,202", 3, Some(6), "Split.hs:3-6,24,202"),
        ("Cost.hs:42-55", 44, None, "Cost.hs:44"),
        ("§4.2", 9, None, "§4.2"),
        ("../x.rs#L42", 44, Some(57), "../x.rs#L44"),
        ("../x.rs#L42-L55", 44, Some(57), "../x.rs#L44-L57"),
        ("../x.rs#L42-L55", 44, None, "../x.rs#L44"),
    ];
    for (spelling, line, end, want) in table {
        let got = if spelling.contains("#L") {
            link(spelling, *line, *end)
        } else {
            label(spelling, *line, *end)
        };
        assert_eq!(got, *want, "{spelling} @ L{line}");
    }
}

#[test]
fn a_page_is_respelled_only_inside_the_spans() {
    let text = "see [a.rs:1-2](../a.rs#L1) and [3](../a.rs#L3)\r\n";
    let c1 = cite(text, "a.rs:1-2", "../a.rs#L1", 1);
    let c2 = cite(text, "3", "../a.rs#L3", 3);
    let r1 = resolved(5, Some(6));
    let r2 = resolved(9, None);
    let cites = [(&c1, &r1), (&c2, &r2)];
    assert_eq!(
        page(text, &cites),
        "see [a.rs:5-6](../a.rs#L5) and [9](../a.rs#L9)\r\n"
    );
    assert_eq!(drift(text, &cites).len(), 2);
    assert!(drift(text, &[(&c1, &resolved(1, Some(2)))]).is_empty());
}

#[cfg(test)]
fn resolved(line: usize, end: Option<usize>) -> Resolved {
    Resolved {
        line,
        end,
        window: super::anchor::Window {
            text: String::new(),
            head: 0,
        },
    }
}

#[cfg(test)]
fn cite(text: &str, label: &str, link: &str, line: usize) -> Citation {
    let whole = format!("[{label}]({link})");
    let start = text.find(&whole).expect("citation present");
    Citation {
        citing: "p.md".into(),
        citing_line: 1,
        label: label.into(),
        link: link.into(),
        target: "../a.rs".into(),
        line,
        end: None,
        span: (start, start + whole.len()),
    }
}
