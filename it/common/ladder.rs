//! Ladder-test arrange + act helpers (the hooks.rs precedent:
//! common/ is one module per concern, each binary uses a subset —
//! the allows in mod.rs are module-level and cover this file too).

use codeeraser::graph::ladder::{self, Outcome, Reason, Scope, java_header, lua_path};
use codeeraser::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Materialize a ladder fixture tree and collect what the real walk
/// would hand the resolver: the in-scope lang files (node_modules is
/// never entered) plus the resolver-config paths.
pub fn materialize(dir: &Path, tree: &[(&str, &str)]) -> (BTreeSet<String>, Vec<String>) {
    let mut files = BTreeSet::new();
    let mut configs = Vec::new();
    for (rel, content) in tree {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, content).expect(rel);
        if rel.starts_with("node_modules/") {
            continue;
        }
        // judged_path — the index walk's own gate (plan v2.5): the
        // scan-only arm never enters the file set the resolver sees
        if codeeraser::scan::lang::Lang::judged_path(&path).is_some() {
            files.insert(rel.to_string());
        }
        if codeeraser::graph::store::is_resolver_config(&path) {
            configs.push(rel.to_string());
        }
    }
    (files, configs)
}

/// A materialized ladder fixture that OWNS what a Scope borrows —
/// the shared arrange throat of every ladder test (the ratchet
/// caught three copies of the materialize + Scope stanza).
pub struct Fixture {
    pub dir: PathBuf,
    pub files: BTreeSet<String>,
    pub configs: Vec<String>,
    pub memo: ladder::Memo,
    /// Declared crate roots (ce.toml `[graph] crate_roots`); empty
    /// unless a leg sets it.
    pub crate_roots: BTreeSet<String>,
    /// Declared search roots (ce.toml `[graph.search_roots]`, language
    /// → directories); empty unless a leg sets it.
    pub search_roots: BTreeMap<String, BTreeSet<String>>,
    /// Every in-scope Java file's header, read as the walk reads it
    /// (java_header::read) — the Java ladder's candidate index.
    pub java: BTreeMap<String, java_header::Header>,
    /// The templates the in-scope Lua files assign to `package.path`,
    /// read as the walk reads them (lua_path::read).
    pub lua: BTreeSet<lua_path::Template>,
}

pub fn fixture(tag: &str, tree: &[(&str, &str)]) -> Fixture {
    let dir = super::tmp(tag);
    let (files, configs) = materialize(&dir, tree);
    let in_scope = &files;
    let of = |ext: &'static str| {
        tree.iter()
            .filter(move |(rel, _)| rel.ends_with(ext) && in_scope.contains(*rel))
    };
    let java = of(".java")
        .map(|(rel, text)| (rel.to_string(), java_header::read(text)))
        .collect();
    let lua = of(".lua")
        .flat_map(|(_, text)| lua_path::read(text))
        .collect();
    Fixture {
        dir,
        files,
        configs,
        memo: Default::default(),
        crate_roots: BTreeSet::new(),
        search_roots: BTreeMap::new(),
        java,
        lua,
    }
}

impl Fixture {
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            files: &self.files,
            configs: &self.configs,
            root: &self.dir,
            memo: &self.memo,
            crate_roots: &self.crate_roots,
            search_roots: &self.search_roots,
            java: &self.java,
            lua: &self.lua,
        }
    }
}

/// (lang, site kind, from-file, spec) → expected outcome. Aliases
/// keep every row on one line: the table IS the spec, scannable or
/// dead.
pub type Case = (Lang, &'static str, &'static str, &'static str, Outcome);

/// Drive a case table through the dispatcher against one
/// materialized fixture — the shared act + assert throat. Case rows
/// carry no line: table fixtures hold no inline modules, so line 1
/// is exact (the inline-module cases pass real lines directly).
pub fn run_cases(fx: &Fixture, cases: Vec<Case>) {
    let scope = fx.scope();
    for (lang, kind, from, spec, want) in cases {
        let got = ladder::resolve(lang, &site(kind, from, spec, 1), &scope);
        assert_eq!(got, want, "{lang:?} {kind} {spec:?}");
    }
}

/// A one-off site for direct dispatch — the inline-module cases need
/// a REAL line; the tables go through run_cases at line 1.
pub fn site(
    kind: &'static str,
    from: &'static str,
    spec: &'static str,
    line: usize,
) -> ladder::Site<'static> {
    ladder::Site {
        kind,
        from,
        spec,
        line,
    }
}

/// Ladder case-table constructors: resolved file, resolved package
/// directory (Go granularity), refusal, external.
pub fn ok(path: &str, rung: u8) -> Outcome {
    Outcome::Resolved {
        path: path.to_string(),
        rung,
    }
}

pub fn pkg(dir: &str, rung: u8) -> Outcome {
    Outcome::ResolvedPackage {
        dir: dir.to_string(),
        rung,
    }
}

pub fn no(reason: Reason) -> Outcome {
    Outcome::Unresolved(reason)
}

/// Resolved through a re-export surface (§4 R5 amendment).
pub fn via(path: &str, rung: u8) -> Outcome {
    Outcome::ResolvedVia {
        path: path.to_string(),
        rung,
    }
}

pub fn ext(rung: u8) -> Outcome {
    Outcome::External { rung }
}

pub fn sec(path: &str, slug: &str, rung: u8) -> Outcome {
    Outcome::ResolvedSection {
        path: path.to_string(),
        slug: Some(slug.to_string()),
        rung,
    }
}

/// The anchored link whose section claim degraded to a file-level
/// edge (design: ambiguous_anchor).
pub fn secf(path: &str, rung: u8) -> Outcome {
    Outcome::ResolvedSection {
        path: path.to_string(),
        slug: None,
        rung,
    }
}

/// One language's ladder battery as ONE text — the Java, Lua and R
/// fixtures: `==== path` blocks are the habitat's files, a `==== @cases`
/// block holds rows run as the tree stands and a `==== @rooted <dir> …`
/// block rows run with those directories declared as the language's
/// `[graph.search_roots]` entry (text_cases reads both). One literal per
/// language rather than a habitat, a case table and a test per scope:
/// that stanza rhymed token for token across the three files, and the
/// clone gate read the rhyme.
pub fn text_ladder(lang: Lang, text: &'static str) {
    let (runs, tree): (Vec<_>, Vec<_>) = text_tree(text)
        .into_iter()
        .partition(|(head, _)| head.starts_with('@'));
    assert!(
        !runs.is_empty(),
        "{}: a ladder text with no rows",
        lang.name()
    );
    for (n, (head, rows)) in runs.into_iter().enumerate() {
        let mut fx = fixture(&format!("ladder-{}-{n}", lang.name()), &tree);
        let mut words = head.split_whitespace();
        match words.next() {
            Some("@cases") => {}
            Some("@rooted") => {
                let roots = words.map(String::from).collect();
                fx.search_roots.insert(lang.name().to_string(), roots);
            }
            other => panic!("{}: unknown block {other:?}", lang.name()),
        }
        run_cases(&fx, text_cases(lang, rows));
    }
}

/// Text split at `==== ` into (head, body) blocks, the head the rest of
/// the marker's line.
fn text_tree(text: &'static str) -> Vec<(&'static str, &'static str)> {
    text.trim()
        .split("==== ")
        .filter(|block| !block.is_empty())
        .map(|block| block.split_once('\n').expect("a head over its body"))
        .collect()
}

/// One run's rows, `kind @@ from @@ spec @@ outcome` per line; the
/// outcome is `ok <path> <rung>`, `pkg <dir> <rung>` (`.` = the tree
/// root), `ext <rung>` or `no <reason>` (the reason as the ledger
/// spells it, reason_name).
fn text_cases(lang: Lang, table: &'static str) -> Vec<Case> {
    table
        .trim()
        .lines()
        .map(|line| {
            let [kind, from, spec, want]: [&'static str; 4] = line
                .split(" @@ ")
                .collect::<Vec<_>>()
                .try_into()
                .expect("kind @@ from @@ spec @@ outcome");
            (lang, kind, from, spec, text_outcome(want))
        })
        .collect()
}

fn text_outcome(spelled: &str) -> Outcome {
    let words: Vec<&str> = spelled.split(' ').collect();
    let rung = |w: &str| w.parse().expect("a rung");
    match words.as_slice() {
        ["ok", path, r] => ok(path, rung(r)),
        ["pkg", ".", r] => pkg("", rung(r)),
        ["pkg", dir, r] => pkg(dir, rung(r)),
        ["ext", r] => ext(rung(r)),
        ["no", why] => no(REASONS
            .into_iter()
            .find(|r| reason_name(*r) == *why)
            .unwrap_or_else(|| panic!("reason {why:?}"))),
        _ => panic!("outcome {spelled:?}"),
    }
}

/// Every refusal reason, so a table can name one as the ledger does.
const REASONS: [Reason; 10] = [
    Reason::Dynamic,
    Reason::AmbiguousPaths,
    Reason::AmbiguousRoot,
    Reason::AmbiguousWorkspace,
    Reason::AmbiguousExports,
    Reason::Macro,
    Reason::ConfigDepth,
    Reason::OutOfScope,
    Reason::Unsupported,
    Reason::Empty,
];

/// A refusal reason as the design §4 vocabulary spells it: the variant
/// name in snake case, so a new variant needs no table to be spelled.
pub fn reason_name(reason: Reason) -> String {
    let mut out = String::new();
    for c in format!("{reason:?}").chars() {
        if c.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}
