use super::*;
use crate::scan::lang::Lang;

/// Red/green for the stacking evidence. The green direction: a
/// genuine top-level duplicate fires, and every after-side occurrence
/// rides as its own `[hash, start, end]` row (7.0.0) so the core can
/// place the novel mass. The red direction, one row per measured or
/// attack-reviewed false-positive shape: methods under different
/// Python classes, anonymous Rust closures (fpr-fourclass 8/600 before
/// the top-level scoping), methods under different Rust impls, impl
/// containers themselves (inherent + trait impl of one type — the FPR
/// replay caught the unqualified key colliding), and Go methods on
/// different receivers (attack review F7).
#[test]
fn dup_evidence_is_top_level_and_named() {
    let real = PairInput {
        before: "fn one() {}\n",
        after: "fn one() {}\nfn work(a: i32) -> i32 { a }\nfn work(a: i32) -> i32 { a + 1 }\n",
        lang: Lang::Rust,
    };
    let hash = fnv1a(b"work/1");
    assert_eq!(
        dup_spans(&real),
        vec![[hash, 2, 2], [hash, 3, 3]],
        "both occurrences of the duplicated key, with their spans"
    );
    let grown = PairInput {
        before: "fn work(a: i32) -> i32 { a }\nfn work(a: i32) -> i32 { a + 1 }\n",
        after: "fn work(a: i32) -> i32 { a }\nfn work(a: i32) -> i32 { a + 1 }\n",
        lang: Lang::Rust,
    };
    assert!(
        dup_spans(&grown).is_empty(),
        "a duplicate that was already there is not NEW"
    );
    let none: [(&str, Lang, &str); 5] = [
        (
            "class A:\n    def add(self, x):\n        pass\n\nclass B:\n    def add(self, x):\n        pass\n",
            Lang::Python,
            "cross-class methods",
        ),
        (
            "fn go() {\n    let a = |x: i32| x;\n    let b = |x: i32| x + 1;\n}\n",
            Lang::Rust,
            "anonymous closures",
        ),
        (
            "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n",
            Lang::Rust,
            "cross-impl methods",
        ),
        (
            "impl Foo {\n    fn go(&self) {}\n}\nimpl Advisor for Foo {\n    fn advise(&self) {}\n}\n",
            Lang::Rust,
            "impl containers",
        ),
        (
            "func (t T) add(x int) {}\nfunc (u U) add(x int) {}\n",
            Lang::Go,
            "cross-receiver Go methods",
        ),
    ];
    for (after, lang, what) in none {
        let input = PairInput {
            before: "",
            after,
            lang,
        };
        assert_eq!(
            dup_spans(&input),
            Vec::<[u64; 3]>::new(),
            "{what} are not evidence"
        );
    }
}
