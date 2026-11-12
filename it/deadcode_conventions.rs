//! The tool conventions as entries (lane v235_licence, part 4A): a file
//! a tool finds by where it sits is no dead candidate, and the same name
//! where the rule does not name it still is. One fixture holds, per rule
//! kind of the core's `flags.entry_conventions`, the positive case and the
//! nearest negative; `ce deadcode` must name exactly the negatives, and
//! the query family (`role(F, convention)`, the one reason every face
//! prints) exactly the positives.

use crate::common;
use crate::query_face::{json_face, query_face, values};
use codeeraser::graph::deadcode;

/// The fixture: a path on a `--- ` line, its text under it. Positives —
/// `basename` (setup.py, docs/conf.py, package-info.java, a vitest
/// config), `under` (a page below `app/`), `exact` (root SECURITY.md, a
/// case-folded docs/contributing.md, docs/CODE_OF_CONDUCT.md, the root
/// init.lua). Negatives — a near name, a page below a `_` segment, the
/// exact names in another directory, a module's own init.lua.
const FIXTURE: &str = "\
--- setup.py
print(1)
--- docs/conf.py
project = 'x'
--- src/main/java/a/package-info.java
package a;
--- web/vitest.config.ts
export default {};
--- web/app/blog/page.tsx
export default function Page() { return null; }
--- SECURITY.md
# Security
--- docs/contributing.md
# Contributing
--- docs/CODE_OF_CONDUCT.md
# Conduct
--- init.lua
local x = 1
--- tools/setup_py.py
print(2)
--- web/app/_lib/page.tsx
export default function Lib() { return null; }
--- pkg/CONTRIBUTING.md
# Contributing
--- src/SECURITY.md
# Security
--- lua/foo/init.lua
local y = 2
";

/// The fixture's convention files, and the nearest negative of each kind.
const POSITIVE: &str = "SECURITY.md docs/CODE_OF_CONDUCT.md docs/conf.py docs/contributing.md
    init.lua setup.py src/main/java/a/package-info.java web/app/blog/page.tsx web/vitest.config.ts";
const NEGATIVE: &str =
    "lua/foo/init.lua pkg/CONTRIBUTING.md src/SECURITY.md tools/setup_py.py web/app/_lib/page.tsx";

#[test]
fn a_convention_file_is_an_entry_and_its_nearest_negative_is_not() {
    let dir = common::seeded_tree("deadcode-conventions", FIXTURE);
    let report = deadcode::run(&dir, None, &common::core_bin()).expect("deadcode");
    assert!(report.degraded.is_none(), "healthy run");
    let mut dead: Vec<&str> = report.dead.iter().map(|d| d.path.as_str()).collect();
    dead.sort();
    let negative: Vec<&str> = NEGATIVE.split_whitespace().collect();
    assert_eq!(dead, negative, "exactly the negatives are dead");
    let (code, doc) = json_face(&dir, &["query", "role(F, convention)"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        doc,
        query_face(&dir, "role(F, convention)", false),
        "one document"
    );
    let mut named: Vec<String> = values(&doc).into_iter().flatten().collect();
    named.sort();
    let positive: Vec<&str> = POSITIVE.split_whitespace().collect();
    assert_eq!(named, positive, "the reason is the convention role");
}
