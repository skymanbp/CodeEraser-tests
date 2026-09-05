use super::*;
use crate::testutil::scratch;
use std::path::PathBuf;

fn git(root: &Path, args: &[&str]) {
    let mut full = vec![
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@t",
        "-c",
        "commit.gpgsign=false",
    ];
    full.extend_from_slice(args);
    let out = crate::proc::git_output(root, &full).expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// One Markdown file at `rel` holding three texts at once: a committed
/// one, a staged rewrite, and a further working-tree rewrite. Written
/// as a script table so the seed is not the repo-birth stanza every
/// it leg spells (the clone gate's own word on the first draft).
fn three_texts(tag: &str, rel: &str) -> PathBuf {
    let root = scratch(tag);
    git(&root, &["init", "-q"]);
    let at = root.join(rel);
    std::fs::create_dir_all(at.parent().expect("parent")).expect("mkdir");
    for (text, then) in [
        ("# committed\n", "commit"),
        ("# staged\n", "add"),
        ("# worktree\n", ""),
    ] {
        std::fs::write(&at, text).expect("write the file");
        if !then.is_empty() {
            git(&root, &["add", rel]);
        }
        if then == "commit" {
            git(&root, &["commit", "-qm", "seed"]);
        }
    }
    root
}

fn one(rel: &str) -> PathPair {
    (Some(rel.to_string()), Some(rel.to_string()))
}

/// HEAD against `after`, or the test dies naming git.
fn loaded(root: &Path, pairs: &[PathPair], after: Side) -> (Vec<Loaded>, usize) {
    load(root, pairs, Side::Rev("HEAD"), after).expect("git answers")
}

#[test]
fn every_side_comes_from_where_it_says() {
    let root = three_texts("tombstone-texts-sides", "a.md");
    let pairs = [one("a.md")];
    let (got, unread) = loaded(&root, &pairs, Side::Worktree);
    assert_eq!((got.len(), unread), (1, 0));
    assert_eq!(
        (got[0].before.as_str(), got[0].after.as_str()),
        ("# committed\n", "# worktree\n")
    );
    assert_eq!((got[0].rel.as_str(), got[0].lang), ("a.md", Lang::Markdown));
    let (got, _) = loaded(&root, &pairs, Side::Index);
    assert_eq!(got[0].after, "# staged\n");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_absent_side_is_empty_and_a_missing_or_unjudged_pair_drops() {
    let root = three_texts("tombstone-texts-drops", "a.md");
    let pairs = [
        (None, Some("a.md".to_string())),
        one("never.md"),
        one("notes.txt"),
    ];
    let (got, unread) = loaded(&root, &pairs, Side::Worktree);
    assert_eq!(
        (got.len(), unread),
        (1, 1),
        "the missing blob is counted unread; the unjudged file is no pair at all"
    );
    assert_eq!(
        (got[0].before.as_str(), got[0].after.as_str()),
        ("", "# worktree\n")
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_nested_root_reads_its_own_relative_paths_through_git() {
    // `./`-relative specs against `git -C root`: no prefix spawn
    let root = three_texts("tombstone-texts-nested", "pkg/n.md");
    let (got, _) = loaded(&root.join("pkg"), &[one("n.md")], Side::Worktree);
    assert_eq!(
        (
            got[0].rel.as_str(),
            got[0].before.as_str(),
            got[0].after.as_str()
        ),
        ("n.md", "# committed\n", "# worktree\n")
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn pairs_past_the_cap_are_counted_not_read() {
    let root = three_texts("tombstone-texts-cap", "a.md");
    let pairs: Vec<PathPair> = (0..PAIR_CAP + 3).map(|_| one("a.md")).collect();
    let (got, unread) = loaded(&root, &pairs, Side::Worktree);
    assert_eq!((got.len(), unread), (PAIR_CAP, 3));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_bounded_reader_refuses_binary_and_absence() {
    let root = scratch("tombstone-read-capped");
    let cases: [(&str, &[u8], Option<&str>); 2] = [
        ("t.md", b"# ok\n", Some("# ok\n")),
        ("b.bin", &[0xff, 0xfe, 0x00], None),
    ];
    for (name, bytes, want) in cases {
        let at = root.join(name);
        std::fs::write(&at, bytes).expect("write case");
        assert_eq!(read_capped(&at).as_deref(), want, "{name}");
    }
    assert_eq!(read_capped(&root.join("none.md")), None);
    let _ = std::fs::remove_dir_all(&root);
}
