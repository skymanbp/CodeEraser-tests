//! Unit-test scratch directories (compiled under cfg(test) only):
//! the one tmp-dir scaffold the in-crate test modules share — the
//! dedup ratchet caught join::churn_unit growing a second copy of
//! deadcode's observe-test setup, so the scaffold became a throat.

use std::path::{Path, PathBuf};

/// Fresh empty scratch dir keyed by tag + pid (parallel-test safe);
/// callers remove it themselves when the assertion needs a clean end.
pub fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ce-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// A scratch tree from (relative path, text) pairs, parents made —
/// the manifest-and-file setup the target and mount facts read from
/// disk, written once here rather than as a per-test mkdir/write run
/// (the clone gate's next-most-rhyming shape after the tuple table).
pub fn write_tree(root: &Path, files: &[(&str, &str)]) {
    for (rel, text) in files {
        let at = root.join(rel);
        at.parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&at, text))
            .unwrap_or_else(|e| panic!("{rel}: {e}"));
    }
}

/// A case table written as ONE literal, as (language, the N header
/// columns after the extension, the body): blocks separated by a `====`
/// line, each a header `ext @@ column @@ …` over its body. One literal
/// rather than a slice of typed rows: rows of any one tuple or struct
/// shape repeat every dozen tokens, and the clone gate reads such a
/// table as clones of itself. The column count is the table's shape,
/// so it is checked here once — each caller destructures its columns
/// in the loop pattern, where every caller used to convert them.
pub fn blocks<const N: usize>(
    table: &str,
) -> impl Iterator<Item = (crate::scan::lang::Lang, [&str; N], &str)> {
    table.trim().split("\n====\n").map(|block| {
        let (head, body) = block.split_once('\n').expect("a header line over a body");
        let mut cols = head.split(" @@ ");
        let ext = cols.next().expect("an extension column");
        let lang = crate::scan::lang::Lang::from_path(Path::new(&format!("x.{ext}")))
            .expect("a known extension");
        let cols: Vec<&str> = cols.collect();
        let cols = <[&str; N]>::try_from(cols)
            .unwrap_or_else(|c| panic!("{N} columns after the extension, not {}: {head}", c.len()));
        (lang, cols, body)
    })
}

/// A node literal for wire-side tests.
pub fn node(path: &str, unit: &str, kind: i64) -> crate::graph::nodes::Node {
    crate::graph::nodes::Node {
        path: path.into(),
        unit: unit.into(),
        kind,
        foreign: false,
    }
}

/// One per-declaration word case as one line: the source, then
/// ` ⇒`, then every unit `measure` must find, in source order, as
/// `name:letters` with `bit_of` reading each letter (`-` = none); an
/// empty right side asserts that `measure` finds nothing at all. A
/// string per case rather than a (source, table) tuple: a run of such
/// tuples is this repo's most-rhyming token shape, and its own clone
/// gate said so when the visibility tables were drafted — the mention
/// category word measures its tables through this same throat.
pub fn check_word_case(
    lang: crate::scan::lang::Lang,
    case: &str,
    measure: impl Fn(&str, crate::scan::lang::Lang) -> Vec<(String, i64)>,
    bit_of: impl Fn(char) -> i64,
) {
    let (src, want) = case.split_once(" ⇒").expect("case has ` ⇒`");
    let want: Vec<(String, i64)> = want
        .split_whitespace()
        .map(|unit| {
            let (name, letters) = unit.rsplit_once(':').expect("name:letters");
            (name.to_string(), letters.chars().map(&bit_of).sum())
        })
        .collect();
    assert_eq!(measure(src, lang), want, "{src:?}");
}
