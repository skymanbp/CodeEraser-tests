//! The receipt leg of the homepage screenshots gate (site_screenshots.rs
//! holds the ancestry, window, prose and asset legs): the receipt
//! `scripts/shoot_gui.js` leaves beside the three pictures names the
//! schemas this code declares, the bytes actually shot, and the `gui/ui`
//! tree that rendered them.
//!
//! Ancestry alone would not have caught the bug that started this road.
//! The candidates screen showed `ce.join-report/0.1.0` through two
//! schema bumps, and `gui/ui` never had to change for that: the SHAPE a
//! screen renders can move underneath a picture while the rendering code
//! stands still. So the shoot leaves a receipt, read here against the
//! constants themselves — and against the files on disk, because a
//! receipt nothing binds to the pixels is three strings anyone can edit
//! green.
//!
//! The `ui` digest closes the other blind spot: the ancestry leg reads
//! commits, so an edit to `gui/ui` that is not committed yet is invisible
//! to it, and a local run stayed green while the CI run on the commit
//! went red — twice (plan v2.29 steps 6 and 8). The digest is over the
//! tree itself, so it answers on either side of the commit. CRLF folds
//! to LF before hashing, exactly as `scripts/shoot_receipt.js` does: the
//! Windows checkout carries CRLF and the Linux one LF, and both must
//! name the same tree.

use crate::common::{repo_root, tmp, write_all};
use crate::site_screenshots::{RECEIPT, REGEN, SHOTS, UI, WINDOW};
use codeeraser::update::apply::hex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// Every file under `dir`, recursively, as `/`-joined paths relative to
/// it, sorted — the order the JavaScript side hashes in.
fn files_under(dir: &Path) -> Vec<String> {
    fn walk(base: &Path, at: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(at).expect("a gui/ui directory").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path.strip_prefix(base).expect("under the base");
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// `\r\n` → `\n`, byte-wise; a lone `\r` is left alone.
fn fold_crlf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// sha256 over `<rel>\0<folded bytes>\0` for every file under `dir`.
fn ui_digest(dir: &Path) -> String {
    let mut hash = Sha256::new();
    for rel in files_under(dir) {
        hash.update(rel.as_bytes());
        hash.update(b"\0");
        hash.update(fold_crlf(
            &std::fs::read(dir.join(&rel)).expect("a gui/ui file"),
        ));
        hash.update(b"\0");
    }
    hex(&hash.finalize())
}

#[test]
fn the_receipt_names_what_the_pictures_show() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(RECEIPT))
        .unwrap_or_else(|e| panic!("{RECEIPT}: {e} — the pictures have no receipt:\n  {REGEN}"));
    let receipt: Value = serde_json::from_str(&text).expect("the receipt is JSON");

    assert_eq!(
        receipt["window"],
        serde_json::json!([WINDOW.0, WINDOW.1]),
        "the receipt names another window"
    );

    for (face, live) in [
        ("structure", codeeraser::structure::judge::SCHEMA_ID),
        ("join", codeeraser::join::SCHEMA_ID),
        ("dedup", codeeraser::dedup::SCHEMA_ID),
    ] {
        assert_eq!(
            receipt["schemas"][face].as_str(),
            Some(live),
            "the {face} screen was photographed against a schema this code \
             no longer speaks — the picture shows a report shape that is \
             gone. Re-shoot:\n  {REGEN}"
        );
    }

    for name in SHOTS {
        let bytes = std::fs::read(root.join("site/assets").join(name))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            receipt["shots"][name].as_str(),
            Some(digest(&bytes).as_str()),
            "{name} is not the file the receipt was written for — either \
             the picture was replaced by hand or the receipt was. \
             Re-shoot:\n  {REGEN}"
        );
    }

    assert_eq!(
        receipt["ui"].as_str(),
        Some(ui_digest(&root.join(UI)).as_str()),
        "{UI} is not the tree these pictures were rendered from — an edit \
         to it (committed or not) has landed since the shoot, so the \
         homepage shows a window the product no longer has. Re-shoot:\n  {REGEN}"
    );
}

/// The digest is a function of content, not of line endings or of the
/// order the directory walk happens to yield: the same tree checked out
/// with CRLF on one platform and LF on another names one digest, and a
/// one-byte edit names another.
#[test]
fn the_ui_digest_folds_line_endings_and_sees_a_byte() {
    let dir = tmp("site_shots_receipt");
    write_all(
        &dir,
        &[
            ("b.js", "let a = 1;\nlet b = 2;\n"),
            ("sub/a.css", "x { y: z }\n"),
        ],
    );
    let lf = ui_digest(&dir);
    write_all(&dir, &[("b.js", "let a = 1;\r\nlet b = 2;\r\n")]);
    assert_eq!(ui_digest(&dir), lf, "CRLF and LF must name one tree");
    write_all(&dir, &[("b.js", "let a = 1;\nlet b = 3;\n")]);
    assert_ne!(ui_digest(&dir), lf, "a one-byte edit must move the digest");
}
