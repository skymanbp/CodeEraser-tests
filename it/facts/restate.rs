//! `restate:` — a prose number bound to a cell of a table in the SAME
//! document (plan v2.21 S8). Booklet 13's §8 prose reads its own
//! acceptance table back ("zod typescript 197 / 1127 = 17.5 %"), and a
//! re-taken row left the prose behind — a quarter of the confirmed
//! drifts had this shape. The id names the table by one of its header
//! cells — any that no other table of the document shares (booklet 13
//! opens two tables with `corpus`; `survival` names the acceptance
//! one) — the row by its first non-empty cell, the column by its
//! header, each as a slug (lowercase, runs of non-alphanumerics → `-`):
//!
//!   restate:survival:zod-912f0f5:declared-exported#paren                  → 1127
//!   restate:survival:zod-912f0f5:unmentioned-exported/declared-exported#paren-pct1 → 17.5
//!
//! `#lead` is the integer a cell opens with, `#paren` the one inside
//! its parentheses; a `-pct1` form divides two such cells, ×100, to
//! one decimal (half up, in integers). The value is document-local —
//! never in the registry, never in the projection — and every lookup
//! that is not unique is a refusal naming what it saw.

pub struct Table {
    pub headers: Vec<String>,
    /// (row key, cells)
    pub rows: Vec<(String, Vec<String>)>,
}

/// Lowercase; every run of non-alphanumerics one `-`; trimmed.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// The cells of one `| a | b |` line.
fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

fn is_separator(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.trim_matches(['|', '-', ':', ' ']).is_empty()
}

/// Every Markdown table of `doc`: a header line, its separator, then
/// rows until the first line that is not a row.
pub fn tables(doc: &str) -> Vec<Table> {
    let lines: Vec<&str> = doc.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        if !(lines[i].trim_start().starts_with('|') && is_separator(lines[i + 1])) {
            i += 1;
            continue;
        }
        let headers: Vec<String> = cells(lines[i]).iter().map(|h| slug(h)).collect();
        let mut rows = Vec::new();
        i += 2;
        while i < lines.len() && lines[i].trim_start().starts_with('|') {
            let row = cells(lines[i]);
            let key = row
                .iter()
                .find(|c| !c.is_empty())
                .map(|c| slug(c))
                .unwrap_or_default();
            rows.push((key, row));
            i += 1;
        }
        out.push(Table { headers, rows });
    }
    out
}

fn only<'a, T>(what: &str, found: Vec<&'a T>, seen: &[String]) -> &'a T {
    match found.as_slice() {
        [one] => one,
        [] => panic!("restate: no {what}; the document has {seen:?}"),
        many => panic!("restate: {what} matches {} times", many.len()),
    }
}

/// The cell at (table, row, column) of `doc`.
fn cell(doc: &str, table: &str, row: &str, col: &str) -> String {
    let tables = tables(doc);
    let keys: Vec<String> = tables.iter().map(|t| t.headers.join("|")).collect();
    let t = only(
        &format!("table with a header `{table}`"),
        tables
            .iter()
            .filter(|t| t.headers.iter().any(|h| h == table))
            .collect(),
        &keys,
    );
    let row_keys: Vec<String> = t.rows.iter().map(|(k, _)| k.clone()).collect();
    let (_, cells) = only(
        &format!("row `{row}` in table `{table}`"),
        t.rows.iter().filter(|(k, _)| k == row).collect(),
        &row_keys,
    );
    only(
        &format!("column `{col}` in table `{table}`"),
        t.headers.iter().filter(|h| *h == col).collect(),
        &t.headers,
    );
    let i = t
        .headers
        .iter()
        .position(|h| h == col)
        .expect("unique above");
    cells.get(i).cloned().unwrap_or_default()
}

/// The first digit run at or after `from` in `cell`.
fn digits(cell: &str, from: usize) -> u64 {
    let rest = &cell[from..];
    let start = rest
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or_else(|| panic!("restate: cell {cell:?} carries no number"));
    let run: String = rest[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    run.parse().expect("a digit run")
}

/// `#lead` / `#paren` of one cell.
fn number(cell: &str, part: &str) -> u64 {
    match part {
        "lead" => digits(cell, 0),
        "paren" => digits(
            cell,
            cell.find('(')
                .unwrap_or_else(|| panic!("restate: cell {cell:?} has no parenthesis")),
        ),
        _ => panic!("restate: unknown part {part:?}"),
    }
}

/// 100·a/b to one decimal, half up, in integers.
fn pct1(a: u64, b: u64) -> String {
    assert!(b > 0, "restate: a ratio over zero");
    let tenths = (1000 * a + b / 2) / b;
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// The value a `restate:` id names in `doc`.
pub fn resolve(doc: &str, id: &str) -> String {
    let (path, form) = id.rsplit_once('#').expect("a #form suffix");
    let parts: Vec<&str> = path
        .strip_prefix("restate:")
        .expect("a restate: id")
        .split(':')
        .collect();
    let [table, row, cols] = parts.as_slice() else {
        panic!("restate id {id:?}: expected restate:<table>:<row>:<col>[/<col>]#<form>")
    };
    let (part, ratio) = form
        .strip_suffix("-pct1")
        .map_or((form, false), |p| (p, true));
    if !ratio {
        return number(&cell(doc, table, row, cols), part).to_string();
    }
    let (a, b) = cols
        .split_once('/')
        .unwrap_or_else(|| panic!("restate id {id:?}: a -pct1 form divides two columns"));
    pct1(
        number(&cell(doc, table, row, a), part),
        number(&cell(doc, table, row, b), part),
    )
}

#[test]
fn a_table_is_read_back_by_slugs() {
    // two tables open with `corpus`; only the first has a `U` column
    let doc = "text\n\n| corpus | U | declared (exported) | unmentioned (exported) |\n|---|---|---|---|\n| self @ this commit | 764 | 2021 (1102) | 297 (0) |\n| | | 1333 (309) | 307 (2) |\n| zod 912f0f5 | 536 | 1944 (1127) | 353 (197) |\n\nafter\n\n| corpus | rows |\n|---|---|\n| zod 912f0f5 | 3 |\n";
    let t = tables(doc);
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].headers[0], "corpus");
    assert_eq!(t[0].headers[2], "declared-exported");
    assert_eq!(resolve(doc, "restate:rows:zod-912f0f5:rows#lead"), "3");
    assert_eq!(t[0].rows[0].0, "self-this-commit");
    assert_eq!(t[0].rows[1].0, "1333-309");
    assert_eq!(
        resolve(doc, "restate:u:zod-912f0f5:declared-exported#paren"),
        "1127"
    );
    assert_eq!(
        resolve(doc, "restate:u:zod-912f0f5:declared-exported#lead"),
        "1944"
    );
    assert_eq!(
        resolve(
            doc,
            "restate:u:zod-912f0f5:unmentioned-exported/declared-exported#paren-pct1"
        ),
        "17.5"
    );
    assert_eq!(
        resolve(
            doc,
            "restate:u:self-this-commit:unmentioned-exported/declared-exported#paren-pct1"
        ),
        "0.0"
    );
    assert_eq!(pct1(313, 481), "65.1");
    assert_eq!(pct1(1, 8), "12.5");
    assert_eq!(pct1(1, 16), "6.3");
    assert_eq!(slug("declared (exported)"), "declared-exported");
}
