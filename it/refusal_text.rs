//! A deliberate refusal must not READ like a broken build.
//!
//! `ce` states its named refusals in `ensure!` / `bail!` / `panic!`
//! messages. Two of them carried the fourteen and ten spaces a lost
//! `\` line continuation leaves behind — the shape a multi-line
//! literal takes when it is moved between files. A user meeting a real
//! cap-mirror drift reads the gap as a formatting accident and files
//! "the build is broken" instead of the drift the sentence names.
//!
//! Neither was visible to any gate: they are string literals, and
//! every byte gate here compares a file with its own generator. This
//! one reads the source instead. Report lines are deliberately column
//! aligned with runs of spaces, so the scan is scoped to the refusal
//! macros and never looks at `println!` / `write!` / `format!`.

use crate::common;

const REFUSALS: [&str; 4] = ["ensure!(", "bail!(", "panic!(", "anyhow!("];

/// Every string literal on `line`, unquoted, honouring backslash
/// escapes so a `\"` inside a message does not end it early.
fn literals(line: &str) -> Vec<String> {
    let (mut out, mut cur, mut inside, mut esc) = (Vec::new(), String::new(), false, false);
    for c in line.chars() {
        if !inside {
            inside = c == '"';
        } else if esc {
            esc = false;
            cur.push(c);
        } else if c == '\\' {
            esc = true;
        } else if c == '"' {
            out.push(std::mem::take(&mut cur));
            inside = false;
        } else {
            cur.push(c);
        }
    }
    out
}

/// The 1-based lines a refusal macro's arguments occupy: the opener,
/// then every line up to and including the first one that starts with
/// `)` — the shape rustfmt gives every one of these call sites.
fn refusal_regions(text: &str) -> Vec<(usize, String)> {
    let (mut out, mut open) = (Vec::new(), false);
    for (i, line) in text.lines().enumerate() {
        let opens = REFUSALS.iter().any(|m| line.contains(m));
        if opens || open {
            out.push((i + 1, line.to_string()));
            open = !line.trim_start().starts_with(')');
        }
    }
    out
}

#[test]
fn no_named_refusal_reads_like_a_broken_build() {
    let mut files = Vec::new();
    common::files_with_ext(&common::repo_root().join("cli/src"), "rs", &mut files);
    assert!(files.len() > 50, "cli/src walked: {} files", files.len());
    let mut bad = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).expect("read");
        for (line_no, line) in refusal_regions(&text) {
            for lit in literals(&line) {
                if lit.contains("  ") {
                    bad.push(format!("{}:{line_no} {lit:?}", f.display()));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a named refusal reads like a broken build (a lost line \
         continuation leaves the indentation inside the literal):\n{}",
        bad.join("\n")
    );
}

/// The scanner itself, on the two real shapes it exists to tell apart.
#[test]
fn the_scan_sees_the_refusal_and_not_the_report_column() {
    let src = "    println!(\"name       count\");\n\
               anyhow::ensure!(\n\
               \x20   cond,\n\
               \x20   \"a refusal —          with the gap\",\n\
               );\n";
    let hits: Vec<String> = refusal_regions(src)
        .iter()
        .flat_map(|(_, l)| literals(l))
        .filter(|l| l.contains("  "))
        .collect();
    assert_eq!(hits, vec!["a refusal —          with the gap".to_string()]);
    // the escape is consumed, so a quoted quote never ends a literal early
    assert_eq!(
        literals(r#"say("a \" quote", "two")"#),
        ["a \" quote", "two"]
    );
}
