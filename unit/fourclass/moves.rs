//! Differential gate for the L1 judgment's move into the core (plan
//! v2.33 W3, user instruction 2026-10-03): seeded random file pairs
//! through (a) the frozen Rust oracle (unit/w3_oracle/fourclass.rs, the
//! 093aede4 code) and (b) this lowering over the real core wire, every
//! field equal — counts, moved lines with their units, relocated units,
//! the one-sided declarations, the changed lines, the leftover runs and
//! the duplicated spans. The generator walks the edges: empty sides,
//! single lines, all-equal lines (owner and sort ties), duplicate units,
//! blank bridges past MAX_BRIDGE, whitespace-only differences, shuffled
//! and reversed files; batches of several pairs share one key table.

use super::{Judged, read, request};
use crate::corelink::Link;
use crate::fourclass::{Classification, FourClass, MovedLine, batch::PairInput};
use crate::scan::lang::Lang;
use crate::w3_oracle::{Lcg, core_link, fourclass as oracle};

const RUST: [&str; 18] = [
    "fn a() {",
    "fn b(x: u8) {",
    "fn a() { 1 }",
    "}",
    "    let x = 1;",
    "    let y = 2;",
    "    x + y",
    "",
    "    ",
    "{",
    "impl A {",
    "    fn m(&self) {}",
    "pub fn a() {}",
    "const K: u8 = 1;",
    "    // note",
    "fn work(a: i32) -> i32 { a }",
    "    let x = 1;  ",
    "\tlet x = 1;",
];

const PYTHON: [&str; 11] = [
    "def f():",
    "def g(x):",
    "    return 1",
    "    x = compute()",
    "",
    "class A:",
    "    def add(self, x):",
    "        pass",
    "x = 1",
    "  x = 1",
    "def f():  ",
];

fn text(rng: &mut Lcg, vocab: &[&str], len: usize) -> Vec<String> {
    (0..len)
        .map(|_| vocab[rng.next(vocab.len())].to_string())
        .collect()
}

/// One generated pair: (before, after, lang), by case shape.
fn pair(rng: &mut Lcg, case: usize) -> (String, String, Lang) {
    let (vocab, lang): (&[&str], Lang) = if rng.next(2) == 0 {
        (&RUST, Lang::Rust)
    } else {
        (&PYTHON, Lang::Python)
    };
    let len = rng.next(41);
    let base = text(rng, vocab, len);
    let (before, after): (Vec<String>, Vec<String>) = match case % 10 {
        0 => (Vec::new(), base),
        1 => (base, Vec::new()),
        2 => (Vec::new(), Vec::new()),
        3 => {
            let line = vocab[rng.next(vocab.len())].to_string();
            (vec![line.clone(); len], vec![line; rng.next(41)])
        }
        4 => {
            let mut shuffled = base.clone();
            for i in (1..shuffled.len()).rev() {
                shuffled.swap(i, rng.next(i + 1));
            }
            (base, shuffled)
        }
        5 => {
            let mut edited = base.clone();
            for _ in 0..rng.next(6) {
                let at = rng.next(edited.len() + 1);
                match rng.next(3) {
                    0 if at < edited.len() => {
                        edited.remove(at);
                    }
                    1 if at < edited.len() => edited[at] = vocab[rng.next(vocab.len())].to_string(),
                    _ => edited.insert(at, vocab[rng.next(vocab.len())].to_string()),
                }
            }
            (base, edited)
        }
        6 => {
            let len = rng.next(41);
            (base, text(rng, vocab, len))
        }
        7 => (text(rng, vocab, 1), text(rng, vocab, 1)),
        8 => {
            // significant lines across blank bridges of 6 to 12 lines
            let mut bridged = Vec::new();
            for line in &base {
                bridged.push(line.clone());
                bridged.extend(std::iter::repeat_n(String::new(), 6 + rng.next(7)));
            }
            (Vec::new(), bridged)
        }
        _ => {
            let mut doubled = base.clone();
            doubled.extend(base.iter().cloned());
            let reversed: Vec<String> = base.iter().rev().cloned().collect();
            (reversed, doubled)
        }
    };
    (before.join("\n"), after.join("\n"), lang)
}

fn ask(link: &mut Link, inputs: &[PairInput]) -> Result<Judged, String> {
    let (req, bodies) = request(inputs);
    let mut replies = Vec::new();
    for body in bodies {
        replies.push(link.request(super::KIND, body)?);
    }
    read(req, &replies)
}

type Decls = Vec<(String, i64, usize, usize)>;
type Fields = (
    FourClass,
    Vec<MovedLine>,
    Vec<String>,
    (Decls, Decls),
    (Vec<usize>, Vec<usize>),
    bool,
);

/// Everything a Classification states, comparable.
fn fields(c: &Classification) -> Fields {
    let decl = |ds: &[crate::fourclass::decls::Decl]| -> Decls {
        ds.iter()
            .map(|d| (d.key.clone(), d.kind, d.start, d.end))
            .collect()
    };
    (
        c.counts.clone(),
        c.moved.clone(),
        c.relocated_units.clone(),
        (decl(&c.decls.0), decl(&c.decls.1)),
        (c.changed.removed.clone(), c.changed.added.clone()),
        c.degraded,
    )
}

#[test]
fn ten_thousand_seeded_pairs_answer_as_the_frozen_rust_did() {
    let mut link = core_link();
    let mut rng = Lcg(0x5EED_0004);
    let (mut case, mut compared) = (0usize, 0usize);
    // [a move, a relocated unit, a one-sided declaration, a duplicated
    // span, a side with two or more leftover runs]
    let mut seen = [0u32; 5];
    while compared < 10_000 {
        let texts: Vec<(String, String, Lang)> = (0..1 + rng.next(8))
            .map(|_| {
                case += 1;
                pair(&mut rng, case)
            })
            .collect();
        let inputs: Vec<PairInput> = texts
            .iter()
            .map(|(b, a, lang)| PairInput {
                before: b,
                after: a,
                lang: *lang,
            })
            .collect();
        let got = ask(&mut link, &inputs).expect("the core answers");
        for (i, (b, a, lang)) in texts.iter().enumerate() {
            let want = oracle::classify(b, a, *lang);
            let what = format!("case {case} pair {i} ({lang:?}): {b:?} -> {a:?}");
            assert_eq!(fields(&got.pairs[i]), fields(&want), "{what}");
            assert_eq!(got.runs[i], oracle::leftovers(b, a, &want), "runs, {what}");
            assert_eq!(
                got.dup_spans[i],
                oracle::dup_spans(b, a, *lang),
                "dup spans, {what}"
            );
            seen[0] += u32::from(!want.moved.is_empty());
            seen[1] += u32::from(!want.relocated_units.is_empty());
            seen[2] += u32::from(!want.decls.0.is_empty() || !want.decls.1.is_empty());
            seen[3] += u32::from(!got.dup_spans[i].is_empty());
            let (rem, add) = &got.runs[i];
            seen[4] += u32::from(rem.len() > 1 || add.len() > 1);
            compared += 1;
        }
    }
    assert!(seen.iter().all(|&c| c > 0), "an edge never met: {seen:?}");
}

/// The ceilings: consecutive pairs share a body while the running line
/// and unit counts stay within the core's ceilings (the body starts are
/// stated over numbers, no text), and a body over a ceiling is the
/// core's named degradation, which `read` returns as the reason — the
/// only place the core road differs from the frozen Rust, which had no
/// ceiling.
#[test]
fn the_ceilings_split_requests_and_the_core_names_a_body_over_one() {
    assert_eq!(super::packing(&[], 10, 10), Vec::<usize>::new());
    assert_eq!(
        super::packing(&[(6, 1), (4, 1), (1, 1)], 10, 10),
        vec![0, 2],
        "lines: 10 fits, 11 splits"
    );
    assert_eq!(
        super::packing(&[(1, 6), (1, 5)], 10, 10),
        vec![0, 1],
        "units: 11 splits"
    );
    assert_eq!(
        super::packing(&[(11, 1), (1, 1), (1, 1)], 10, 10),
        vec![0, 1],
        "one pair over the line cap alone"
    );
    let mut link = core_link();
    let cap = crate::tables::get().limits.moves.unit_cap;
    use serde_json::json;
    // one outer unit encloses the rest, so the top-level test stops at
    // the first unit it reads (a linear walk at the ceiling)
    let body = |n: usize| {
        let mut units = vec![json!([0, 0, 1, 2, 0])];
        units.extend(std::iter::repeat_n(json!([0, 0, 1, 1, 0]), n - 1));
        json!({"keys": [7], "pairs": [{
            "before": {"lines": [], "changed": [], "units": []},
            "after": {"lines": [[0, 7, 1], [0, 7, 1]], "changed": [0], "units": units},
        }]})
    };
    let at = link
        .request(super::KIND, body(cap))
        .expect("at the ceiling");
    assert_eq!(at["degraded"], json!(false), "at the ceiling is judged");
    let over = link
        .request(super::KIND, body(cap + 1))
        .expect("over the ceiling");
    assert_eq!(
        (&over["degraded"], &over["reason"]),
        (&json!(true), &json!("moves_too_large"))
    );
    let (req, _) = request(&[PairInput {
        before: "",
        after: "def f():\n    pass\n",
        lang: Lang::Python,
    }]);
    assert_eq!(read(req, &[over]).err().as_deref(), Some("moves_too_large"));
}
