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
    let added = added(&p).lines;
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
    let got = labels(&p, &added(&p).lines);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(
        (got[0].text.as_str(), got[0].kind, got[0].line),
        ("cook_without_dongpo", LabelKind::Unit, 1)
    );
    let fresh = pair("no_dongpo.rs", "", "fn boil() {}\n", Lang::Rust);
    let got: Vec<(String, LabelKind)> = labels(&fresh, &added(&fresh).lines)
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
/// and only the sentences this change wrote: a brand-new segment is
/// whole; a touched segment whose old lines carry the mark yields
/// just the new sentence, and the site is its added line.
#[test]
fn prose_is_the_sentences_this_change_wrote_read_raw() {
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
        let segs = prose(&p, &added(&p).lines);
        assert_eq!(segs.len(), 1, "{segs:?}");
        assert_eq!((segs[0].start, segs[0].text.as_str()), *want);
    }
}

/// Sentences are cut in the WHOLE segment (codex review 2026-09-04):
/// two added lines around an unchanged one that ends the sentence are
/// two sentences — joined alone they once read `We no longer` +
/// `Consult dongpo_pork.` as one sentence, mark and name together. A
/// Markdown paragraph, because docdup keeps every `///` line a segment
/// of its own (the node spans into the next row and never merges).
#[test]
fn a_sentence_boundary_on_an_unchanged_line_still_cuts() {
    let before = "Intro.\nuse the wok.\nOutro.\n";
    let after = "We no longer\nuse the wok.\nConsult dongpo_pork.\n";
    let p = pair("r.md", before, after, Lang::Markdown);
    let segs = prose(&p, &added(&p).lines);
    let got: Vec<(usize, usize, &str)> = segs
        .iter()
        .map(|s| (s.start, s.end, s.text.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            (1, 1, "We no longer use the wok."),
            (3, 3, "Consult dongpo_pork.")
        ]
    );
}

/// A commit message's one label is its subject line, wherever the
/// first non-blank line sits.
#[test]
fn the_subject_is_the_first_non_blank_line() {
    let got = subject("\n\nSides (no dongpo)\n\n- more\n");
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(
        (got[0].line, got[0].text.as_str(), got[0].kind),
        (3, "Sides (no dongpo)", LabelKind::Subject)
    );
    assert!(subject("\n\n").is_empty());
}

#[test]
fn markdown_prose_skips_fenced_examples() {
    let p = pair(
        "r.md",
        "",
        "Intro line.\n\n```\nno longer dongpo\n```\n",
        Lang::Markdown,
    );
    let segs = prose(&p, &added(&p).lines);
    let texts: Vec<&str> = segs.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(texts, ["Intro line."]);
}
