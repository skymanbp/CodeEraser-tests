//! Each hand-written surface speaks one language (plan v2.21 ⑤, S7):
//! README.md and the four English site pages carry no CJK outside the
//! exact language-switch strings; README.zh.md and the four Chinese
//! pages carry no run of English prose. Code (fences, spans, `<code>`,
//! `<pre>`, `<kbd>`), comments, tags, link targets and every generated
//! block (`<!-- name:begin -->` … `<!-- name:end -->`, whose language
//! belongs to its renderer) are cut before judging — only the prose a
//! human wrote is read. docs/ and its ledgers are out of scope by the
//! plan (English, with the CLI's own `--lang zh` as the Chinese face).
//!
//! The same step gave every static SVG a `<title>` — the name a browser
//! tab or a raw-file view shows — so the second leg holds that, and
//! holds the docs/ and site/ twins to identical bytes.

use crate::common::repo_root;
use std::path::Path;

/// (page, Chinese?)
const SURFACES: &[(&str, bool)] = &[
    ("README.md", false),
    ("README.zh.md", true),
    ("site/index.html", false),
    ("site/how/index.html", false),
    ("site/stack/index.html", false),
    ("site/bench/index.html", false),
    ("site/zh/index.html", true),
    ("site/zh/how/index.html", true),
    ("site/zh/stack/index.html", true),
    ("site/zh/bench/index.html", true),
];

/// The language-switch strings an English page carries, exactly.
const SWITCH: &[&str] = &["中文"];

/// An English run this long outside code is a sentence, not a name.
const RUN: usize = 4;

/// The static SVGs, docs/ twin first where one exists.
const SVGS: &[(&str, Option<&str>)] = &[
    (
        "docs/assets/architecture.en.svg",
        Some("site/assets/architecture.en.svg"),
    ),
    (
        "docs/assets/architecture.zh.svg",
        Some("site/assets/architecture.zh.svg"),
    ),
    (
        "docs/assets/judgment.en.svg",
        Some("site/assets/judgment.en.svg"),
    ),
    (
        "docs/assets/judgment.zh.svg",
        Some("site/assets/judgment.zh.svg"),
    ),
    ("docs/assets/stack.svg", Some("site/assets/stack.svg")),
    ("docs/assets/stack.zh.svg", Some("site/assets/stack.zh.svg")),
    ("site/assets/methodology.svg", None),
    ("site/assets/methodology.zh.svg", None),
];

/// Every `open` … `close` span replaced by one space.
fn cut(text: &str, open: &str, close: &str) -> String {
    let mut out = text.to_string();
    let mut from = 0;
    while let Some(i) = out[from..].find(open).map(|i| i + from) {
        let Some(j) = out[i + open.len()..].find(close) else {
            break;
        };
        out.replace_range(i..i + open.len() + j + close.len(), " ");
        from = i + 1;
    }
    out
}

/// Generated blocks whole: `<!-- name:begin -->` … `<!-- name:end -->`.
fn cut_generated(text: &str) -> String {
    let mut out = text.to_string();
    while let Some(b) = out.find(":begin -->") {
        let start = out[..b].rfind("<!-- ").unwrap_or(b);
        let Some(e) = out[b..].find(":end -->") else {
            break;
        };
        out.replace_range(start..b + e + ":end -->".len(), " ");
    }
    out
}

/// The hand-written prose of a page.
fn prose(rel: &str, text: &str) -> String {
    let mut t = cut_generated(text);
    t = cut(&t, "<!--", "-->");
    if rel.ends_with(".html") {
        for tag in ["script", "style", "code", "pre", "kbd"] {
            t = cut(&t, &format!("<{tag}"), &format!("</{tag}>"));
        }
    } else {
        t = cut(&t, "```", "```");
        t = cut(&t, "`", "`");
        t = cut(&t, "](", ")");
    }
    cut(&t, "<", ">")
}

fn is_cjk(c: char) -> bool {
    matches!(c, '\u{3000}'..='\u{303F}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{FF00}'..='\u{FFEF}')
}

/// A word starts with a letter; `--fail-under` and `-q` are flags.
fn is_word(token: &str) -> bool {
    token.starts_with(|c: char| c.is_ascii_alphabetic())
        && token
            .chars()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, '\'' | '’' | '-'))
}

/// The longest run of consecutive English words on one line.
fn english_run(line: &str) -> (usize, String) {
    let mut best = (0, String::new());
    let mut run: Vec<&str> = Vec::new();
    for token in line.split_whitespace().chain(std::iter::once("")) {
        if is_word(token) {
            run.push(token);
        } else {
            if run.len() > best.0 {
                best = (run.len(), run.join(" "));
            }
            run.clear();
        }
    }
    best
}

fn errors_of(rel: &str, zh: bool, text: &str) -> Vec<String> {
    let mut prose = prose(rel, text);
    if !zh {
        for s in SWITCH {
            prose = prose.replace(s, " ");
        }
    }
    let mut errors = Vec::new();
    for (i, line) in prose.lines().enumerate() {
        if zh {
            let (n, run) = english_run(line);
            if n >= RUN {
                errors.push(format!(
                    "{rel}:{}: English prose on a Chinese page: `{run}`",
                    i + 1
                ));
            }
        } else if let Some(c) = line.chars().find(|&c| is_cjk(c)) {
            errors.push(format!("{rel}:{}: CJK on an English page: `{c}`", i + 1));
        }
    }
    errors
}

#[test]
fn each_surface_speaks_its_own_language() {
    let root = repo_root();
    let errors: Vec<String> = SURFACES
        .iter()
        .flat_map(|(rel, zh)| errors_of(rel, *zh, &crate::facts::read(&root, rel)))
        .collect();
    assert!(
        errors.is_empty(),
        "docs language errors:\n{}",
        errors.join("\n")
    );
}

fn svg_title(root: &Path, rel: &str) -> String {
    let text = crate::facts::read(root, rel);
    let open = text.find('>').expect("an svg opening tag") + 1;
    let rest = text[open..].trim_start();
    // `<title id="…">` is the accessible-name form archify emits
    // (aria-labelledby points at the id); attributes are allowed
    let title = rest
        .strip_prefix("<title")
        .and_then(|t| t.split_once('>'))
        .and_then(|(_, t)| t.split_once("</title>"))
        .map(|(t, _)| t.trim().to_string())
        .unwrap_or_default();
    assert!(
        !title.is_empty(),
        "{rel}: the first child of <svg> is not a non-empty <title>"
    );
    title
}

#[test]
fn static_svgs_are_titled_and_twinned() {
    let root = repo_root();
    for (rel, twin) in SVGS {
        let title = svg_title(&root, rel);
        if let Some(twin) = twin {
            assert_eq!(
                svg_title(&root, twin),
                title,
                "{rel} / {twin} titles differ"
            );
            assert_eq!(
                std::fs::read(root.join(rel)).expect("svg"),
                std::fs::read(root.join(twin)).expect("twin svg"),
                "{rel} and {twin} are not byte-identical twins"
            );
        }
    }
}

#[test]
fn the_cuts_leave_only_prose() {
    let md = "a [x](http://z) `code` <!-- c --> <!-- demo:begin -->\n| 中 |\n<!-- demo:end --> b ```\nfoo\n``` end";
    // the link target goes from `](` through `)`; the label's bracket stays
    assert_eq!(
        prose("p.md", md).split_whitespace().collect::<Vec<_>>(),
        ["a", "[x", "b", "end"]
    );
    let html =
        "<p>hi <code>x y z w v</code> <a href=\"/zh/\">中文</a></p><script>var a = b c d;</script>";
    assert_eq!(
        prose("p.html", html).split_whitespace().collect::<Vec<_>>(),
        ["hi", "中文"]
    );
    assert_eq!(english_run("这 is a long enough run 了").0, 5);
    assert_eq!(english_run("ce check --fail-under 946").0, 2);
    assert!(errors_of("p.html", false, "<p>中文</p>").is_empty());
    assert_eq!(
        errors_of("p.html", false, "<p>中文 与 English</p>").len(),
        1
    );
    assert_eq!(errors_of("p.md", true, "the turn may end here").len(), 1);
}
