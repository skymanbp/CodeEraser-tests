use super::*;
use crate::tombstone::surfaces::added_lines;
use crate::tombstone::tests::pair;

/// R of a changeset, the added sets read the way the hub reads them.
fn r(pairs: &[PairText]) -> Erased {
    let added: Vec<_> = pairs.iter().map(added_lines).collect();
    erased(pairs, &added)
}

/// R of one Markdown document rewritten from `before` to `after`.
fn md(before: &str, after: &str) -> Erased {
    r(&[pair("r.md", before, after, Lang::Markdown)])
}

/// The names of one text must hold every `present` spelling and no
/// `absent` one.
fn expect(text: &str, lang: Lang, present: &[&str], absent: &[&str]) {
    let names = names_of(text, lang);
    let got: Vec<&str> = names.iter().map(|n| n.text.as_str()).collect();
    let missing: Vec<&&str> = present.iter().filter(|n| !got.contains(n)).collect();
    let extra: Vec<&&str> = absent.iter().filter(|n| got.contains(n)).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "missing {missing:?}, unwanted {extra:?} in {got:?}"
    );
}

#[test]
fn one_key_for_every_spelling_of_a_name() {
    assert_eq!(canon("DongpoPork"), "dongpo_pork");
    assert_eq!(key("DongpoPork"), key("dongpo_pork"));
    assert_eq!(key("Dongpo Pork"), key("dongpo-pork"));
    assert_ne!(key("dongpo"), key("dongpo_pork"));
    assert_eq!(canon("东坡肉 recipe"), "东坡肉_recipe");
}

#[test]
fn the_floor_admits_names_not_absence_or_reserved_words() {
    let yes = ["abc", "user_data", "东坡"];
    // too short / one wide char / no letter / a reserved word alone /
    // a frame word inside / an absence word inside
    let no = ["ab", "无", "123", "data", "no_cache", "is_empty"];
    assert!(yes.iter().all(|s| admitted(s)), "{yes:?}");
    let leaked: Vec<&&str> = no.iter().filter(|s| admitted(s)).collect();
    assert!(leaked.is_empty(), "{leaked:?}");
}

#[test]
fn code_names_are_identifiers_and_units_never_comment_or_literal_words() {
    expect(
        "// the downtime window\n#[cfg(target_os = \"linux\")]\nfn braise_dongpo_pork(heat: u32) {\n    let msg = \"an independent audit\";\n}\n",
        Lang::Rust,
        &[
            "braise_dongpo_pork",
            "dongpo_pork",
            "dongpo",
            "heat",
            "msg",
            "target_os",
        ],
        &[
            "downtime",
            "fn",
            "fn_braise",
            "linux",
            "independent",
            "audit",
        ],
    );
    expect(
        "greet = 'hello there'\ndef braise(): return \"pork belly\"\n",
        Lang::Python,
        &["greet", "braise"],
        &["hello", "there", "pork_belly", "belly"],
    );
    expect(
        "module Kitchen where\nlabel = \"dongpo pork\"\n",
        Lang::Haskell,
        &["kitchen", "label"],
        &["dongpo", "pork", "dongpo_pork"],
    );
}

#[test]
fn markdown_names_are_headings_and_list_leads_a_span_only_keeps_alive() {
    expect(
        "# Dongpo Pork\n\nBraise the belly slowly with `soy_sauce` until done.\n\n\
         - Shaoxing wine\n\n```\nexample_only()\n```\n",
        Lang::Markdown,
        &["dongpo_pork", "dongpo", "shaoxing"],
        &["soy_sauce", "belly", "slowly", "example_only", "wine"],
    );
    // the plan banner: one long line rewritten in place re-mentions
    // its own spans — nothing was declared, nothing was removed
    let e = md(
        "# Plan\n\n> v1 used `lang_scan`; v2 不再 uses `i18n`.\n",
        "# Plan\n\n> v1 used `lang_scan`; v2 不再 uses `i18n`; v3 adds more.\n",
    );
    assert!(e.names.is_empty(), "{:?}", e.names);
}

#[test]
fn a_move_is_not_an_erasure() {
    let moved = [
        pair("a.rs", "fn braise_dongpo_pork() {}\n", "", Lang::Rust),
        pair("b.rs", "", "fn braise_dongpo_pork() {}\n", Lang::Rust),
    ];
    assert!(r(&moved).names.is_empty());
}

#[test]
fn a_name_that_recurs_only_inside_a_frame_is_erased() {
    let e = md("# Dongpo Pork\n", "# Tomato and Egg (no Dongpo Pork)\n");
    assert!(
        e.has(key("dongpo")) && e.has(key("dongpo_pork")),
        "{:?}",
        e.names
    );
    assert!(
        e.has(key("pork")),
        "every word under the frame is erased with the phrase"
    );
}

#[test]
fn prose_this_change_wrote_is_not_survival_but_an_untouched_line_is() {
    let narrated = md("# Dongpo Pork\n", "Dongpo Pork is gone from the menu.\n");
    assert!(narrated.has(key("dongpo_pork")), "a paragraph narrates");
    let kept = md(
        "# Dongpo Pork\n\nUse `dongpo_pork`.\n",
        "# Menu\n\nUse `dongpo_pork`.\n",
    );
    assert!(kept.names.is_empty(), "the untouched code span survives");
    let written = md("# Dongpo Pork\n", "# Menu\n\nUse `dongpo_pork`.\n");
    assert!(
        written.has(key("dongpo_pork")),
        "a code span this change wrote does not"
    );
}

#[test]
fn a_framed_window_is_no_name_on_either_side() {
    // the requests replay: `def test_header_no_return_chars` moving
    // one line read as "erased `return_chars`, wrote it back framed"
    expect(
        "def test_header_no_return_chars():\n    pass\n",
        Lang::Python,
        &["test_header", "header"],
        &["return_chars", "chars", "no_return_chars"],
    );
    let e = md("## Menu (no dongpo pork)\n", "## Sides (no dongpo pork)\n");
    assert_eq!(e.names.len(), 1, "only `menu` went away: {:?}", e.names);
}

#[test]
fn vocabulary_words_and_stop_word_windows_are_no_names() {
    // the self replay: `longer` from a removed `## No longer supported`
    // bound every later `no longer`; `the_pre` / `budget_is` are cuts
    // through sentence-shaped identifiers
    expect(
        "fn the_pre_budget_is_here() {}\n",
        Lang::Rust,
        &["pre_budget", "pre", "budget"],
        &["the_pre", "budget_is", "is_here", "here", "longer"],
    );
    expect(
        "fn removed_items() {}\n",
        Lang::Rust,
        &["items"],
        &["removed", "removed_items"],
    );
    expect(
        "## Previously supported\n",
        Lang::Markdown,
        &["supported"],
        &["previously"],
    );
}

#[test]
fn a_wide_name_survives_only_as_a_whole_word() {
    let e = md("# 东坡肉\n", "# 番茄炒蛋（无东坡肉）\n");
    assert!(e.has(key("东坡肉")), "无东坡肉 is another word");
    assert_eq!(
        (e.wide_in("东坡肉做法的段落"), e.wide_in("番茄炒蛋")),
        (Some("东坡肉"), None)
    );
    let longer = md("# 东坡肉\n", "# 东坡肉做法\n");
    assert!(
        longer.has(key("东坡肉")),
        "a superstring is not the word: the segmentation limit, on the over-counting side"
    );
}

#[test]
fn spelled_in_reads_every_window_and_a_long_run() {
    let text = "uses parse_http_header_line here";
    let found = |name: &str| spelled_in(text, |k| k == key(name));
    assert_eq!(
        found("parse_http_header_line").as_deref(),
        Some("parse_http_header_line"),
        "four words: beyond a window, its own spelling"
    );
    assert_eq!(found("uses_parse").as_deref(), Some("uses_parse"));
    assert_eq!(found("header_line").as_deref(), Some("header_line"));
    assert_eq!(found("header_here"), None, "windows are adjacent words");
}
