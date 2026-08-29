use super::mention_name;

/// One case per line: `path key ⇒ name`, or `⇒ -` for out of
/// domain — K24's extraction half (the frozen out-of-domain
/// spellings of §3.1/§2) beside the shapes that stay in. The keys
/// are the producer's own spellings (`(anonymous)/0` is what a TS
/// anonymous default export is keyed); the producer-measured
/// witnesses of the same exits are in mention::conv::tests.
#[test]
fn the_domain_is_exactly_the_single_token_names() {
    for case in [
        "a.rs open/0 ⇒ open",
        "a.rs CLASSES ⇒ CLASSES",
        "a.rs cellar ⇒ cellar",
        "a.rs impl A ⇒ -",
        "a.rs impl Show for A ⇒ -",
        "a.ts (anonymous)/0 ⇒ -",
        "a.rs r#type/0 ⇒ -",
        "a.rs a/b/2 ⇒ -",
        "a.py x/ ⇒ -",
        "a.go (T) add/1 ⇒ add",
        "a.go (*pkg.Cache[K, V]) M/0 ⇒ M",
        "a.go free/1 ⇒ free",
        "a.py __init__/1 ⇒ -",
        "a.py __main/0 ⇒ __main",
        "a.py public_call/0 ⇒ public_call",
        "a.ts $ZodString/0 ⇒ $ZodString",
        "a.rs $foo/0 ⇒ -",
        "a.ts \"~validate\"/0 ⇒ -",
        "a.ts \"zod 3\"/0 ⇒ -",
        "a.ts 图_report/0 ⇒ -",
        "a.hs foo'/1 ⇒ -",
        "a.hs unbox#/1 ⇒ -",
        "a.hs (<+>)/2 ⇒ -",
        "a.hs fmtRow/1 ⇒ fmtRow",
        "a.md Heading ⇒ -",
        "a.js x/0 ⇒ -",
        "a.txt x/0 ⇒ -",
    ] {
        let (input, want) = case.rsplit_once(" ⇒ ").expect("case has ` ⇒ `");
        let (rel, key) = input.split_once(' ').expect("path then key");
        let want = (want != "-").then(|| want.to_string());
        assert_eq!(mention_name(rel, key), want, "{case:?}");
    }
}
