//! ADR-009 gate 4 (plan v2.21 S3/S4): the chip surfaces. Each
//! enrolled file renders to itself through the registry in its
//! declared language (CE_BLESS=1 rewrites a moved value in place),
//! carries the chip count it is enrolled with (a chip is added or
//! retired by name), and no un-enrolled document carries a chip at
//! all. The language is DECLARED, not guessed from the path: the
//! contracts and the release runbook are Chinese prose with no `zh`
//! marker, and a `#word` chip renders differently in each.

use crate::common::{files_with_ext, repo_root};
use crate::facts::{self, chip, read};

const EN: bool = false;
const ZH: bool = true;
const M: &str = "docs/reference/methodology/";

/// (repo-relative surface, chip count, Chinese?).
const SURFACES: &[(&str, usize, bool)] = &[
    ("README.md", 35, EN),
    ("README.zh.md", 35, ZH),
    ("contracts/DAEMON.md", 2, ZH),
    ("contracts/VERSIONING.md", 6, ZH),
    ("docs/RELEASE.md", 5, ZH),
    ("site/index.html", 5, EN),
    ("site/zh/index.html", 5, ZH),
    ("site/how/index.html", 10, EN),
    ("site/zh/how/index.html", 10, ZH),
    ("site/stack/index.html", 2, EN),
    ("site/zh/stack/index.html", 2, ZH),
];

/// The methodology booklets carrying chips (English): `file=count`
/// per line — one literal, not a tuple table (a list of same-shaped
/// tuples is a clone of every other such table by construction).
const BOOKLETS: &str = "
01-t1-t2-clone-detection-winnowing-fingerprint.md=3
02-t3-near-miss-clones-tree-edit-distance-tsed.md=1
03-documentation-duplication-shingling-minhash.md=2
06-graph-liveness-and-dead-code-verdicts.md=1
07-the-three-signal-join.md=1
10-score-trajectory-the-trend-slope-verdict.md=1
11-fpr-discipline-and-the-guard-tier-ladder.md=1
13-unmentioned-declaration-advisory.md=12
";

pub(crate) fn surfaces() -> Vec<(String, usize, bool)> {
    let booklets = BOOKLETS.lines().filter(|l| !l.is_empty()).map(|l| {
        let (name, n) = l.split_once('=').expect("file=count");
        (format!("{M}{name}"), n.parse().expect("a chip count"), EN)
    });
    SURFACES
        .iter()
        .map(|(rel, n, zh)| (rel.to_string(), *n, *zh))
        .chain(booklets)
        .collect()
}

#[test]
fn every_chip_surface_renders_to_itself() {
    let root = repo_root();
    let bless = facts::blessing();
    let mut behind = Vec::new();
    for (rel, count, zh) in surfaces() {
        let text = read(&root, &rel);
        assert_eq!(
            chip::chips(&text, &rel).len(),
            count,
            "{rel}: chip count — enroll or retire the chip by name"
        );
        let (rendered, notes) =
            chip::render(&text, &rel, zh, &|id| facts::render_in(&text, id, zh));
        if rendered != text {
            if bless {
                std::fs::write(root.join(&rel), rendered).expect("rewrite chips");
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
    let enrolled = surfaces();
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
        assert!(
            enrolled.iter().any(|(s, ..)| *s == rel) || !text.contains(chip::OPEN),
            "{rel} carries a chip but is not enrolled in facts_chips::SURFACES"
        );
    }
}
