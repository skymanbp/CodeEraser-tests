//! Anchor windows — the text that IS a citation's fact.
//!
//! One trimmed line is the natural anchor, and for most citations it
//! is the whole window. But `}` is a line, and so is `-- |`: an anchor
//! that short, or one the target spells twice, cannot say WHERE the
//! citation points once the file moves. The window rule closes both
//! without a human: grow the window downward from the cited line
//! until it weighs at least `MIN_CHARS` non-space characters AND
//! occurs once in the target; when EOF stops the growth (a range-end
//! citation on a file's last brace), grow upward instead and record
//! how many lines sit above the cited one (`head`). A window that
//! cannot form — a target shorter than `MIN_CHARS` — is the one
//! `needs_human` refusal left.

pub const MIN_CHARS: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// Trimmed lines joined by `\n`.
    pub text: String,
    /// Lines of the window above the cited line.
    pub head: usize,
}

fn weight(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

fn join(lines: &[String], from: usize, to: usize) -> String {
    lines[from..to]
        .iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Start lines (1-based) at which the window's lines appear
/// consecutively, each trimmed.
pub fn occurrences(lines: &[String], text: &str) -> Vec<usize> {
    let parts: Vec<&str> = text.split('\n').collect();
    let k = parts.len();
    if lines.len() < k {
        return Vec::new();
    }
    (0..=lines.len() - k)
        .filter(|&i| (0..k).all(|j| lines[i + j].trim() == parts[j]))
        .map(|i| i + 1)
        .collect()
}

/// Whether `text` sits at `line` as a window whose `head` lines
/// precede the cited one.
pub fn at(lines: &[String], text: &str, head: usize, line: usize) -> bool {
    line > head && occurrences(lines, text).contains(&(line - head))
}

/// The minimal window around `line` (1-based) that weighs at least
/// `MIN_CHARS` and occurs once — downward first, upward once EOF
/// stops the growth. None when no window of the whole file does.
pub fn seed(lines: &[String], line: usize) -> Option<Window> {
    let (mut lo, mut hi) = (line - 1, line);
    loop {
        let text = join(lines, lo, hi);
        if weight(&text) >= MIN_CHARS && occurrences(lines, &text).len() == 1 {
            return Some(Window {
                text,
                head: line - 1 - lo,
            });
        }
        if hi < lines.len() {
            hi += 1;
        } else if lo > 0 {
            lo -= 1;
        } else {
            return None;
        }
    }
}

#[cfg(test)]
fn file(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

#[test]
fn windows_by_table() {
    // (file, cited line, window text, head)
    let twins = "}\nfn first_of_two() {}\n}\nfn second_of_two() {}\n";
    let table: &[(&str, usize, &str, usize)] = &[
        // one line carrying enough is the whole window
        (
            "fn a() {\n    let long_enough_name = 1;\n}\n",
            2,
            "let long_enough_name = 1;",
            0,
        ),
        // a short, duplicated `}` grows down to the line that tells them apart
        (twins, 1, "}\nfn first_of_two() {}", 0),
        (twins, 3, "}\nfn second_of_two() {}", 0),
        // at EOF the window climbs: `body();` + `}` weigh 8, the signature makes 12
        (
            "fn only_function() {\n    body();\n}\n",
            3,
            "fn only_function() {\nbody();\n}",
            2,
        ),
    ];
    for (src, line, text, head) in table {
        let f = file(src);
        let w = seed(&f, *line).expect(text);
        assert_eq!(
            (w.text.as_str(), w.head),
            (*text, *head),
            "{src:?} @ L{line}"
        );
        assert!(at(&f, &w.text, w.head, *line));
        assert_eq!(occurrences(&f, &w.text), vec![line - head]);
    }
}

#[test]
fn a_tiny_file_needs_a_human() {
    let f = file("fn a() {\n    let long_enough_name = 1;\n}\n");
    assert!(!at(&f, "let long_enough_name = 1;", 0, 1));
    assert_eq!(seed(&file("x\ny\n"), 1), None);
    assert_eq!(occurrences(&file("x"), "a\nb\nc"), Vec::<usize>::new());
}
