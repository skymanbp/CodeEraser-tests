use super::*;

/// The erase width is what the terminal actually shows. Both
/// tables' rows are measured, because the bug this guards is
/// zh-only: ASCII would have passed a char count too.
#[test]
fn columns_counts_cells_not_chars() {
    assert_eq!(columns("commits 3/9"), 11);
    assert_eq!(columns("正在建立引用图"), 14);
    assert_eq!(columns("提交 3/9"), 4 + 1 + 3);
    assert_eq!(columns("存活归因：3/9 文件"), 8 + 2 + 3 + 1 + 4);
}

/// Every phase indexes its own row, and the table keeps exactly
/// one row past the last of them — the unknown case i18n::coded
/// falls back to. A row added without a phase, or a phase past
/// the table, is a silent mislabel of a live measurement.
#[test]
fn phase_positions_and_table_agree() {
    let all = [
        Phase::Window,
        Phase::Commits,
        Phase::Survival,
        Phase::Index,
        Phase::Graph,
        Phase::Assemble,
        Phase::Measure,
    ];
    for (i, p) in all.iter().enumerate() {
        assert_eq!(*p as i64, i as i64, "phases are their own positions");
    }
    assert_eq!(phases().len(), all.len() + 1, "one unknown row, no more");
    assert_eq!(phases().last().expect("unknown row").0, "working");
    assert!(
        PHASE_TSV.lines().all(|l| l.contains('\t')),
        "every row carries both languages"
    );
}

/// Ticking an unarmed face is a no-op, not a panic and not a
/// write: this is the guarantee every library caller relies on.
#[test]
fn unarmed_ticks_are_silent() {
    at(Phase::Commits, 1, 2);
    step(Phase::Graph);
    drop(span());
}
