use super::*;

fn row(span: (i64, i64)) -> Row {
    Row {
        class: "verbatim_doc",
        eraseable: true,
        reason: "",
        path: "f.md".into(),
        span: Some(span),
        provenance: "docdup".into(),
        sites: 0,
        hash: 1,
    }
}

/// v2.35 G6 (R3-14): spans (5,6) and (9,10) in a 20-line file have
/// windows 2–9 and 6–13, which touch — ONE hunk 2–13 whose counts say
/// what it removes (12 old lines, 8 kept), each line printed once;
/// a span whose window starts past the last hunk's tail + 1 opens a
/// second hunk, its new start shifted by what the first removed.
#[test]
fn touching_windows_coalesce_into_one_hunk() {
    let text: String = (1..=20).map(|n| format!("l{n}\n")).collect();
    let rows = [row((5, 6)), row((9, 10)), row((18, 18))];
    let mut out = String::new();
    file_diff(&mut out, &text, &rows.iter().collect::<Vec<_>>());
    let heads: Vec<&str> = out.lines().filter(|l| l.starts_with("@@")).collect();
    let named: Vec<usize> = heads.iter().map(|h| h.matches("## ").count()).collect();
    assert!(
        heads.len() == 2
            && heads[0].starts_with("@@ -2,12 +2,8 @@ ## verbatim_doc docdup")
            && heads[1].starts_with("@@ -15,6 +11,5 @@")
            && named == [2, 1],
        "two hunks, the first naming both its rows: {out}"
    );
    // a file header line (`--- f.md`) never reads as "l6"
    let l6 = out.lines().filter(|l| l.get(1..) == Some("l6"));
    assert_eq!(l6.collect::<Vec<_>>(), ["-l6"], "line 6 once, removed");
}
