//! The L1 judgment over a real core link (plan v2.33 W3, `moves/1`):
//! the move rule, the unit attribution, the intact-relocation summary,
//! the one-sided declarations and the duplicated top-level spans — the
//! cases unit/fourclass/{model,decls,stacking}.rs pinned while the
//! judgment was Rust's, now asked of the core through the lowering
//! (fourclass::moves) every production caller uses.

use crate::common::core_bin;
use codeeraser::corelink::Link;
use codeeraser::dedup::tokens::fnv1a;
use codeeraser::fourclass::batch::PairInput;
use codeeraser::fourclass::moves::{self, Judged};
use codeeraser::scan::lang::Lang;

/// The lowering's request, the core's reply, read back.
pub fn judged(link: &mut Link, inputs: &[PairInput]) -> Judged {
    let (req, bodies) = moves::request(inputs);
    let replies: Vec<_> = bodies
        .into_iter()
        .map(|b| link.request(moves::KIND, b).expect("moves reply"))
        .collect();
    moves::read(req, &replies).expect("replies that read back")
}

fn one(before: &str, after: &str, lang: Lang) -> Judged {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    judged(
        &mut link,
        &[PairInput {
            before,
            after,
            lang,
        }],
    )
}

#[test]
fn moves_are_significant_content_matches_attributed_to_their_unit() {
    let c = &one(
        "def f():\n    x = compute()\n    return x\n\ndef g():\n    pass\n",
        "def f():\n    if ok:\n        x = compute()\n    return x\n\ndef g():\n    pass\n",
        Lang::Python,
    )
    .pairs[0];
    // [added novel, added moved, removed deleted, removed moved]: the
    // indented re-add moved, the new `if ok:` is novel
    let counts = &c.counts;
    let got = [
        counts.added_novel,
        counts.added_moved,
        counts.removed_deleted,
        counts.removed_moved,
    ];
    assert_eq!(got, [1, 1, 0, 1], "indent change is a move");
    let m = c.moved.iter().find(|m| !m.removed).unwrap();
    assert_eq!(m.unit.as_deref(), Some("f/0"));
    let blank = &one("a = 1\n\nb = 2\n", "b = 2\n\nc = 3\n", Lang::Python).pairs[0];
    assert_eq!(
        blank.counts.added_moved + blank.counts.removed_moved,
        0,
        "blank lines never move"
    );
    let swap = &one(
        "def a():\n    return 1\n\ndef b():\n    return 2\n",
        "def b():\n    return 2\n\ndef a():\n    return 1\n",
        Lang::Python,
    )
    .pairs[0];
    assert!(
        ["a/0", "b/0"]
            .iter()
            .any(|k| swap.relocated_units.iter().any(|r| r == k)),
        "a swapped function is relocated intact, got {:?}",
        swap.relocated_units
    );
    // the swap repositions one BLANK separator: blanks never move
    let counts = &swap.counts;
    let got = [counts.added_novel, counts.added_moved, counts.removed_moved];
    assert_eq!(got, [1, 2, 2]);
}

/// Before ==== after ==== vanished keys ==== appeared keys, one case
/// per `@@` line (`|`-separated keys): a leaving function, an
/// arriving const, visibility is not identity, arity is, and a key
/// twice on its side is no candidate at all.
const DECLS: &str = "fn a() {}\nfn b() {}\n====fn b() {}\n====a/0====
@@fn b() {}\n====const K: u8 = 1;\nfn b() {}\n========K
@@fn a() {}\n====pub fn a() {}\n========
@@fn a() {}\n====fn a(x: u8) -> u8 { x }\n====a/0====a/1
@@impl A {\n    fn m(&self) {}\n}\nimpl B {\n    fn m(&self) {}\n}\n========impl A|impl B====";

#[test]
fn declarations_are_one_sided_and_multiplicity_one() {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    for case in DECLS.split("\n@@") {
        let fields: Vec<&str> = case.split("====").collect();
        let [before, after, gone, arrived] = fields[..] else {
            panic!("four fields: {case}")
        };
        let input = PairInput {
            before,
            after,
            lang: Lang::Rust,
        };
        let c = &judged(&mut link, &[input]).pairs[0];
        let keys = |ds: &[codeeraser::fourclass::decls::Decl]| {
            ds.iter()
                .map(|d| d.key.clone())
                .collect::<Vec<_>>()
                .join("|")
        };
        assert_eq!(
            (keys(&c.decls.0), keys(&c.decls.1)),
            (gone.to_string(), arrived.to_string()),
            "{before:?} -> {after:?}"
        );
    }
}

/// The green direction: a new top-level duplicate rides every
/// occurrence with its span; a duplicate already there is not new.
/// The red direction, one row per measured or attack-reviewed
/// false-positive shape (cross-class methods, anonymous closures,
/// cross-impl methods, impl containers, cross-receiver Go methods).
#[test]
fn dup_spans_are_new_top_level_named_duplicates() {
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let work = "fn work(a: i32) -> i32 { a }\nfn work(a: i32) -> i32 { a + 1 }\n";
    let grown = format!("fn one() {{}}\n{work}");
    let mut spans = |before: &str, after: &str, lang| {
        let j = judged(
            &mut link,
            &[PairInput {
                before,
                after,
                lang,
            }],
        );
        j.dup_spans[0].clone()
    };
    let hash = fnv1a(b"work/1");
    assert_eq!(
        spans("fn one() {}\n", &grown, Lang::Rust),
        vec![[hash, 2, 2], [hash, 3, 3]]
    );
    assert!(
        spans(work, work, Lang::Rust).is_empty(),
        "already there is not NEW"
    );
    let none = [
        (
            "class A:\n    def add(self, x):\n        pass\n\nclass B:\n    def add(self, x):\n        pass\n",
            Lang::Python,
        ),
        (
            "fn go() {\n    let a = |x: i32| x;\n    let b = |x: i32| x + 1;\n}\n",
            Lang::Rust,
        ),
        (
            "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n",
            Lang::Rust,
        ),
        (
            "impl Foo {\n    fn go(&self) {}\n}\nimpl Advisor for Foo {\n    fn advise(&self) {}\n}\n",
            Lang::Rust,
        ),
        (
            "func (t T) add(x int) {}\nfunc (u U) add(x int) {}\n",
            Lang::Go,
        ),
    ];
    for (after, lang) in none {
        assert_eq!(
            spans("", after, lang),
            Vec::<[u64; 3]>::new(),
            "{after:?} is not evidence"
        );
    }
}
