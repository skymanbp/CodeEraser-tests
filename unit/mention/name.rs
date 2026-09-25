use super::mention_name;

/// One case: `path key ⇒ name`, or `⇒ -` for out of domain.
fn check(case: &str) {
    let (input, want) = case.rsplit_once(" ⇒ ").expect("case has ` ⇒ `");
    let (rel, key) = input.split_once(' ').expect("path then key");
    let want = (want != "-").then(|| want.to_string());
    assert_eq!(mention_name(rel, key), want, "{case:?}");
}

/// K24's extraction half (the frozen out-of-domain spellings of
/// §3.1/§2) beside the shapes that stay in. The keys are the
/// producer's own spellings (`(anonymous)/0` is what a TS anonymous
/// default export is keyed); the producer-measured witnesses of the
/// same exits are in mention::conv::tests.
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
        check(case);
    }
}

/// The plan v2.30 languages under the same domain: a C++ qualifier and
/// a Lua table prefix go with the last segment (`K::b`, `M:method`);
/// an R member (`x$f`), a dotted or dot-led R name, an operator, a
/// destructor, a specialisation and an anonymous unit are no single
/// token and fall out.
#[test]
fn the_newer_languages_keep_the_same_domain() {
    for case in [
        "a.c plain/2 ⇒ plain",
        "a.c FN ⇒ FN",
        "a.cpp K::b/0 ⇒ b",
        "a.cpp ns::In::out/0 ⇒ out",
        "a.cpp K::operator==/1 ⇒ -",
        "a.cpp K::~K/0 ⇒ -",
        "a.h spec<int>/1 ⇒ -",
        "a.lua open/0 ⇒ open",
        "a.lua M.setup/1 ⇒ setup",
        "a.lua M:method/1 ⇒ method",
        "a.lua a.b.c/0 ⇒ c",
        "a.lua (anonymous)/0 ⇒ -",
        "a.R check_input/1 ⇒ check_input",
        "a.r lower/0 ⇒ lower",
        "a.R check.input/1 ⇒ -",
        "a.R .onLoad/2 ⇒ -",
        "a.R x$f/0 ⇒ -",
    ] {
        check(case);
    }
}
