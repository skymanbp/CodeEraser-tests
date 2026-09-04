use super::*;

#[test]
fn sentences_cut_after_terminators_but_not_inside_identifiers() {
    let text = "Edit ce.toml first; then run a.rs. 不再需要东坡肉。Done? yes! last";
    assert_eq!(
        sentences(text),
        [
            "Edit ce.toml first;",
            "then run a.rs.",
            "不再需要东坡肉。",
            "Done?",
            "yes!",
            "last"
        ]
    );
}

fn ascii(s: &str) -> Word {
    Word::Ascii(s.to_string())
}

fn wide(s: &str) -> Word {
    Word::Wide(s.to_string())
}

/// The spellings a surface's frames bind, with the bracket flag.
fn bound(surface: &str) -> Vec<(String, bool)> {
    label_candidates(&words(surface))
        .into_iter()
        .map(|c| (c.span.text, c.bracketed))
        .collect()
}

fn one(text: &str, bracketed: bool) -> Vec<(String, bool)> {
    vec![(text.to_string(), bracketed)]
}

#[test]
fn the_cut_splits_snake_camel_brackets_and_scripts() {
    assert_eq!(
        words("braise_dongpo_pork"),
        [ascii("braise"), ascii("dongpo"), ascii("pork")]
    );
    assert_eq!(
        words("DongpoPork v2"),
        [ascii("dongpo"), ascii("pork"), ascii("v2")]
    );
    assert_eq!(words("HTTPServer"), [ascii("httpserver")]);
    assert_eq!(
        words("Tomato (no Dongpo)"),
        [
            ascii("tomato"),
            Word::Open,
            ascii("no"),
            ascii("dongpo"),
            Word::Close
        ]
    );
    assert_eq!(
        words("番茄炒蛋（无东坡肉）"),
        [wide("番茄炒蛋"), Word::Open, wide("无东坡肉"), Word::Close]
    );
    assert_eq!(words("无cache"), [wide("无"), ascii("cache")]);
    assert!(words("").is_empty());
}

#[test]
fn windows_stop_at_a_bracket_or_the_end() {
    let texts: Vec<String> = windows(&words("(no dongpo) pork"))
        .into_iter()
        .map(|s| s.text)
        .collect();
    assert_eq!(texts, ["no", "no_dongpo", "dongpo", "pork"]);
    let last = windows(&words("a b c d")).into_iter().last().unwrap();
    assert_eq!((last.at, last.len, last.text.as_str()), (3, 1, "d"));
}

#[test]
fn an_english_prefix_binds_the_words_after_it() {
    assert_eq!(
        bound("Tomato and Egg (no Dongpo Pork)"),
        [
            ("dongpo".to_string(), true),
            ("dongpo_pork".to_string(), true)
        ]
    );
    assert_eq!(bound("cook_without_dongpo"), one("dongpo", false));
    assert_eq!(bound("no more dongpo"), one("dongpo", false));
    // a prefix is a whole word; a frame with nothing in its slot binds nothing
    let empty: Vec<usize> = ["nobody knows", "no"]
        .iter()
        .map(|s| bound(s).len())
        .collect();
    assert_eq!(empty, [0, 0]);
}

#[test]
fn an_english_suffix_binds_the_words_before_it() {
    assert_eq!(bound("lock free queue"), one("lock", false));
    assert_eq!(
        bound("dongpo pork removed"),
        [
            ("pork".to_string(), false),
            ("dongpo_pork".to_string(), false)
        ]
    );
}

#[test]
fn chinese_frames_live_inside_one_run() {
    assert_eq!(bound("番茄炒蛋（无东坡肉）"), one("东坡肉", true));
    assert_eq!(bound("东坡肉已移除"), one("东坡肉", false));
    assert_eq!(bound("无cache"), one("cache", false));
    assert!(bound("无").is_empty());
}

#[test]
fn a_mark_is_a_whole_phrase_at_word_boundaries() {
    let cases = [
        ("This recipe no longer braises pork.", true),
        ("此前由 braise 负责，现已删除", true),
        ("We removed it deliberately.", true),
        ("the previously_seen set", false), // an identifier is not a mark
        ("nothing to see here", false),
    ];
    for (text, expect) in cases {
        assert_eq!(has_mark(text), expect, "{text}");
    }
}

#[test]
fn frame_words_come_from_every_table() {
    let verdicts: Vec<bool> = ["without", "free", "无", "已移除", "cache"]
        .iter()
        .map(|w| frame_word(w))
        .collect();
    assert_eq!(verdicts, [true, true, true, true, false]);
}
