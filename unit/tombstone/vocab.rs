use super::*;

#[test]
fn the_vocabulary_is_every_table_and_every_mark_word() {
    let yes = [
        "no",
        "without",
        "free",
        "无",
        "已删",
        "the",
        "longer",
        "previously",
        "removed",
    ];
    let no = ["dongpo", "cache", "pork", "东坡肉", "user"];
    assert!(yes.iter().all(|w| vocabulary(w)), "{yes:?}");
    assert!(no.iter().all(|w| !vocabulary(w)), "{no:?}");
}
use std::collections::BTreeSet;

fn table(t: &'static [&'static str]) -> Vec<&'static str> {
    entries(t).collect()
}

fn unique(t: &[&str]) -> bool {
    t.iter().collect::<BTreeSet<_>>().len() == t.len()
}

#[test]
fn ascii_tables_are_lower_case_and_unique() {
    for t in [
        v().negations,
        v().keywords,
        v().en_prefix,
        v().en_suffix,
        v().marks_en,
    ]
    .map(table)
    {
        assert!(unique(&t), "{t:?}");
        let clean = t
            .iter()
            .all(|w| *w == w.to_lowercase() && *w == w.trim() && !w.is_empty());
        assert!(clean, "{t:?}");
    }
}

#[test]
fn wide_tables_carry_no_ascii() {
    for t in [v().zh_prefix, v().zh_suffix, v().marks_zh].map(table) {
        assert!(unique(&t), "{t:?}");
        let wide = t
            .iter()
            .all(|w| !w.is_empty() && w.chars().all(|c| !c.is_ascii()));
        assert!(wide, "{t:?}");
    }
}

#[test]
fn keywords_are_sorted_single_words() {
    let k = table(v().keywords);
    assert!(
        k.windows(2).all(|p| p[0] < p[1]),
        "sorted, so a reader can find a word"
    );
    assert!(k.iter().all(|w| w.bytes().all(|b| b.is_ascii_lowercase())));
}

/// V₀ is the absence vocabulary, not a keyword list: `false` and
/// `none` live there alone, and every English prefix but the two
/// loanwords is an absence word.
#[test]
fn absence_words_and_reserved_words_are_disjoint() {
    assert!(entries(v().negations).all(|n| !has(v().keywords, n)));
    assert!(entries(v().en_prefix).all(|p| has(v().negations, p) || matches!(p, "sans" | "minus")));
}

#[test]
fn a_table_holds_a_word_exactly_and_the_floors_nest() {
    assert!(
        has(v().en_suffix, "free") && !has(v().en_suffix, "fre") && !has(v().en_suffix, "freed")
    );
    assert_eq!(v().open.len(), v().close.len());
    assert!(
        v().join_max >= 2,
        "a two-word name like dongpo_pork must fit one window"
    );
    assert!(v().min_wide_name < v().min_ascii_name);
    assert_eq!(TOMBSTONE_REV, 1);
}
