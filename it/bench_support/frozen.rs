//! The frozen evaluation points' value column, in both languages.
//!
//! English is the sealed ledger's own wording: `contracts/bench/bench.json`
//! owns it and nothing here re-types a word of it. Chinese is one
//! template per metric, spliced with the SAME numbers read back out of
//! that English value — a Chinese sentence carrying digits of its own
//! would be a second owner of a frozen fact, free to drift from the
//! ledger the English column quotes.
//!
//! They had already drifted the other way, in nobody's favour: every
//! Chinese surface printed the English value verbatim —
//! `17/17 scoped (100%)`, `0/600 flagged (gate <= 1%)`,
//! `overall gate >= 0.90 held` — through every release. The language
//! gate cuts generated blocks whole, their language belonging to their
//! renderer, and no renderer had ever been given a language.
//! `docs_lang_generated` reads that half now.
//!
//! ONE table keyed by metric, never one renderer per language: two
//! parallel per-language functions of the same shape are a single
//! clone block in the dedup gate's eyes, and the numbers would then be
//! written twice.

use super::render::s;
use serde_json::Value;

/// Rendered where a template asks for a number the ledger no longer
/// states; `zh_value` sees it in the check below and publishes the
/// English instead of a sentence with a hole in it.
const MISSING: &str = "{?}";

/// metric `|` the Chinese sentence, one per line, `{i}` standing for
/// the i-th number of the English value. Chinese punctuation
/// throughout (full-width parentheses and comma, no space against a
/// CJK character); what a reader looks up — the corpora `cobra` and
/// `zod`, every metric identifier — stays as the ledger spells it.
///
/// One string rather than a table of pairs: a run of two-string tuples
/// rhymes with every other constant table in the suite once the dedup
/// tokenizer has normalized the identifiers and literals away (it
/// landed as a clone of `unit/structure/tree.rs`'s vocabulary probe),
/// and the clone budget only ever goes down.
const ZH: &str = "\
docdup_d3_precision|范围内 {0}/{1}（{2}%）
docdup_d1_recall|{0}%
t3_precision|已判 {0}，误判 {1}（{2}）
graph_precision|整体门 ≥ {0} 已满足
fourclass_fpr|{1} 个样本命中 {0} 个（门 ≤ {2}%）
guard_fpr_per500|每 {1} 次编辑 {0} 次误报
l2_moved_recall|跨文件搬迁行 {0}/{1}
dedup_recall_vs_jscpd|cobra 原始 {0}/{1} → 归因后 {2}/{3}
t3_recall_vs_similarity|zod {0} / requests {1} / cobra {2}（原始）
";

/// The Chinese sentence a metric is written with, if it has one.
fn pattern_of(metric: &str) -> Option<&'static str> {
    ZH.lines()
        .filter_map(|row| row.split_once('|'))
        .find(|(named, _)| *named == metric)
        .map(|(_, pattern)| pattern)
}

/// Every number a text states, in order. A dotted run stays ONE token,
/// so `0.5.0` is a comparator's version and not three readings.
pub fn numbers(text: &str) -> Vec<&str> {
    let b = text.as_bytes();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < b.len() {
        if !b[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len()
            && (b[i].is_ascii_digit()
                || (b[i] == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit)))
        {
            i += 1;
        }
        out.push(&text[start..i]);
    }
    out
}

/// `{i}` → `nums[i]`; a slot the ledger no longer fills renders as
/// `MISSING`, which the numeric check then refuses.
fn fill(pattern: &str, nums: &[&str]) -> String {
    let (mut out, mut rest) = (String::new(), pattern);
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}').expect("an unclosed {index} slot");
        let slot: usize = rest[open + 1..close].parse().expect("a {index} slot");
        out.push_str(nums.get(slot).copied().unwrap_or(MISSING));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// The same numbers, in whatever order the language puts them: word
/// order inside a sentence belongs to the language, the facts do not.
fn same_numbers(rendered: &str, ledger: &str) -> bool {
    let sorted = |t: &str| {
        let mut n = numbers(t);
        n.sort_unstable();
        n.join(" ")
    };
    sorted(rendered) == sorted(ledger)
}

/// The Chinese sentence for a frozen point, or `None` when this build
/// has no template for that metric, or the ledger has been reworded
/// past the one it has. Either way the English is published — and the
/// language gate reddens — rather than a stale translation shipping
/// quietly under a Chinese heading.
pub fn zh_value(point: &Value) -> Option<String> {
    let ledger = s(point, "value");
    let rendered = fill(pattern_of(s(point, "metric"))?, &numbers(ledger));
    same_numbers(&rendered, ledger).then_some(rendered)
}

/// A frozen point's value column for one language.
pub fn value(point: &Value, zh: bool) -> String {
    match zh.then(|| zh_value(point)).flatten() {
        Some(chinese) => chinese,
        None => s(point, "value").to_string(),
    }
}
