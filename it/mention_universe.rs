//! K23 / S-A22 (sealed criterion §1, §7): the self corpus's universe
//! is PINNED, not merely reported — but to a formula, never a literal,
//! because a literal breaks on the next added file and would be
//! re-established by rote (the citations-gate lesson). The formula is
//! the walk's own definition read off git: every tracked or
//! untracked-unignored file under `.gitignore` ALONE — git is asked
//! with `--exclude-per-directory=.gitignore`, never
//! `--exclude-standard`, because the walk turns `$GIT_DIR/info/exclude`
//! and `core.excludesFile` off (walk.rs) and one machine's own exclude
//! file must not move U — minus what the walk's published rules leave
//! out, one term per rule. Nothing here re-reads those rules —
//! `mention::cut`, `mention::excluded`, `mention::FILE_CAP` and
//! `mention::decode` ARE the rules — so a walk that drifted from its
//! ledger would disagree with git and the leg would say so, naming the
//! term. The index lives in a scratch directory: this leg writes
//! nothing into the repository. `formula` is shared with the corpus
//! instrument (eval_mention.rs), which pins the same identity on the
//! four external corpora and prints every term, so
//! `listed − Σ terms = U` closes inside the printed line.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use codeeraser::gitmodules;
use codeeraser::mention::{FILE_CAP, Stats, cut, decode, excluded};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// git's listing of `root` and the subtractions, one per rule of the
/// walk in the order the rules are asked. `files` are the members
/// left — U by the formula, the set the corpus instrument tokenizes.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Formula {
    pub listed: usize,
    #[serde(flatten)]
    pub terms: Terms,
    #[serde(skip)]
    pub files: Vec<String>,
}

/// The subtractions, one per rule of the walk, in the order the rules
/// are asked; flattened into the printed line beside `listed`.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Terms {
    /// a `.git` / `.ce` path component: git lists an untracked `.ce/`
    /// under `--others`, the walk never enters it
    pub named_cut: usize,
    /// inside — or itself — a directory owning a `.git`: a nested
    /// repository, which git lists as one `sub/` entry or a gitlink;
    /// a submodule the root's `.gitmodules` declares is not nested
    pub nested: usize,
    /// a TRACKED file a `.gitignore` pattern matches: the walk reads
    /// patterns and never the index (spec §1 C3), git lists it under
    /// `--cached`
    pub pattern_ignored: usize,
    /// the exclusion table (secrets, omni-mentioners)
    pub excluded: usize,
    /// listed but no regular file on disk — deleted unstaged, a broken
    /// link, a link to a directory: the walk never meets it
    pub absent: usize,
    pub oversize: usize,
    /// the early-NUL binary rule
    pub binary: usize,
}

impl Terms {
    fn sum(&self) -> usize {
        self.named_cut
            + self.nested
            + self.pattern_ignored
            + self.excluded
            + self.absent
            + self.oversize
            + self.binary
    }
}

impl Formula {
    pub fn universe(&self) -> usize {
        self.listed - self.terms.sum()
    }

    /// The pin: the pass's header must equal the formula in the two
    /// counted subtractions and in |U| (the other terms are subtracted
    /// before the walk ever counts).
    pub fn assert_matches(&self, stats: &Stats) {
        assert_eq!(
            (stats.universe, stats.skipped.binary, stats.skipped.oversize),
            (self.universe(), self.terms.binary, self.terms.oversize),
            "U = {} listed − {:?} (universe {}, skipped {:?})",
            self.listed,
            self.terms,
            stats.universe,
            stats.skipped
        );
        assert_eq!(stats.skipped.walk_errors, 0);
    }

    /// One listed entry, placed under the first rule that takes it.
    fn place(&mut self, root: &Path, rel: &str, ignored: &BTreeSet<String>) {
        self.listed += 1;
        let t = &mut self.terms;
        if rel.split('/').any(|c| c == ".git" || c == ".ce") {
            t.named_cut += 1;
        } else if cut(root, rel) {
            t.nested += 1;
        } else if ignored.contains(rel) {
            t.pattern_ignored += 1;
        } else if excluded(rel) {
            t.excluded += 1;
        } else {
            let path = root.join(rel);
            match std::fs::metadata(&path).ok().filter(|m| m.is_file()) {
                None => t.absent += 1,
                Some(m) if m.len() > FILE_CAP => t.oversize += 1,
                Some(_) if decode(&std::fs::read(&path).expect(rel)).is_none() => t.binary += 1,
                Some(_) => self.files.push(rel.to_string()),
            }
        }
    }
}

/// git asked per REPOSITORY: the root, then every declared submodule
/// whose checkout is seated (the walk's own exemption, walk.rs — a
/// declared submodule is this tree's, so its files enter U under
/// their superproject paths, and the superproject's bare gitlink row
/// for it is the one entry skipped: those files arrive through the
/// submodule's own listing). An unseated one is refused by the walk
/// before any listing (gitmodules.rs), so the formula, which mirrors
/// the walk, is never asked about it.
pub fn formula(root: &Path) -> Formula {
    let mut f = Formula::default();
    let seated: Vec<String> = gitmodules::seated(root);
    list_repo(&mut f, root, root, "", &seated);
    for sub in &seated {
        list_repo(&mut f, root, &root.join(sub), &format!("{sub}/"), &[]);
    }
    f
}

/// One repository's two git questions, `.gitignore` alone: the listing
/// (tracked, and untracked not pattern-matched) and the tracked files
/// a pattern matches — the one ignore source the walk reads, read the
/// one way the walk reads it (patterns, never the index). `.ceignore`
/// is the walk's second source and git's none, so its presence is
/// refused. `prefix` is the repository's path under `root`.
fn list_repo(f: &mut Formula, root: &Path, repo: &Path, prefix: &str, skip: &[String]) {
    const GITIGNORE_ONLY: &str = "--exclude-per-directory=.gitignore";
    let listed = git_z(repo, &["ls-files", "--cached", "--others", GITIGNORE_ONLY]);
    let ignored: BTreeSet<String> =
        git_z(repo, &["ls-files", "--cached", "--ignored", GITIGNORE_ONLY])
            .into_iter()
            .map(|r| format!("{prefix}{r}"))
            .collect();
    for rel in listed {
        let rel = format!("{prefix}{rel}");
        if skip.contains(&rel) {
            continue;
        }
        assert_ne!(
            rel.rsplit('/').next(),
            Some(".ceignore"),
            "{rel}: the formula reads git's `.gitignore` alone; the walk honours `.ceignore` too"
        );
        f.place(root, &rel, &ignored);
    }
}

fn git_z(root: &Path, args: &[&str]) -> Vec<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .arg("-z")
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout)
        .expect("utf-8 paths")
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

/// The mention pass over `root` into a scratch index named `tag`:
/// nothing is written into the tree under test.
fn pass(root: &Path, tag: &str) -> Stats {
    let scratch = common::tmp(tag);
    let idx = Index::open(&scratch.join("index.db"), Params::default()).expect("scratch index");
    codeeraser::mention::refresh(root, &idx).expect("mention pass")
}

#[test]
fn the_self_universe_is_gits_listing_minus_the_walks_own_rules() {
    let root = common::repo_root();
    let stats = pass(&root, "mention-universe-self");
    formula(&root).assert_matches(&stats);
    assert!(stats.universe > 500, "the repository itself is the corpus");
}

/// Every term the formula names, witnessed on one scratch repository
/// where each shape exists once, and the walk's own count agreeing
/// with the formula on that tree: a nested repository (git's `sub/`
/// entry), a tracked file under a `.gitignore` pattern, a tracked file
/// deleted unstaged, an untracked `.ce/`, and a `$GIT_DIR/info/exclude`
/// entry the walk must NOT read (its file stays in U).
#[test]
fn the_formula_names_every_rule_the_walk_has() {
    let root = common::tmp("mention-universe-shapes");
    common::git(&root, &["init", "-q", "."]);
    common::write_doc(
        &root,
        "--- a.rs\npub fn one() {}\n\
         --- .gitignore\nignored.txt\n\
         --- ignored.txt\ntracked, pattern-matched\n\
         --- gone.txt\ntracked, then deleted\n\
         --- sub/keep.txt\nin a repository of its own\n\
         --- .ce/index.db\nproduct state\n\
         --- scratch.txt\nexcluded by info/exclude, in U\n\
         --- .git/info/exclude\nscratch.txt\n",
    );
    common::git(
        &root,
        &["add", "-f", "a.rs", ".gitignore", "ignored.txt", "gone.txt"],
    );
    common::git(&root, &["commit", "-q", "-m", "shapes"]);
    std::fs::remove_file(root.join("gone.txt")).expect("delete unstaged");
    common::git(&root.join("sub"), &["init", "-q", "."]);
    let f = formula(&root);
    let t = &f.terms;
    assert_eq!(
        (f.listed, t.named_cut, t.nested, t.pattern_ignored, t.absent),
        (7, 1, 1, 1, 1),
        "{f:?}"
    );
    let members: BTreeSet<&str> = f.files.iter().map(String::as_str).collect();
    assert_eq!(
        members,
        BTreeSet::from([".gitignore", "a.rs", "scratch.txt"])
    );
    let stats = pass(&root, "mention-universe-shapes-db");
    f.assert_matches(&stats);
    assert_eq!(stats.universe, 3);
}

/// A nested repository is cut whole; the one the root's `.gitmodules`
/// declares is this tree's (plan v2.18: the suite rides at `cli/tests`
/// as a submodule) — read off the tracked file, so the exemption is
/// the commit's own fact with or without the checkout. The same `cut`
/// the formula above places every listed entry through.
#[test]
fn a_declared_submodule_is_not_a_nested_repository() {
    let dir = common::tmp("gitmodules-exempt");
    common::ladder::materialize(
        &dir,
        &[
            ("declared/.git", "gitdir: ../.git/modules/declared\n"),
            ("declared/inner/.keep", ""),
            ("foreign/.git", "gitdir: elsewhere\n"),
            (
                ".gitmodules",
                "[submodule \"suite\"]\n\tpath = declared/\n\turl = https://example.invalid/suite.git\n",
            ),
        ],
    );
    assert_eq!(
        gitmodules::declared(&dir).into_iter().collect::<Vec<_>>(),
        ["declared"]
    );
    assert!(!cut(&dir, "declared/a.rs"), "declared: this tree's");
    assert!(!cut(&dir, "declared/inner/b.rs"), "…all the way down");
    assert!(cut(&dir, "foreign/a.rs"), "undeclared: its own U");
    assert!(
        cut(&dir, "declared/.ce/x"),
        "the by-name cut still applies inside it"
    );
}

/// The parse half feeds the cut: git's own spellings — a quoted value,
/// an inline comment, a case-variant key under a case-variant header —
/// exempt, and a `path` outside a submodule stanza does not (the
/// grammar itself is pinned in gitmodules.rs's unit table; this leg
/// ties it to the rule the walk consults).
#[test]
fn the_gitmodules_reading_feeds_the_cut() {
    let dir = common::tmp("gitmodules-grammar");
    for repo in ["quoted dir", "inline", "case", "foreign"] {
        std::fs::create_dir_all(dir.join(repo)).expect("mkdir");
        std::fs::write(dir.join(repo).join(".git"), "gitdir: elsewhere\n").expect(repo);
    }
    std::fs::write(
        dir.join(".gitmodules"),
        "[submodule \"a\"]\n\tpath = \"quoted dir\"\n[submodule \"b\"]\n\tpath = inline # was elsewhere\n\
         [SUBMODULE \"c\"]\n\tPath = case\n[core]\n\tpath = foreign\n",
    )
    .expect(".gitmodules");
    let want: BTreeSet<String> = ["case", "inline", "quoted dir"].map(String::from).into();
    assert_eq!(gitmodules::declared(&dir), want);
    for rel in ["quoted dir/a.rs", "inline/a.rs", "case/a.rs"] {
        assert!(!cut(&dir, rel), "{rel}: declared by git's reading");
    }
    assert!(cut(&dir, "foreign/a.rs"), "a `path` outside a submodule stanza declares nothing");
}

/// The oracle: on the repository itself, the product's reading equals
/// git's (`git config -f .gitmodules`), so a mis-parse cannot keep the
/// K23 leg green by moving the walk and the formula together.
#[test]
fn declared_submodules_agree_with_git() {
    let root = common::repo_root();
    let out = Command::new("git")
        .args(["config", "-f", ".gitmodules", "-z", "--get-regexp", r"^submodule\..+\.path$"])
        .current_dir(&root)
        .output()
        .expect("git config");
    assert!(out.status.success(), "{out:?}");
    let gits: BTreeSet<String> = String::from_utf8(out.stdout)
        .expect("utf-8")
        .split('\0')
        .filter_map(|rec| rec.split_once('\n').map(|(_, v)| v.trim_end_matches('/').to_string()))
        .collect();
    assert!(!gits.is_empty(), "the repository declares its suite");
    assert_eq!(gitmodules::declared(&root), gits);
}

/// An unseated declared submodule is named by the shared predicate and
/// refuses the mention pass by name (ruling: one commit, one U — a
/// walk over the hollow checkout would shrink U silently); a declared
/// path that does not exist at all is a stale stanza and names nothing.
#[test]
fn an_unseated_declared_submodule_is_named_and_refused() {
    let sup = common::seed_superproject("mention-unseated", "suite");
    assert!(gitmodules::unseated(&sup).is_empty(), "seated: nothing to name");
    common::unseat(&sup, "suite");
    assert_eq!(gitmodules::unseated(&sup), ["suite"]);
    let scratch = common::tmp("mention-unseated-db");
    let idx = Index::open(&scratch.join("index.db"), Params::default()).expect("scratch index");
    let err = codeeraser::mention::refresh(&sup, &idx).expect_err("hollow U refused");
    assert!(format!("{err:#}").contains("suite is not checked out"), "{err:#}");
    std::fs::remove_dir(sup.join("suite")).expect("a stale stanza");
    assert!(gitmodules::unseated(&sup).is_empty(), "no directory, nothing unseated");
}
