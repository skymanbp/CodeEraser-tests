//! Ladder-test arrange + act helpers (the hooks.rs precedent:
//! common/ is one module per concern, each binary uses a subset —
//! the allows in mod.rs are module-level and cover this file too).

use codeeraser::graph::compdb_find;
use codeeraser::graph::ladder::{self, Outcome, Reason, Scope, c_head, java_header, lua_path};
use codeeraser::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What the walk hands the resolver: the judged files, the assets
/// and the resolver-config paths.
pub type Walked = (BTreeSet<String>, BTreeSet<String>, Vec<String>);

/// Materialize a ladder fixture tree and collect what the real walk
/// would hand the resolver (a node_modules is never entered, wherever
/// it sits — scan/walk.rs).
pub fn materialize(dir: &Path, tree: &[(&str, &str)]) -> Walked {
    for (rel, content) in tree {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, content).expect(rel);
    }
    let seen = tree
        .iter()
        .map(|(rel, _)| rel.to_string())
        .filter(|rel| !rel.starts_with("node_modules/") && !rel.contains("/node_modules/"));
    walk_sets(dir, seen)
}

/// Walked paths sorted the walk's own way (dedup/walkidx.rs
/// refresh_tree): a resolver config aside, a judged file into the set
/// the resolver sees (judged_path, the index walk's gate — the
/// scan-only arm never enters it), every other walked path an asset a
/// page may name.
pub fn walk_sets(dir: &Path, paths: impl IntoIterator<Item = String>) -> Walked {
    let (mut files, mut assets, mut configs) = (BTreeSet::new(), BTreeSet::new(), Vec::new());
    for rel in paths {
        let path = dir.join(&rel);
        if codeeraser::graph::store::is_resolver_config(&path) {
            configs.push(rel);
        } else if codeeraser::scan::lang::Lang::judged_path(&path).is_some() {
            files.insert(rel);
        } else {
            assets.insert(rel);
        }
    }
    (files, assets, configs)
}

/// A materialized ladder fixture that OWNS what a Scope borrows —
/// the shared arrange throat of every ladder test (the ratchet
/// caught three copies of the materialize + Scope stanza).
pub struct Fixture {
    pub dir: PathBuf,
    pub files: BTreeSet<String>,
    /// The walked files the index never holds (WalkIndex::assets) —
    /// what a page may name besides a judged file (the HTML rungs).
    pub assets: BTreeSet<String>,
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
    /// Every in-scope C-family file's include list, read as the walk
    /// reads it (c_head::read) — the compile database's include
    /// closure walks these (plan v2.30 step 5b, item 14).
    pub includes: BTreeMap<String, Vec<String>>,
}

pub fn fixture(tag: &str, tree: &[(&str, &str)]) -> Fixture {
    let dir = super::tmp(tag);
    let (files, assets, configs) = materialize(&dir, tree);
    let in_scope = &files;
    let of = |keep: fn(&str) -> bool| {
        tree.iter()
            .filter(move |(rel, _)| keep(rel) && in_scope.contains(*rel))
    };
    let java = of(|rel| rel.ends_with(".java"))
        .map(|(rel, text)| (rel.to_string(), java_header::read(text)))
        .collect();
    let lua = of(|rel| rel.ends_with(".lua"))
        .flat_map(|(_, text)| lua_path::read(text))
        .collect();
    let includes = of(compdb_find::is_c)
        .map(|(rel, text)| (rel.to_string(), c_head::read(text)))
        .collect();
    Fixture {
        dir,
        files,
        assets,
        configs,
        memo: Default::default(),
        crate_roots: BTreeSet::new(),
        search_roots: BTreeMap::new(),
        java,
        lua,
        includes,
    }
}

impl Fixture {
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            files: &self.files,
            assets: &self.assets,
            configs: &self.configs,
            root: &self.dir,
            memo: &self.memo,
            crate_roots: &self.crate_roots,
            search_roots: &self.search_roots,
            java: &self.java,
            lua: &self.lua,
            includes: &self.includes,
        }
    }
}

/// A compile database under `build/`, where clangd's root probe finds
/// it (compdb_find.rs), from `unit @@ command` rows: each entry's
/// `directory` is the build directory, its `file` the unit relative to
/// it, its `command` the row's text as the JSON string — so a relative
/// `-I` climbs `../` and a quoted argument reaches clang's reader as
/// written. The C-family batteries share this one writer.
pub fn compdb(dir: &Path, rows: &str) {
    let root = dir.to_string_lossy().replace('\\', "/");
    let json = |text: &str| text.replace('\\', "\\\\").replace('"', "\\\"");
    let entries: Vec<String> = rows
        .trim()
        .lines()
        .map(|row| {
            let (unit, command) = row.split_once(" @@ ").expect("unit @@ command");
            format!(
                "{{\"directory\": \"{root}/build\", \"file\": \"../{unit}\", \"command\": \"{}\"}}",
                json(command)
            )
        })
        .collect();
    std::fs::create_dir_all(dir.join("build")).expect("build dir");
    std::fs::write(
        dir.join("build/compile_commands.json"),
        format!("[{}]\n", entries.join(",\n")),
    )
    .expect("compile_commands.json");
}

/// (lang, site kind, from-file, spec) → expected outcome. Aliases
/// keep every row on one line: the table IS the spec, scannable or
/// dead.
pub type Case = (Lang, &'static str, &'static str, &'static str, Outcome);

/// Drive a case table through the dispatcher against one
/// materialized fixture — the shared act + assert throat, every row in
/// one batch (the core answers four languages' rows in one resolve/1
/// request since plan v2.33 wave W2a). A row's kind may carry the
/// site's line as `kind@line` (a folded import, a type reference inside
/// a class body — step 5b); a bare kind stands on line 1, exact for a
/// fixture with nothing above the site.
pub fn run_cases(fx: &Fixture, cases: Vec<Case>) {
    let scope = fx.scope();
    let sites: Vec<(Lang, ladder::Site)> = cases
        .iter()
        .map(|(lang, kind, from, spec, _)| {
            let (kind, line) = kind
                .split_once('@')
                .map_or((*kind, 1), |(k, l)| (k, l.parse().expect("a line after @")));
            (*lang, site(kind, from, spec, line))
        })
        .collect();
    let batch: Vec<(Lang, &ladder::Site)> = sites.iter().map(|(l, s)| (*l, s)).collect();
    let got = ladder::resolve_all(&batch, &scope).expect("the core answers resolve/1");
    for ((lang, at), (got, (.., want))) in sites.iter().zip(got.into_iter().zip(cases)) {
        assert_eq!(got, want, "{lang:?} {}@{} {:?}", at.kind, at.line, at.spec);
    }
}

/// A one-off site for direct dispatch — the inline-module cases build
/// theirs here; the tables go through run_cases.
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

/// One language's ladder battery as ONE text — the Java, Lua, R and C
/// fixtures: `==== path` blocks are the habitat's files, a `==== @cases`
/// block holds rows run as the tree stands, a `==== @rooted <dir> …`
/// block rows run with those directories declared as the language's
/// `[graph.search_roots]` entry (text_cases reads both), and a `====
/// @compdb` block is the habitat's compile database, written under
/// `build/` before every run (compdb). One literal per language rather
/// than a habitat, a case table and a test per scope: that stanza
/// rhymed token for token across the three files, and the clone gate
/// read the rhyme.
pub fn text_ladder(lang: Lang, text: &'static str) {
    let (marked, tree): (Vec<_>, Vec<_>) = text_tree(text)
        .into_iter()
        .partition(|(head, _)| head.starts_with('@'));
    let (dbs, runs): (Vec<_>, Vec<_>) =
        marked.into_iter().partition(|(head, _)| *head == "@compdb");
    assert!(
        !runs.is_empty(),
        "{}: a ladder text with no rows",
        lang.name()
    );
    // the fixture directory is the text's own: two tests of one language
    // run in parallel, and a shared name let each wipe the other's tree
    let own = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut h);
        h.finish()
    };
    for (n, (head, rows)) in runs.into_iter().enumerate() {
        let mut fx = fixture(&format!("ladder-{}-{own:016x}-{n}", lang.name()), &tree);
        for (_, entries) in &dbs {
            compdb(&fx.dir, entries);
        }
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

/// One run's rows, `kind @@ from @@ spec @@ outcome` per line (the
/// kind `kind@line` for a site off line 1, run_cases); the
/// outcome is `ok <path> <rung>` (a path may hold a space: the rung is
/// the last word), `pkg <dir> <rung>` (`.` = the tree root), `sec
/// <path> <slug> <rung>` (a section), `secf <path> <rung>` (the
/// section claim degraded to its file), `ext <rung>` or `no <reason>`
/// (the reason as the ledger spells it, reason_name).
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
        ["ok", path @ .., r] => ok(&path.join(" "), rung(r)),
        ["pkg", ".", r] => pkg("", rung(r)),
        ["pkg", dir, r] => pkg(dir, rung(r)),
        ["sec", path, slug, r] => sec(path, slug, rung(r)),
        ["secf", path, r] => secf(path, rung(r)),
        ["ext", r] => ext(rung(r)),
        ["no", why] => no(REASONS
            .into_iter()
            .find(|r| reason_name(*r) == *why)
            .unwrap_or_else(|| panic!("reason {why:?}"))),
        _ => panic!("outcome {spelled:?}"),
    }
}

/// Every refusal reason, so a table can name one as the ledger does.
const REASONS: [Reason; 11] = [
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
    Reason::OwnUnit,
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
