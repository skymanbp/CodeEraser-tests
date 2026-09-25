//! An R package's DESCRIPTION read for the graph (plan v2.30 step 4):
//! `Package` names the package — one word, `Packaged:` being another
//! field — and `Collate` lists its `R/` files, quoted or bare, over
//! continuation lines.

use super::{Description, read};
use crate::testutil::blocks;

/// Each block: `R @@ package @@ collate @@ why` over a DESCRIPTION text
/// (the collated files `, `-joined; `-` = none, and a package of `-`
/// names no package at all).
const CASES: &str = r#"
R @@ toolkit @@ aaa.R, b b.R, c.R @@ Collate over continuation lines, quoted or bare; Packaged is another field
Package: toolkit
Packaged: 2024-01-01 12:00:00 UTC; builder
Collate:
    'aaa.R'
    "b b.R" c.R
Version: 1.0
====
R @@ inner @@ - @@ no Collate: R loads every file of R/
Title: No collation
Package:   inner
====
R @@ spread @@ x.R, y.R @@ a value may start on the next line
Package:
  spread
Collate: 'x.R'
  'y.R'
====
R @@ - @@ - @@ Packaged alone names no package
Packaged: 2024-01-01; nobody
====
R @@ - @@ - @@ a package name is one word
Package: two words
"#;

/// The Description two columns spell, read from the directory `pkg`.
fn expected(package: &str, collate: &str) -> Option<Description> {
    (package != "-").then(|| Description {
        dir: "pkg".to_string(),
        package: package.to_string(),
        collate: collate
            .split(", ")
            .filter(|c| *c != "-")
            .map(String::from)
            .collect(),
    })
}

#[test]
fn a_description_names_its_package_and_its_collation() {
    for (_, [package, collate, why], text) in blocks(CASES) {
        assert_eq!(
            read(text, "pkg"),
            expected(package, collate),
            "{why}\n--- text ---\n{text}"
        );
    }
}
