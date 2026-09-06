//! BENCH.md's joining rule, executable
//! (docs/BENCH.md:31-40 `joins only when`): a release joins the series
//! only when there is something new to measure. Replaying the whole
//! series to add a duplicate measurement publishes machine drift under
//! a new version number, which is the one thing that page's own header
//! warns readers against.
//!
//! What a measurement is a measurement OF is the sources AND the
//! manifests, lockfiles and toolchain pin that decide what those
//! sources compile into. The rule read two directories, and a
//! dependency bump moves numbers no line under `cli/src` explains:
//! tree-sitter 0.27 re-timed the tokenizer, rusqlite 0.40 the index.
//! A release carrying only such a bump was turned away as "the same
//! program".
//!
//! Every release also rewrites its own version in three of those files
//! — `cli/Cargo.toml`, `cli/Cargo.lock`, `core/ce-core.cabal` — and
//! reading that stamp as new code would let EVERY release join, which
//! is the failure the rule exists to prevent. So the stamp is dropped,
//! at both ends, by the version each side actually declares. Replayed
//! over the whole tag list, the four tags the narrow rule turned away
//! (v0.7.1, v1.0.1, v1.3.1, v1.3.2) are exactly the four this one
//! turns away.
//!
//! Both row writers ask here, and so does the renderer: a surface that
//! says a release has no row must be able to say WHICH of the two
//! reasons applies, or a reader takes "no row" to mean "no code
//! changed".

use super::git_out;

/// The sources, compared BY NAME so a rename or a binary file counts
/// the way it always has. The pathspecs are top-level (`:/`) because
/// the harness runs from `cli/`.
const SOURCES: [&str; 2] = [":/cli/src", ":/core/app"];

/// What those sources compile into, compared by CONTENT: three of
/// these carry the release's own version stamp, and a name-only read
/// could not tell that stamp from a dependency bump.
const BUILD_INPUTS: [&str; 6] = [
    ":/cli/Cargo.toml",
    ":/cli/Cargo.lock",
    ":/core/ce-core.cabal",
    ":/core/cabal.project",
    ":/core/cabal.project.freeze",
    ":/rust-toolchain.toml",
];

/// `git diff <flavour> prev rev -- <paths>` — one owner, so the two
/// halves of the rule cannot drift apart in how they ask.
fn diff(flavour: &str, prev: &str, rev: &str, paths: &[&str]) -> String {
    let mut args = vec!["diff", flavour, prev, rev, "--"];
    args.extend(paths);
    git_out(&args)
}

/// The crate version a revision DECLARES — read from the manifest, not
/// from the tag name: v0.1.0 is tagged on a tree whose manifest says
/// 0.0.1.
fn crate_version(rev: &str) -> String {
    let at = format!("{rev}:cli/Cargo.toml");
    git_out(&["show", &at])
        .lines()
        .find_map(|line| line.trim().strip_prefix("version = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_default()
}

/// A changed line that says nothing but "this is release X". Matching
/// against the version the side actually declares is what keeps a
/// DEPENDENCY's version line — the lock is full of them — a change.
fn version_stamp(line: &str, version: &str) -> bool {
    let Some(rest) = line.get(1..).unwrap_or("").trim().strip_prefix("version") else {
        return false;
    };
    let value = rest.trim_start().trim_start_matches([':', '=']).trim();
    !version.is_empty() && value.trim_matches('"') == version
}

/// Anything the build inputs changed beyond the two version stamps.
fn build_inputs_changed(prev: &str, rev: &str) -> bool {
    let (before, after) = (crate_version(prev), crate_version(rev));
    diff("-U0", prev, rev, &BUILD_INPUTS)
        .lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("+++")
                && !l.starts_with("---")
        })
        .any(|l| !version_stamp(l, &before) && !version_stamp(l, &after))
}

/// Does `rev` bring something new to measure over `prev`?
///
/// A git that failed is a named refusal in `git_out`, never an empty
/// answer read as "nothing changed" — a missing rev or a shallow clone
/// would otherwise stop the series growing in silence.
pub fn brings_something_new(prev: &str, rev: &str) -> bool {
    !diff("--name-only", prev, rev, &SOURCES).is_empty() || build_inputs_changed(prev, rev)
}
