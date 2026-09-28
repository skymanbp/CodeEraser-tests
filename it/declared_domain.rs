//! The widened declaration domain on the advisory face (plan v2.30 step
//! 5b-6; fourclass/declared.rs): a Java `static` field, a C file-scope
//! variable and a TypeScript module-level `const` are symbols, so the
//! unmentioned advisory names the one no other file spells and stays
//! silent on the one another file does — the reason the ruling widened
//! the domain. One tree per language, every file an entry so the
//! advisory judges live files; `deadcode::run` is `ce deadcode`'s own
//! road, the same one deadcode_e2e's advisory legs read.

use crate::common;
use codeeraser::graph::deadcode;
use codeeraser::report::deadcode_json;

/// (tag, tree): a declaring file with one spoken and one unspoken
/// binding, and a second file spelling the spoken one.
const TREES: [(&str, &str); 3] = [
    (
        "java",
        "--- ce.toml\n[graph]\nentry_globs = [\"*.java\"]\n\
         --- A.java\npublic class A {\n  public static final int SPOKEN = 1;\n  \
         public static final int UNSPOKEN = 2;\n}\n\
         --- B.java\npublic class B {\n  int x = A.SPOKEN;\n}\n",
    ),
    (
        "c",
        "--- ce.toml\n[graph]\nentry_globs = [\"*.c\"]\n\
         --- a.c\nint spoken = 1;\nint unspoken = 2;\n\
         --- b.c\nextern int spoken;\nint twice(void) { return spoken * 2; }\n",
    ),
    (
        "ts",
        "--- ce.toml\n[graph]\nentry_globs = [\"*.ts\"]\n\
         --- a.ts\nexport const spoken = 1;\nexport const unspoken = 2;\n\
         --- b.ts\nimport { spoken } from './a';\nexport const twice = spoken * 2;\n",
    ),
];

#[test]
fn a_widened_domain_declaration_reaches_the_advisory() {
    let core = common::gates::core_bin();
    for (tag, doc) in TREES {
        let dir = common::fixtures::doc_tree(&format!("declared-domain-{tag}"), doc);
        let json = deadcode_json(&deadcode::run(&dir, None, &core).expect(tag));
        let rows = json["unmentioned"]
            .as_array()
            .unwrap_or_else(|| panic!("{tag}: advisory rows in {json}"));
        let named = |name: &str| {
            rows.iter().any(|row| {
                row["symbol"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case(name))
            })
        };
        assert!(named("unspoken"), "{tag}: {json}");
        assert!(!named("spoken"), "{tag}: {json}");
    }
}
