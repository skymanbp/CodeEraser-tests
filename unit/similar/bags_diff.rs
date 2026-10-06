//! The differential gate of bags/1 (plan v2.33 W6): the frozen term road
//! (frozen.rs: cli/src/similar/{stem,terms,bag,docs}.rs at 1324c927) and
//! the core's (CE.Similar.Bags over the wire) must give the same bag for
//! the same input, term for term, channel for channel, count for count.
//! Four legs: every scalar value's character classes and lowercase; free
//! texts (`text_terms`); generated source files of six languages, file by
//! file through `file_bags` on both sides; and (ignored, driven by
//! CE_BAGS_DIFF_TREES) every code file of real trees. CE_BAGS_DIFF_SEED
//! picks the stream, CE_BAGS_DIFF_CASES the size (default 10,000).

use super::*;
use crate::scan::lang::Lang;
use crate::similar::{bag::file_bags, frozen};
use draw::Draw;
use std::path::Path;

#[path = "bags_gen.rs"]
mod draw;
#[path = "bags_src.rs"]
mod src;

/// A number the run is driven by: the environment's, else the default.
fn knob(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn cases() -> usize {
    knob("CE_BAGS_DIFF_CASES", 10_000) as usize
}

/// The leg's stream: CE_BAGS_DIFF_SEED mixed with the leg's salt.
fn stream(salt: u64) -> Draw {
    Draw::new(knob("CE_BAGS_DIFF_SEED", 20_261_006), salt)
}

/// What a leg compared, and where the two roads first parted.
#[derive(Default)]
struct Tally {
    cases: usize,
    units: usize,
    terms: usize,
    mismatches: usize,
}

impl Tally {
    fn agree<T: PartialEq + std::fmt::Debug>(&mut self, what: &str, want: T, got: T) {
        self.cases += 1;
        if want != got {
            self.mismatches += 1;
            if self.mismatches <= 5 {
                eprintln!("bags-diff MISMATCH {what}\n  frozen {want:?}\n  core   {got:?}");
            }
        }
    }

    fn report(&self, leg: &str) {
        println!(
            "bags-diff {leg}: cases {} units {} terms {} mismatches {}",
            self.cases, self.units, self.terms, self.mismatches
        );
        assert_eq!(self.mismatches, 0, "{leg}");
    }
}

/// A frozen bag under the live channels.
fn live(terms: &BTreeMap<u64, (frozen::terms::Channel, u32)>) -> Terms {
    let ch = |c: &frozen::terms::Channel| Channel::ALL[c.index()];
    terms.iter().map(|(t, (c, n))| (*t, (ch(c), *n))).collect()
}

/// `text_terms` of cli/src/similar/query.rs as it stood at 1324c927, up
/// to the bag it hands `query_of`, on the frozen road.
fn frozen_text(text: &str) -> Terms {
    use frozen::terms::{Channel as F, prose_words, word_term};
    let mut bag = BTreeMap::new();
    for w in prose_words(text) {
        for ch in [F::Name, F::Doc] {
            bag.entry(word_term(ch, &w)).or_insert((ch, 0)).1 += 1;
        }
    }
    live(&bag)
}

#[test]
fn every_scalar_value_reads_the_same_classes() {
    let all: Vec<u32> = (0..=0x10_FFFF)
        .filter(|u| char::from_u32(*u).is_some())
        .collect();
    let mut tally = Tally::default();
    for batch in all.chunks(65_536) {
        let reply = request(serde_json::json!({ "inspect": batch })).expect("bags/1");
        let rows: Vec<(u8, u8, u8, u8, Vec<u32>)> = judged::table(&reply, "chars").expect("chars");
        assert_eq!(rows.len(), batch.len(), "one row per code point");
        for (u, got) in batch.iter().zip(rows) {
            let c = char::from_u32(*u).expect("scalar");
            let lower: Vec<u32> = c.to_lowercase().map(u32::from).collect();
            let want = (
                u8::from(c.is_alphanumeric()),
                u8::from(c.is_numeric()),
                u8::from(c.is_lowercase()),
                u8::from(c.is_uppercase()),
                lower,
            );
            tally.agree(&format!("U+{u:04X}"), want, got);
        }
    }
    tally.report("chars");
}

#[test]
fn texts_bag_the_same() {
    let mut d = stream(1);
    let texts: Vec<String> = (0..cases())
        .map(|_| {
            let n = d.below(40);
            draw::prose(&mut d, n)
        })
        .collect();
    let mut tally = Tally::default();
    for batch in texts.chunks(500) {
        let refs: Vec<&str> = batch.iter().map(String::as_str).collect();
        let (_, got) = ask(Vec::new(), &refs).expect("bags/1");
        for (text, got) in batch.iter().zip(got) {
            let want = frozen_text(text);
            tally.terms += want.len();
            tally.agree(&format!("text {text:?}"), want, got);
        }
    }
    tally.report("texts");
}

/// One file on both roads: the same units, spans and bags.
fn compare(text: &str, lang: Lang, what: &str, tally: &mut Tally) {
    let want = frozen::bag::file_bags(text, lang);
    let got = file_bags(text, lang).expect("bags/1");
    tally.units += want.len();
    tally.terms += want.iter().map(|b| b.terms.len()).sum::<usize>();
    let spell = |key: &str, nth, s, e, t| (key.to_string(), nth, s, e, t);
    let want: Vec<_> = want
        .iter()
        .map(|b| spell(&b.key, b.nth, b.start_line, b.end_line, live(&b.terms)))
        .collect();
    let got: Vec<_> = got
        .into_iter()
        .map(|b| spell(&b.key, b.nth, b.start_line, b.end_line, b.terms))
        .collect();
    tally.agree(what, want, got);
}

#[test]
fn generated_sources_bag_the_same() {
    let mut d = stream(2);
    let mut tally = Tally::default();
    let mut i = 0;
    while tally.units < cases() {
        let lang = src::LANGS[i % src::LANGS.len()];
        let text = src::source(&mut d, lang, 10);
        compare(
            &text,
            lang,
            &format!("{} source\n{text}", lang.name()),
            &mut tally,
        );
        i += 1;
    }
    tally.report("sources");
}

/// Every file of a tree, depth first, skipping dot directories and build
/// output.
fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() && !name.starts_with('.') && name != "target" && name != "node_modules" {
            files(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

#[test]
#[ignore = "real trees: CE_BAGS_DIFF_TREES=<root>;<root>…"]
fn real_trees_bag_the_same() {
    let roots = std::env::var("CE_BAGS_DIFF_TREES").expect("CE_BAGS_DIFF_TREES");
    for root in roots.split(';').filter(|r| !r.is_empty()) {
        let mut all = Vec::new();
        files(Path::new(root), &mut all);
        all.sort();
        let mut tally = Tally::default();
        for path in all {
            let Some(lang) = Lang::from_path(&path) else {
                continue;
            };
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            compare(&text, lang, &path.display().to_string(), &mut tally);
        }
        tally.report(&format!("real {root}"));
    }
}
