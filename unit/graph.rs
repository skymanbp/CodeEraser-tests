use super::{analyze, counts};

/// End-to-end walk on a real (temp) tree — `ce graph --sites`'
/// engine gets its own test instead of only a smoke number in a
/// commit message (Opus review). Includes a non-UTF-8 in-scope
/// file: lossy reading keeps the run alive and still detects.
#[test]
fn analyze_walks_counts_and_survives_non_utf8() {
    let dir = std::env::temp_dir().join(format!("ce-graph-walk-{}", std::process::id()));
    let fixtures: [(&str, &[u8]); 3] = [
        ("a.py", b"import os\n"),
        ("b.md", b"[x](./a.py)\n"),
        ("c.py", b"import sys\n\xff\xfe\n"),
    ];
    std::fs::create_dir_all(&dir).expect("mkdir");
    for (name, bytes) in fixtures {
        std::fs::write(dir.join(name), bytes).expect(name);
    }
    let files = analyze(&dir).expect("analyze");
    let by = counts(&files);
    assert_eq!(
        by.get(&("python", "import")),
        Some(&2),
        "lossy file still detected"
    );
    assert_eq!(by.get(&("markdown", "link")), Some(&1));
    std::fs::remove_dir_all(&dir).ok();
}

/// The scan-only arm never reaches this face: before plan v2.5's
/// boundary landed here, a grammarless .css/.js file fell through
/// to the MARKDOWN detector and invented link sites (review
/// 2026-08-20 #4).
#[test]
fn scan_only_files_are_neither_graphed_nor_md_fallback() {
    let dir = crate::testutil::scratch("graph-scanonly");
    let fixtures: [(&str, &[u8]); 2] = [
        ("style.css", b"/* [x](./style.css) */ a { color: red }\n"),
        ("app.js", b"// [doc](./app.js)\nconst x = 1;\n"),
    ];
    for (name, bytes) in fixtures {
        std::fs::write(dir.join(name), bytes).expect(name);
    }
    let files = analyze(&dir).expect("analyze");
    assert!(files.is_empty(), "no scan-only rows: {:?}", counts(&files));
    std::fs::remove_dir_all(&dir).ok();
}
