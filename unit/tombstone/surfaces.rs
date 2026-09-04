use super::*;
use crate::tombstone::tests::pair;

#[test]
fn labels_are_the_headings_this_change_added() {
    let p = pair(
        "r.md",
        "# Dongpo Pork\n\ntext\n\n## Sides\n",
        "# Tomato and Egg (no Dongpo Pork)\n\ntext\n\n## Sides\n\n<!-- ## no dongpo -->\n",
        Lang::Markdown,
    );
    let added = added_lines(&p);
    assert!(added.contains(&1) && !added.contains(&5), "{added:?}");
    let got = labels(&p, &added);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].text, "Tomato and Egg (no Dongpo Pork)");
    assert_eq!((got[0].kind, got[0].line), (LabelKind::Heading, 1));
}

#[test]
fn code_labels_are_new_units_and_a_new_file_adds_its_stem() {
    let p = pair(
        "kitchen.rs",
        "fn braise_dongpo_pork() {}\nfn boil() {}\n",
        "fn cook_without_dongpo() {}\nfn boil() {}\n",
        Lang::Rust,
    );
    let got = labels(&p, &added_lines(&p));
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(
        (got[0].text.as_str(), got[0].kind, got[0].line),
        ("cook_without_dongpo", LabelKind::Unit, 1)
    );
    let fresh = pair("no_dongpo.rs", "", "fn boil() {}\n", Lang::Rust);
    let got: Vec<(String, LabelKind)> = labels(&fresh, &added_lines(&fresh))
        .into_iter()
        .map(|l| (l.text, l.kind))
        .collect();
    assert_eq!(
        got,
        [
            ("boil".to_string(), LabelKind::Unit),
            ("no_dongpo".to_string(), LabelKind::FileStem)
        ]
    );
}

/// The prose surface, read raw (backticks and comment markers stay)
/// and only on the lines this change added: a brand-new segment is
/// whole; a touched segment whose old lines carry the mark yields
/// just the new line, and the site is that line.
#[test]
fn prose_is_the_added_lines_of_each_touched_segment_read_raw() {
    let old = "/// Old: no longer braises.\n/// Second line.\n";
    let cases = [
        (
            "/// Braises pork.\nfn braise() {}\n".to_string(),
            "/// Braises pork.\nfn braise() {}\n\n/// This recipe no longer uses `dongpo_pork`.\nfn cook() {}\n".to_string(),
            (4, "/// This recipe no longer uses `dongpo_pork`."),
        ),
        (
            format!("{old}fn cook() {{}}\n"),
            format!("{old}/// Third uses `dongpo_pork`.\nfn cook() {{}}\n"),
            (3, "/// Third uses `dongpo_pork`."),
        ),
    ];
    for (before, after, want) in &cases {
        let p = pair("k.rs", before, after, Lang::Rust);
        let segs = prose(&p, &added_lines(&p));
        assert_eq!(segs.len(), 1, "{segs:?}");
        assert_eq!((segs[0].start, segs[0].text.as_str()), *want);
    }
}

#[test]
fn markdown_prose_skips_fenced_examples() {
    let p = pair(
        "r.md",
        "",
        "Intro line.\n\n```\nno longer dongpo\n```\n",
        Lang::Markdown,
    );
    let segs = prose(&p, &added_lines(&p));
    let texts: Vec<&str> = segs.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts, ["Intro line."]);
}
