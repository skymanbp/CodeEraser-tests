//! R ladder fixtures (plan v2.30 step 4): `source` reads a path beside
//! the sourcing file, then under the tree root, then under the declared
//! `[graph.search_roots] r`; `library` and `pkg::` reach the package
//! whose DESCRIPTION the tree holds, and any other package is External.
//! Ambiguity rows MUST refuse: a package two DESCRIPTIONs declare, or a
//! file two declared roots hold, that resolves is the red condition.

use codeeraser::scan::lang::Lang;

use crate::common::text_ladder;

/// The habitat, `==== path` over each file, then the rows
/// (common::text_ladder; `pkg .` is the package at the tree root): a
/// package at the root, one in `sub/`, one name two directories
/// declare, and two declarable roots holding one name in common. Under
/// `[graph.search_roots] r = ["lib1", "lib2"]` (the `@rooted` rows) the
/// declared roots come after the file's own directory and the root.
const LADDER: &str = r#"
==== DESCRIPTION
Package: toolkit
Title: The root package
==== R/a.R
f <- function() 1
==== analysis/run.R
source("helpers.R")
==== analysis/helpers.R
g <- function() 2
==== scripts/setup.R
h <- 3
==== sub/DESCRIPTION
Package: inner
==== sub/R/x.R
x <- 1
==== one/DESCRIPTION
Package: twin
==== two/DESCRIPTION
Package: twin
==== lib1/u.R
u <- 1
==== lib2/u.R
u <- 2
==== lib1/only.R
o <- 1
==== @cases
source @@ analysis/run.R @@ helpers.R @@ ok analysis/helpers.R 1
source @@ analysis/run.R @@ scripts/setup.R @@ ok scripts/setup.R 1
source @@ analysis/run.R @@ ./R/a.R @@ ok R/a.R 1
source @@ analysis/run.R @@ https://example.org/remote.R @@ ext 3
source @@ analysis/run.R @@ /abs/path.R @@ no out_of_scope
source @@ analysis/run.R @@ u.R @@ no out_of_scope
source @@ analysis/run.R @@ gone.R @@ no out_of_scope
library @@ analysis/run.R @@ toolkit @@ pkg . 2
library @@ analysis/run.R @@ inner @@ pkg sub 2
library @@ analysis/run.R @@ twin @@ no ambiguous_workspace
library @@ analysis/run.R @@ ggplot2 @@ ext 3
==== @rooted lib1 lib2
source @@ analysis/run.R @@ u.R @@ no ambiguous_root
source @@ analysis/run.R @@ only.R @@ ok lib1/only.R 1
source @@ analysis/run.R @@ helpers.R @@ ok analysis/helpers.R 1
"#;

#[test]
fn r_rungs_resolve_and_refuse() {
    text_ladder(Lang::R, LADDER);
}
