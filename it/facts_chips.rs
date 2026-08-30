//! ADR-009 gate 4 (plan v2.21 S3): the chip surfaces. Each enrolled
//! file renders to itself through the registry (CE_BLESS=1 rewrites a
//! moved value in place), carries the chip count it is enrolled with
//! (a chip is added or retired by name), and no un-enrolled document
//! carries a chip at all.

use crate::common::{files_with_ext, repo_root};
use crate::facts::{self, chip};
use std::path::Path;

/// (repo-relative surface, chip count).
const SURFACES: &[(&str, usize)] = &[
    ("README.md", 13),
    ("README.zh.md", 13),
    ("contracts/DAEMON.md", 2),
    ("contracts/VERSIONING.md", 6),
    ("docs/RELEASE.md", 4),
    (
        "docs/reference/methodology/01-t1-t2-clone-detection-winnowing-fingerprint.md",
        3,
    ),
    (
        "docs/reference/methodology/02-t3-near-miss-clones-tree-edit-distance-tsed.md",
        1,
    ),
    (
        "docs/reference/methodology/03-documentation-duplication-shingling-minhash.md",
        2,
    ),
    (
        "docs/reference/methodology/06-graph-liveness-and-dead-code-verdicts.md",
        1,
    ),
    ("docs/reference/methodology/07-the-three-signal-join.md", 1),
    (
        "docs/reference/methodology/10-score-trajectory-the-trend-slope-verdict.md",
        1,
    ),
    (
        "docs/reference/methodology/11-fpr-discipline-and-the-guard-tier-ladder.md",
        1,
    ),
    (
        "docs/reference/methodology/13-unmentioned-declaration-advisory.md",
        1,
    ),
    ("site/index.html", 1),
    ("site/zh/index.html", 1),
];

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn every_chip_surface_renders_to_itself() {
    let root = repo_root();
    let bless = facts::blessing();
    let mut behind = Vec::new();
    for (rel, count) in SURFACES {
        let text = read(&root, rel);
        assert_eq!(
            chip::chips(&text, rel).len(),
            *count,
            "{rel}: chip count — enroll or retire the chip by name"
        );
        let (rendered, notes) = chip::render(&text, rel, &|id| facts::resolve(id));
        if rendered != text {
            if bless {
                std::fs::write(root.join(rel), rendered).expect("rewrite chips");
            }
            behind.extend(notes);
        }
    }
    assert!(
        behind.is_empty() || bless,
        "chips behind the registry (CE_BLESS=1 rewrites them):\n{}",
        behind.join("\n")
    );
}

/// Documents that DESCRIBE the chip syntax (the ADR-009 text) rather
/// than carry one: every open tag there must sit in a code span.
const DESCRIBES: &[&str] = &["docs/DEVELOPMENT_PLAN.md"];

#[test]
fn no_unenrolled_document_carries_a_chip() {
    let root = repo_root();
    let mut files = vec![
        root.join("README.md"),
        root.join("README.zh.md"),
        root.join("plugin/README.md"),
    ];
    for (dir, ext) in [("docs", "md"), ("contracts", "md"), ("site", "html")] {
        files_with_ext(&root.join(dir), ext, &mut files);
    }
    for path in files {
        let rel = path
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
        if DESCRIBES.contains(&rel.as_str()) {
            for (i, _) in text.match_indices(chip::OPEN) {
                assert!(
                    text[..i].ends_with('`'),
                    "{rel}: describes the chip syntax outside a code span at byte {i}"
                );
            }
            continue;
        }
        let enrolled = SURFACES.iter().any(|(s, _)| *s == rel);
        assert!(
            enrolled || !text.contains(chip::OPEN),
            "{rel} carries a chip but is not enrolled in facts_chips::SURFACES"
        );
    }
}
