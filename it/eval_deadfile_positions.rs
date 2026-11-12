//! BARS §3's position reader on synthetic rules (lane v235_licence,
//! phases 2B part 1 and 2): the four rule kinds and their two flags as
//! addendum 4 asks for them, the refusals, the second-reading selector,
//! and the literal entry glob a matched path becomes.

use crate::common;
use crate::common::gates::core_bin;
use crate::eval_deadfile_bars::cases;
use crate::eval_deadfile_parts::audit::conventions::{
    Docs, candidates, carries, drawn_for_second_reading, matches,
};
use crate::eval_deadfile_parts::audit::population::literal;
use codeeraser::graph::deadcode;
use serde_json::{Value, json};

/// A synthetic candidate input: `name | position rule` (the citation
/// fields are filled in by `docs`).
pub const RULES: &str = r#"
package-info.java | {"kind": "basename", "names": ["package-info.java"]}
next-app-entry | {"kind": "under", "segment": "app", "names": ["page.tsx", "layout.tsx"]}
busted-spec | {"kind": "under", "segment": "spec", "names": ["*_spec.lua"]}
vitest.config | {"kind": "basename", "names": ["vitest.config.*"]}
CONTRIBUTING.md | {"kind": "exact", "paths": ["CONTRIBUTING.md", ".github/CONTRIBUTING.md"]}
koreader-plugin-meta | {"kind": "under", "segment": "plugins", "names": ["_meta.lua"]}"#;

pub fn docs(rules: &str, cited: bool) -> Docs {
    let quote = if cited { "the tool's sentence" } else { "" };
    rules
        .lines()
        .filter_map(|l| l.split_once(" | "))
        .map(|(name, rule)| {
            let position: Value = serde_json::from_str(rule).expect("rule");
            (
                name.to_string(),
                json!({"url": "https://example.org", "quote": quote, "position": position}),
            )
        })
        .collect()
}

/// `candidate | corpus | path => carries`; `SECURITY.md` has no rule in
/// the input, so it carries nothing.
const MARKS: &str = "
package-info.java | p | src/main/java/a/package-info.java => true
next-app-entry | zod | packages/docs/app/page.tsx => true
next-app-entry | zod | packages/docs/page.tsx => false
busted-spec | p | spec/util_spec.lua => true
busted-spec | p | spec/helper.lua => false
busted-spec | p | src/util_spec.lua => false
vitest.config | p | packages/a/vitest.config.mts => true
vitest.config | p | vitest.workspace.ts => false
CONTRIBUTING.md | p | CONTRIBUTING.md => true
CONTRIBUTING.md | p | .github/CONTRIBUTING.md => true
CONTRIBUTING.md | p | packages/docs-v3/CONTRIBUTING.md => false
SECURITY.md | p | SECURITY.md => false
koreader-plugin-meta | koreader | plugins/x.koplugin/_meta.lua => true
koreader-plugin-meta | lua | plugins/x.koplugin/_meta.lua => false";

/// Every `candidate | corpus | path => carries` row of `marks` under `rules`.
fn marked(rules: &str, marks: &'static str) {
    let given = docs(rules, true);
    for (g, want) in cases(marks) {
        let c = candidates()
            .into_iter()
            .find(|c| c.name == g[0])
            .expect("candidate");
        assert_eq!(carries(&c, &given, g[1], g[2]).to_string(), want, "{g:?}");
    }
}

#[test]
fn conventions_are_matched_by_the_input_position_rule() {
    marked(RULES, MARKS);
}

/// `path => drawn`: the second-reading selector, as the packets were drawn.
const DRAWN: &str = "
docs/conf.py => true
a/vitest.config.ts => true
a/vitest.config.mts => false
spec/x.lua => true
spec.lua => false
src/test/resources/a.html => true
lib/src/test/resources/a.html => true
xsrc/test/resources/a.html => false";

#[test]
fn the_second_reading_selector_is_the_packets_one() {
    for (g, want) in cases(DRAWN) {
        assert_eq!(drawn_for_second_reading(g[0]).to_string(), want, "{g:?}");
    }
}

/// Addendum 4's three additions: candidate 10 without case, candidate 4
/// with its first `app` and no later `_` segment, candidate 9's run.
const RULES_2B: &str = r#"
CONTRIBUTING.md | {"kind": "exact", "nocase": true, "paths": ["CONTRIBUTING.md", "docs/CONTRIBUTING.md", ".github/CONTRIBUTING.md"]}
SECURITY.md | {"kind": "exact", "paths": ["SECURITY.md", "docs/SECURITY.md", ".github/SECURITY.md"]}
next-app-entry | {"kind": "under", "segment": "app", "names": ["page.tsx", "layout.tsx", "route.ts"], "no_later_prefix": "_"}
maven-test-resources | {"kind": "sequence", "dirs": ["src", "test", "resources"]}"#;

const MARKS_2B: &str = "
CONTRIBUTING.md | p | contributing.md => true
CONTRIBUTING.md | p | .GitHub/Contributing.MD => true
CONTRIBUTING.md | p | packages/x/CONTRIBUTING.md => false
SECURITY.md | p | security.md => false
next-app-entry | p | app/page.tsx => true
next-app-entry | p | src/app/blog/[id]/page.tsx => true
next-app-entry | p | app/(group)/layout.tsx => true
next-app-entry | p | app/_private/page.tsx => false
next-app-entry | p | app/x/_y/z/route.ts => false
next-app-entry | p | _tools/app/page.tsx => true
next-app-entry | p | app/_p/app/page.tsx => false
next-app-entry | p | pages/page.tsx => false
next-app-entry | p | app/page.ts => false
maven-test-resources | p | src/test/resources/a.json => true
maven-test-resources | p | m/src/test/resources/x/y.txt => true
maven-test-resources | p | src/test/java/resources/a => false
maven-test-resources | p | src/test/resources => false
maven-test-resources | p | xsrc/test/resources/a => false";

#[test]
fn the_three_additions_read_as_addendum_four_says() {
    marked(RULES_2B, MARKS_2B);
}

/// `rule => refusal`: an unknown kind, a key the kind does not take, a
/// field of the wrong type — each refused by the candidate's name.
const REFUSED: &str = r#"
{"kind": "glob", "names": ["x"]} => c: unknown position kind "glob"
{"kind": "basename", "names": ["x"], "nocase": true} => c: position kind basename takes no `nocase`
{"kind": "exact", "paths": ["x"], "nocase": "yes"} => c: `nocase` has the wrong type
{"kind": "under", "segment": "app", "names": ["x"], "no_later_prefix": 1} => c: `no_later_prefix` has the wrong type
{"kind": "sequence", "dirs": "src"} => c: position rule without `dirs`"#;

#[test]
fn a_rule_the_reader_does_not_know_is_refused() {
    for line in REFUSED.lines().filter(|l| !l.is_empty()) {
        let (rule, want) = line.split_once(" => ").expect("row");
        let rule: Value = serde_json::from_str(rule).expect("rule");
        let got = std::panic::catch_unwind(|| matches("c", &rule, "app/x"));
        let said = got.expect_err(line);
        let said = said.downcast_ref::<String>().cloned().unwrap_or_default();
        assert_eq!(said, want, "{line}");
    }
}

/// A matched path enters `[graph] entry_globs` as a literal: `[id]`,
/// `(group)` and `{x}` name exactly their own file, never the sibling a
/// glob reading of the same characters would also select. The files are
/// named `view.tsx` / `frame.tsx`, no convention's name (since part 4A
/// `page.tsx` below `app/` is an entry of the product itself).
#[test]
fn a_literal_entry_names_exactly_its_file() {
    let entries = [
        "app/blog/[id]/view.tsx",
        "app/(group)/frame.tsx",
        "app/{x,y}/view.tsx",
    ];
    let globs: Vec<String> = entries
        .iter()
        .map(|p| format!("{:?}", literal(p)))
        .collect();
    let tree = format!(
        "--- ce.toml\n[graph]\nentry_globs = [{}]\n--- app/blog/[id]/view.tsx\nexport {{}};\n--- app/blog/i/view.tsx\nexport {{}};\n\
         --- app/(group)/frame.tsx\nexport {{}};\n--- app/{{x,y}}/view.tsx\nexport {{}};\n--- app/x/view.tsx\nexport {{}};\n",
        globs.join(", ")
    );
    let dir = common::fixtures::doc_tree("deadfile-literal-entry", &tree);
    let report = deadcode::run(&dir, None, &core_bin()).expect("deadcode");
    let dead: Vec<&str> = report.dead.iter().map(|d| d.path.as_str()).collect();
    assert_eq!(dead, ["app/blog/i/view.tsx", "app/x/view.tsx"], "{globs:?}");
}
