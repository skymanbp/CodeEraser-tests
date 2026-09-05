//! `ce commitmsg <file>`, the commit-msg hook's face (plan v2.27 step
//! 5): the pre-commit gate with the message as one more surface. A
//! message that argues an erased name away is a site named
//! `COMMIT_EDITMSG:line prose`; a git comment line — the repository's
//! own comment prefix — is not; the class's deny tier past its budget
//! refuses the commit (exit 1) naming the site; a message file that
//! cannot be read is a usage error (exit 2), never a silent pass.

use crate::common;
use crate::tombstone_audit::{assert_refused, erased};
use crate::tombstone_guard::{site, sites};
use std::path::{Path, PathBuf};

/// The message git would hand the hook: a title, an argument from
/// absence, and a line of git's own comment block.
const MESSAGE: &str = "Drop the pork module\n\nbraise_pork is no longer needed.\n\
                       # Please enter the commit message: braise_pork is no longer here\n";

/// The erased repo with its deletion staged, plus `extra` files.
fn staged(tag: &str, extra: &str) -> PathBuf {
    let dir = erased(tag);
    if !extra.is_empty() {
        common::write_doc(&dir, extra);
    }
    common::git(&dir, &["add", "-A"]);
    dir
}

/// `ce commitmsg` over `msg` written where git writes it: the exit
/// code, stdout + stderr, and the feed line it left.
fn commitmsg(dir: &Path, msg: &str) -> (Option<i32>, String, serde_json::Value) {
    std::fs::write(dir.join(".git/COMMIT_EDITMSG"), msg).expect("message");
    let out = common::run_ce(dir, &["commitmsg", ".git/COMMIT_EDITMSG"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code(), text, common::last_observe(dir))
}

#[test]
fn the_message_is_a_prose_surface_and_a_comment_line_is_not() {
    let dir = staged("tomb-commitmsg-observe", "");
    let (code, text, line) = commitmsg(&dir, MESSAGE);
    assert_eq!(code, Some(0), "observe never blocks: {text}");
    assert_eq!(line["event"], "commitmsg", "{line}");
    assert!(
        line["session_id"].is_null(),
        "no session owns a git hook: {line}"
    );
    let want = [site("COMMIT_EDITMSG", 3, "prose")];
    assert_eq!(sites(&line["tombstone"]), want, "{line}");
    assert!(
        text.contains(&want[0]) && text.contains("ce commitmsg:"),
        "{text}"
    );
}

#[test]
fn a_deny_tier_past_its_budget_refuses_the_commit_and_names_the_message_line() {
    let dir = staged(
        "tomb-commitmsg-deny",
        "--- ce.toml\n[guard]\nmode = \"observe\"\n\n[tombstone]\ntier = \"deny\"\nbudget = 0\n",
    );
    let (code, text, line) = commitmsg(&dir, MESSAGE);
    assert_eq!(code, Some(1), "{text}");
    assert_refused(&text, &site("COMMIT_EDITMSG", 3, "prose"), &line);
}

/// The sites `ce commitmsg` seats for one message under the repository's
/// own git config (exit 0, observe): the staged deletion of a.py is R.
fn sites_under(tag: &str, config: &[(&str, &str)], msg: &str) -> Vec<String> {
    let dir = staged(tag, "");
    for (key, value) in config {
        common::git(&dir, &["config", key, value]);
    }
    let (code, text, line) = commitmsg(&dir, msg);
    assert_eq!(code, Some(0), "{text}");
    sites(&line["tombstone"])
}

#[test]
fn the_message_is_read_with_the_repository_s_own_comment_prefix() {
    // git reads `core.commentChar` / `core.commentString` as aliases, last
    // one set wins; the two keys are matched by their exact names and the
    // value kept byte for byte (codex review 2026-09-04: `core.commentary`
    // is somebody else's key, and a trailing space is part of the prefix —
    // git strips `// ` lines, not `//x` ones). And a message is a SURFACE:
    // its subject line is its one label, and `- braise_pork …` as a list
    // lead — a structural position that keeps a name alive in a file —
    // declares nothing here, so the item is a site.
    type Probe<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str, Vec<String>);
    let cases: [Probe; 3] = [
        (
            "tomb-commitmsg-prefix",
            &[("core.commentChar", ";")],
            "Drop the pork module\n; braise_pork is no longer needed.\n\nbraise_pork is no longer needed.\n",
            vec![site("COMMIT_EDITMSG", 4, "prose")],
        ),
        (
            "tomb-commitmsg-exact",
            &[("core.commentString", "// "), ("core.commentary", "zzz")],
            "Drop the pork module\n// braise_pork is no longer needed.\n//x braise_pork is no longer needed.\n",
            vec![site("COMMIT_EDITMSG", 3, "prose")],
        ),
        (
            "tomb-commitmsg-surface",
            &[],
            "Sides (no braise_pork)\n\n- braise_pork is no longer needed.\n",
            vec![
                site("COMMIT_EDITMSG", 1, "bracketed"),
                site("COMMIT_EDITMSG", 3, "prose"),
            ],
        ),
    ];
    for (tag, config, msg, want) in cases {
        assert_eq!(sites_under(tag, config, msg), want, "{tag}");
    }
}

#[test]
fn a_clean_message_over_the_staged_set_exits_zero_and_the_summary_names_the_face() {
    let dir = staged("tomb-commitmsg-clean", "");
    let (code, text, line) = commitmsg(&dir, "Drop the pork module\n");
    assert_eq!(code, Some(0), "{text}");
    assert!(text.starts_with("ce commitmsg: 1 staged file(s)"), "{text}");
    assert!(sites(&line["tombstone"]).is_empty(), "{line}");
}

#[test]
fn an_unreadable_message_file_is_a_usage_error_not_a_pass() {
    let dir = staged("tomb-commitmsg-missing", "");
    let out = common::run_ce(&dir, &["commitmsg", "no-such-file"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no-such-file"));
    assert!(
        !dir.join(".ce/observe.ndjson").exists(),
        "nothing was measured, so nothing is recorded"
    );
}
