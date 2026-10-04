//! What the legs share: the tally, the two-sided comparison (sites,
//! forced-include arcs), a Scope's owned inputs and the random-tree
//! driver (the seeded stream: rng.rs; the frozen dispatcher:
//! unit/graph/ladder/frozen.rs).

use super::rng::Rng;
use crate::graph::ladder::lua_path::Template;
use crate::graph::ladder::{self, Memo, Outcome, Scope, Site};
use crate::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What one leg saw: per language, the sites compared, the outcome
/// classes both sides gave, and the mismatches (first ones kept whole).
#[derive(Default)]
pub(super) struct Tally {
    sites: BTreeMap<&'static str, usize>,
    classes: BTreeMap<(&'static str, String), usize>,
    forced: (usize, usize),
    mismatches: usize,
    shown: Vec<String>,
}

impl Tally {
    pub(super) fn report(&self, leg: &str) {
        println!("== ladder_diff {leg} ==");
        for (lang, n) in &self.sites {
            println!("{lang}: sites {n}");
        }
        for ((lang, class), n) in &self.classes {
            println!("  {lang} {class}: {n}");
        }
        let (trees, arcs) = self.forced;
        println!("forced arcs: trees {trees} arcs {arcs}");
        println!("mismatches: {}", self.mismatches);
        self.shown.iter().for_each(|m| println!("MISMATCH {m}"));
        assert_eq!(self.mismatches, 0, "{leg}: core and frozen oracle disagree");
    }

    pub(super) fn sites(&self) -> usize {
        self.sites.values().sum()
    }

    fn miss(&mut self, what: String) {
        self.mismatches += 1;
        if self.shown.len() < 25 {
            self.shown.push(what);
        }
    }
}

fn lang_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Python => "python",
        Lang::Lua => "lua",
        Lang::Go => "go",
        Lang::C => "c",
        Lang::Cpp => "cpp",
        _ => "other",
    }
}

fn class(o: &Outcome) -> String {
    match o {
        Outcome::Resolved { rung, .. } => format!("resolved r{rung}"),
        Outcome::ResolvedPackage { rung, .. } => format!("package r{rung}"),
        Outcome::External { rung } => format!("external r{rung}"),
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
    for ((lang, site), got) in sites.iter().zip(answered) {
        let want = ladder::frozen::resolve(*lang, site, oracle);
        let name = lang_name(*lang);
        *tally.sites.entry(name).or_default() += 1;
        *tally.classes.entry((name, class(&want))).or_default() += 1;
        if got != want {
            let (kind, from, spec) = (site.kind, site.from, site.spec);
            tally.miss(format!(
                "{name} kind={kind} from={from} spec={spec:?}: core {got:?} oracle {want:?}\n{}",
                ctx()
            ));
        }
    }
}

/// The forced-include arcs both ways over one file set.
pub(super) fn compare_forced(
    root: &Path,
    files: &BTreeSet<String>,
    tally: &mut Tally,
    ctx: &dyn Fn() -> String,
) {
    let ids: BTreeMap<(&str, &str), usize> = files
        .iter()
        .enumerate()
        .map(|(i, f)| ((f.as_str(), ""), i))
        .collect();
    let (mut want, mut got) = (BTreeSet::new(), BTreeSet::new());
    ladder::frozen::forced_wire(root, files, &ids, &mut want);
    if let Err(e) = crate::graph::resolve::forced_wire(root, files, &ids, &mut got) {
        return tally.miss(format!("forced: core refused: {e}\n{}", ctx()));
    }
    tally.forced.0 += 1;
    tally.forced.1 += want.len();
    if got != want {
        tally.miss(format!("forced: core {got:?} oracle {want:?}\n{}", ctx()));
    }
}

/// Owned inputs of a Scope, so a random tree can build two (one memo
/// each: the core side's and the frozen side's caches never meet).
#[derive(Default)]
pub(super) struct World {
    pub files: BTreeSet<String>,
    pub configs: Vec<String>,
    pub search_roots: BTreeMap<String, BTreeSet<String>>,
    pub lua: BTreeSet<Template>,
    pub includes: BTreeMap<String, Vec<String>>,
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
    fn judge(&self, root: &Path, sites: &[(Lang, Site)], tally: &mut Tally) {
        let (none, java) = (BTreeSet::new(), BTreeMap::new());
        let (m1, m2) = (Memo::default(), Memo::default());
        let scope = |memo| Scope {
            files: &self.files,
            assets: &none,
            configs: &self.configs,
            root,
            memo,
            crate_roots: &none,
            search_roots: &self.search_roots,
            java: &java,
            lua: &self.lua,
            includes: &self.includes,
        };
        compare(sites, &scope(&m1), &scope(&m2), tally, &|| {
            self.describe(root)
        });
    }

    /// The tree as text, for a mismatch report.
    fn describe(&self, root: &Path) -> String {
        let mut configs = Vec::new();
        texts(root, root, &mut configs);
        format!(
            "  files {:?}\n  search_roots {:?}\n  lua {:?}\n  includes {:?}\n  configs {configs:?}",
            self.files, self.search_roots, self.lua, self.includes
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

/// Write a file under a tree root, parents made.
pub(super) fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    let parent = p.parent().expect("a joined path has a parent");
    std::fs::create_dir_all(parent)
        .and_then(|()| std::fs::write(&p, text))
        .expect("write");
}

/// A site's owned text (Site borrows): lang, kind, from, spec.
pub(super) type Owned = (Lang, &'static str, String, String);

/// One random tree: its world (configuration files already written
/// under the root it is handed) and its sites.
pub(super) type Tree = (World, Vec<Owned>);

/// A random leg: `CE_LADDER_DIFF_N` seeded trees, each built by `tree`
/// in a fresh directory, both sides on its sites (and on its forced-
/// include arcs when `forced`); the tally printed, any mismatch fails.
pub(super) fn leg(
    name: &str,
    salt: u64,
    forced: bool,
    mut tree: impl FnMut(&mut Rng, &Path) -> Tree,
) {
    let mut rng = Rng::new(salt);
    let mut tally = Tally::default();
    let n = std::env::var("CE_LADDER_DIFF_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);
    for i in 0..n {
        let root = scratch().join(format!("{name}-{i}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("scratch dir");
        let (world, owned) = tree(&mut rng, &root);
        let sites: Vec<(Lang, Site)> = owned
            .iter()
            .map(|(lang, kind, from, spec)| {
                let site = Site {
                    kind,
                    from,
                    spec,
                    line: 1,
                };
                (*lang, site)
            })
            .collect();
        world.judge(&root, &sites, &mut tally);
        if forced {
            compare_forced(&root, &world.files, &mut tally, &|| world.describe(&root));
        }
    }
    tally.report(&format!("{name} random"));
}

/// The scratch root the instruments write under.
pub(super) fn scratch() -> PathBuf {
    std::env::var("CE_LADDER_DIFF_SCRATCH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("ce-ladder-diff"))
}
