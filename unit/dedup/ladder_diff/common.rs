//! What the legs share: the two-sided comparison of sites and a Scope's
//! owned inputs (the tally: tally.rs; the random-tree driver: legs.rs;
//! the seeded stream: rng.rs; the comparisons beside the sites:
//! beside.rs; the frozen dispatcher: unit/graph/ladder/frozen.rs).

use super::beside::names_never_move;
pub(super) use super::legs::{leg, legs, legs_marked, scratch};
use super::rng::Rng;
pub(super) use super::tally::{Mark, Tally};
use crate::graph::ladder::java_header::Header;
use crate::graph::ladder::lua_path::Template;
use crate::graph::ladder::{self, Memo, Outcome, Scope, Site};
use crate::scan::lang::Lang;
pub(super) use crate::testutil::write_tree;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn class(o: &Outcome) -> String {
    match o {
        Outcome::Resolved { rung, .. } => format!("resolved r{rung}"),
        Outcome::ResolvedPackage { rung, .. } => format!("package r{rung}"),
        Outcome::External { rung } => format!("external r{rung}"),
        Outcome::ResolvedSection { rung, slug, .. } => {
            format!("section r{rung} {}", slug.is_some())
        }
        Outcome::ResolvedInert { rung, .. } => format!("inert r{rung}"),
        Outcome::Unresolved(r) => format!("unresolved {r:?}"),
        other => format!("{other:?}"),
    }
}

/// The two sides on one batch of sites (one core request, as a sweep
/// sends): equal outcomes site by site.
pub(super) fn compare(
    sites: &[(Lang, Site)],
    core: &Scope,
    oracle: &Scope,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let batch: Vec<(Lang, &Site)> = sites.iter().map(|(l, s)| (*l, s)).collect();
    let answered = match ladder::resolve_all(&batch, core) {
        Ok(a) => a,
        Err(e) => return tally.miss(format!("core refused the batch: {e}\n{}", ctx())),
    };
    names_never_move(&batch, &answered, core, tally, ctx);
    for ((lang, site), got) in sites.iter().zip(answered) {
        let want = ladder::frozen::resolve(*lang, site, oracle);
        let name = lang.name();
        tally.count(name, class(&want));
        if tally.unexplained(site, oracle, got != want) {
            let (kind, from, spec) = (site.kind, site.from, site.spec);
            tally.miss(format!(
                "{name} kind={kind} from={from} spec={spec:?}: core {got:?} oracle {want:?}\n{}",
                ctx()
            ));
        }
    }
}

/// Owned inputs of a Scope, so a random tree can build two (one memo
/// each: the core side's and the frozen side's caches never meet).
#[derive(Default)]
pub(super) struct World {
    pub files: BTreeSet<String>,
    pub assets: BTreeSet<String>, // walked, never parsed: Markdown's second set
    pub configs: Vec<String>,
    pub search_roots: BTreeMap<String, BTreeSet<String>>,
    pub lua: BTreeSet<Template>,
    pub includes: BTreeMap<String, Vec<String>>,
    pub java: BTreeMap<String, Header>,
    /// Each package's `__all__` (none in a random world).
    pub py_all: BTreeMap<String, ladder::py_all::All>,
    /// The declared Rust crate roots (`[graph] crate_roots`).
    pub crate_roots: BTreeSet<String>,
    /// Each site's line, by its place in the tree's sites (absent: 1).
    pub lines: Vec<usize>,
}

impl World {
    /// A world of `base + below(span)` paths drawn from two tables.
    pub(super) fn seeded(
        rng: &mut Rng,
        (base, span): (usize, usize),
        dirs: &'static str,
        bases: &'static str,
    ) -> World {
        let n = base + rng.below(span);
        World {
            files: rng.paths(dirs, bases, n),
            ..World::default()
        }
    }

    /// Up to two declared search roots for `lang`, drawn from `table`.
    pub(super) fn declare(&mut self, rng: &mut Rng, lang: &str, table: &'static str) {
        let declared: BTreeSet<String> = (0..rng.below(3))
            .map(|_| rng.pick(table).to_string())
            .collect();
        if !declared.is_empty() {
            self.search_roots.insert(lang.into(), declared);
        }
    }

    /// Manifests named `name` (go.mod, DESCRIPTION) at the directories of
    /// the `|` table `dirs`, each written as `text` draws it with chance
    /// `pct` %, and (5 %) one listed at `ghost/` and never written.
    pub(super) fn manifests(
        &mut self,
        (rng, root): (&mut Rng, &Path),
        (dirs, name): (&'static str, &str),
        pct: usize,
        text: fn(&mut Rng) -> String,
    ) {
        for dir in dirs.split('|') {
            if rng.chance(pct) {
                let rel = crate::graph::roots::join_dir(dir, name);
                write_tree(root, &[(&rel, &text(rng))]);
                self.configs.push(rel);
            }
        }
        if rng.chance(5) {
            self.configs.push(format!("ghost/{name}"));
        }
    }

    /// Thirty sites, each from a file `keep` holds (see `Rng::origin`),
    /// the rest of the site drawn by `site` after its file.
    pub(super) fn sites(
        &self,
        rng: &mut Rng,
        keep: fn(&str) -> bool,
        (dirs, unwalked): (&'static str, &str),
        mut site: impl FnMut(&mut Rng, String) -> Owned,
    ) -> Vec<Owned> {
        let own: Vec<&String> = self.files.iter().filter(|f| keep(f)).collect();
        (0..30)
            .map(|_| {
                let from = rng.origin(&own, dirs, unwalked);
                site(rng, from)
            })
            .collect()
    }

    /// Both sides on this world's sites, rooted at `root`.
    pub(super) fn judge(&self, root: &Path, sites: &[(Lang, Site)], tally: &mut Tally) {
        self.with_scopes(root, |core, oracle| {
            compare(sites, core, oracle, tally, &|| self.describe(root));
        });
    }

    /// The two Scopes over this world (one memo each: the core side's and
    /// the frozen side's caches never meet), handed to `f`.
    pub(super) fn with_scopes<R>(&self, root: &Path, f: impl FnOnce(&Scope, &Scope) -> R) -> R {
        let (m1, m2) = (Memo::default(), Memo::default());
        let core = Scope {
            files: &self.files,
            assets: &self.assets,
            configs: &self.configs,
            root,
            memo: &m1,
            crate_roots: &self.crate_roots,
            search_roots: &self.search_roots,
            java: &self.java,
            lua: &self.lua,
            includes: &self.includes,
            py_all: &self.py_all,
        };
        let oracle = Scope { memo: &m2, ..core };
        f(&core, &oracle)
    }

    /// The tree as text, for a mismatch report.
    pub(super) fn describe(&self, root: &Path) -> String {
        let mut configs = Vec::new();
        texts(root, root, &mut configs);
        format!(
            "  files {:?}\n  assets {:?}\n  crate_roots {:?}\n  search_roots {:?}\n  lua {:?}\n  includes {:?}\n  java {:?}\n  configs {configs:?}",
            self.files,
            self.assets,
            self.crate_roots,
            self.search_roots,
            self.lua,
            self.includes,
            self.java
        )
    }
}

fn texts(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            texts(root, &p, out);
        } else if let Ok(text) = std::fs::read_to_string(&p) {
            let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
            out.push(format!("{rel}: {text}"));
        }
    }
}

/// A site's owned text (Site borrows): lang, kind, from, spec.
pub(super) type Owned = (Lang, &'static str, String, String);

/// One random tree: its world (configuration files already written
/// under the root it is handed) and its sites.
pub(super) type Tree = (World, Vec<Owned>);
