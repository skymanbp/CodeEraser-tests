//! The lowering's shared test helpers (plan v2.31 step 4 A2): the tree
//! contract the core's CE.Flow.Tree and CE.Flow.Shape enforce, each
//! check written again here in Rust, and the expectation tables every
//! per-rule and per-language leg is written in. A table is one text:
//! blocks split by `====`, each headed `ext @@ name @@ why`, its source,
//! then after `----` the first unit's rows — `s seq parent kind flags
//! aux`, `v v declSeq flags name`, `x seq v mode`.
//!
//! Imports come through `super`: the lib mounts this file under
//! flow::lower. The crosscheck corpus leg (flow_shape) lives here too.

use super::{Lang, Unit, lower_file};
use std::path::Path;

const LEAVES: [i64; 7] = [1, 9, 10, 11, 12, 13, 15];

/// The shape contract, or the first offence.
pub(crate) fn shape(u: &Unit) -> Result<(), String> {
    for (i, &row) in u.stmts.iter().enumerate() {
        let offence = head_offence(i as i64, row).or_else(|| body_offence(u, row));
        if let Some(what) = offence {
            return Err(format!("stmt {i}: {what}"));
        }
    }
    tables(u)
}

fn kind_of(u: &Unit, s: i64) -> i64 {
    u.stmts.get(s as usize).map_or(-1, |r| r[2])
}

fn kids_of(u: &Unit, s: i64) -> Vec<i64> {
    u.stmts.iter().filter(|r| r[1] == s).map(|r| r[2]).collect()
}

fn ancestors(u: &Unit, mut s: i64) -> Vec<i64> {
    let mut out = Vec::new();
    while s >= 0 {
        s = u.stmts[s as usize][1];
        out.push(s);
    }
    out
}

/// A row's own fields: its place, its ranges, flags on their kinds.
fn head_offence(i: i64, [seq, parent, k, flags, aux]: [i64; 5]) -> Option<&'static str> {
    let bit = |b: i64| flags >> b & 1 == 1;
    if seq != i || parent >= seq || parent < -1 {
        return Some("seq not consecutive or parent not before");
    }
    if !(0..=15).contains(&k) || !(0..=31).contains(&flags) {
        return Some("kind or flags out of range");
    }
    if bit(0) && k != 2 && k != 4 || bit(1) && k != 3 || bit(2) && k != 5 {
        return Some("flag on the wrong kind");
    }
    if aux != 0 && ![11, 12, 13].contains(&k) {
        return Some("aux on a non-jump");
    }
    None
}

/// A row against the tree: its jump's target, its children.
fn body_offence(u: &Unit, [seq, parent, k, flags, aux]: [i64; 5]) -> Option<&'static str> {
    if !jump_ok(u, seq, k, aux) {
        return Some("jump target");
    }
    let own = kids_of(u, seq);
    let has_else = flags & 1 == 1;
    if LEAVES.contains(&k) && !own.is_empty() {
        return Some("children under a leaf");
    }
    if k == 2 && own.len() != 1 + has_else as usize {
        return Some("if branches disagree with has_else");
    }
    if k == 4 && (own.iter().any(|c| *c != 5) || has_else && own.is_empty()) {
        return Some("switch arms");
    }
    let outside = |p: i64| kind_of(u, parent) != p;
    if k == 5 && outside(4) || [7, 8].contains(&k) && outside(6) {
        return Some("case or handler outside its parent");
    }
    (k == 6 && !try_order(&own)).then_some("try order")
}

/// A break out to an enclosing loop or switch, a continue to an
/// enclosing loop, a goto to a label.
fn jump_ok(u: &Unit, seq: i64, k: i64, aux: i64) -> bool {
    let up = ancestors(u, seq);
    match k {
        11 => up.contains(&aux) && [3, 4].contains(&kind_of(u, aux)),
        12 => up.contains(&aux) && kind_of(u, aux) == 3,
        13 => kind_of(u, aux) == 14,
        _ => true,
    }
}

/// Body statements, then catches, then at most one finally.
fn try_order(own: &[i64]) -> bool {
    let handler = |c: i64| c == 7 || c == 8;
    let ordered = own
        .windows(2)
        .all(|w| (!handler(w[0]) || handler(w[1])) && !(w[0] == 8 && w[1] == 7));
    ordered && own.iter().filter(|c| **c == 8).count() <= 1
}

/// The variable and access rows, the parameter count, the legend.
fn tables(u: &Unit) -> Result<(), String> {
    let n = u.stmts.len() as i64;
    for (i, &[v, decl, flags]) in u.vars.iter().enumerate() {
        let param = flags & 1 == 1;
        let ok = v == i as i64 && (0..=15).contains(&flags) && (param == (decl == -1)) && decl < n;
        if !ok {
            return Err(format!("var {i}: {v} {decl} {flags}"));
        }
    }
    let mut last = 0;
    for (i, &[seq, v, mode]) in u.uses.iter().enumerate() {
        let ok = seq >= last && seq < n && (v as usize) < u.vars.len() && (0..=2).contains(&mode);
        if !ok {
            return Err(format!("use {i}: {seq} {v} {mode}"));
        }
        last = seq;
    }
    let params = u.vars.iter().filter(|r| r[2] & 1 == 1).count() as u32;
    let l = &u.legend;
    let stmts = [l.stmt_at.len(), l.stmt_end.len(), l.stmt_text.len()]
        .iter()
        .all(|x| *x as i64 == n);
    let vars = [l.var_name.len(), l.var_at.len()]
        .iter()
        .all(|x| *x == u.vars.len());
    if params != u.params || !stmts || !vars {
        return Err("params or legend lengths".to_owned());
    }
    Ok(())
}

pub(crate) fn assert_shape(u: &Unit) {
    if let Err(e) = shape(u) {
        panic!("{} (line {}): {e}\n{}", u.name, u.start_line, render(u));
    }
}

/// A unit's rows as the tables write them.
pub(crate) fn render(u: &Unit) -> String {
    let mut out = Vec::new();
    out.extend(
        u.stmts
            .iter()
            .map(|r| format!("s {} {} {} {} {}", r[0], r[1], r[2], r[3], r[4])),
    );
    let names = u.legend.var_name.iter();
    out.extend(
        u.vars
            .iter()
            .zip(names)
            .map(|(r, n)| format!("v {} {} {} {n}", r[0], r[1], r[2])),
    );
    out.extend(
        u.uses
            .iter()
            .map(|r| format!("x {} {} {}", r[0], r[1], r[2])),
    );
    out.join("\n")
}

/// One block: its header, its source, its expected rows.
struct Block {
    head: String,
    source: String,
    want: Vec<String>,
}

fn blocks(table: &str) -> Vec<Block> {
    let mut out = Vec::new();
    for chunk in table.split("\n====\n") {
        let mut lines = chunk.trim().lines();
        let Some(head) = lines.next() else { continue };
        let body: Vec<&str> = lines.collect();
        let cut = body.iter().position(|l| *l == "----").unwrap_or(body.len());
        let rows = body
            .get(cut + 1..)
            .unwrap_or_default()
            .iter()
            .map(|l| l.trim());
        out.push(Block {
            head: head.to_owned(),
            source: body[..cut].join("\n"),
            want: rows.filter(|l| !l.is_empty()).map(str::to_owned).collect(),
        });
    }
    out
}

/// Every block of a table lowered and compared; every difference is
/// reported, both sides printed.
pub(crate) fn run_table(table: &str) {
    let mut failures = Vec::new();
    for b in blocks(table) {
        let ext = b.head.split(" @@ ").next().unwrap_or_default();
        let lang = Lang::from_path(Path::new(&format!("x.{ext}"))).expect("a known extension");
        let lowered = lower_file(&b.source, lang).expect("a flow language");
        let reasons: Vec<&str> = lowered
            .unlowered
            .iter()
            .map(|u| u.reason.as_str())
            .collect();
        lowered.units.iter().skip(1).for_each(assert_shape);
        let Some(unit) = lowered.units.first() else {
            failures.push(format!("{}: no unit ({reasons:?})", b.head));
            continue;
        };
        if let Err(e) = shape(unit) {
            failures.push(format!("{}: shape: {e}", b.head));
        }
        let got = render(unit);
        if b.want != got.lines().collect::<Vec<_>>() {
            failures.push(format!(
                "{}:\n-- want\n{}\n-- got\n{got}",
                b.head,
                b.want.join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The nine code-language crosscheck corpora (the flow_shape leg).
const CORPORA: [&str; 9] = [
    "python",
    "typescript",
    "rust",
    "go",
    "c",
    "cpp",
    "java",
    "lua",
    "r",
];

/// One corpus file lowered: its unit count, and every unit it could
/// not lower or that breaks the tree contract pushed onto `failures`.
fn corpus_file(path: &Path, failures: &mut Vec<String>) -> usize {
    let Some(lang) = Lang::from_path(path) else {
        return 0;
    };
    let text = std::fs::read_to_string(path).expect("a corpus file");
    let Some(done) = lower_file(&text, lang) else {
        return 0;
    };
    let at = |name: &str, line| format!("{}: {name} (line {line})", path.display());
    for u in &done.unlowered {
        failures.push(format!("{}: {}", at(&u.name, u.start_line), u.reason));
    }
    for u in &done.units {
        if let Err(e) = shape(u) {
            failures.push(format!("{}: {e}", at(&u.name, u.start_line)));
        }
    }
    done.units.len()
}

/// Every unit of every file in the nine corpora lowers, and every
/// lowered unit keeps the tree contract; a form a corpus holds that the
/// lowering cannot shape fails here, its reason named. It lives in the
/// unit tree, not the integration tree: the tests root's rules forbid
/// an it/ file reading a unit/ one (ce.rules `it_reads_unit`).
#[test]
fn every_crosscheck_unit_lowers_into_a_legal_tree() {
    let root = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../contracts/fixtures/crosscheck"
    ));
    let (mut units, mut failures) = (0, Vec::new());
    for corpus in CORPORA {
        let mut files: Vec<_> = std::fs::read_dir(root.join(corpus))
            .expect("a corpus")
            .flatten()
            .map(|e| e.path())
            .collect();
        files.sort();
        for path in files {
            units += corpus_file(&path, &mut failures);
        }
    }
    println!(
        "flow_shape: {units} units lowered, {} failures",
        failures.len()
    );
    assert!(units > 0, "no unit lowered");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
