//! The three GUI pictures on the homepage are a REPORT, not a pose.
//!
//! They were posed by hand on 2026-08-22 and then went stale in
//! silence, because nothing on the site could re-derive them: an
//! eight-tab strip after the strip grew to ten, `ce.join-report/0.1.0`
//! after the schema reached 0.3.0, a tree measuring `cli/tests` inline
//! after it became a submodule that is read but not measured, and alt
//! text in both languages stating a score the structure face had long
//! since stopped giving. Every one of those facts is stated elsewhere
//! on the same site, derived — so the pictures were the only surface
//! left that could disagree with the product and not be caught.
//!
//! `scripts/shoot_gui.js` ended the posing: it renders the real
//! `gui/ui` in headless Edge — the engine the shipped app draws
//! through, since Tauri on Windows is WebView2 — over report documents
//! the CLI produced, which are the documents the webview itself would
//! have received (`ce … --format json` and `faces::*` call the same
//! `report_json` over the same `judge::run`). The six legs below hold
//! what that script cannot.
//!
//! The freshness leg is deliberately blunt — ANY commit to `gui/ui`
//! after a picture's commit fails it. Re-shooting is one command, and
//! the alternative (judging which UI edits are "visible") is the
//! judgement call that let four of them through. It is also not
//! sufficient alone, which is why the receipt leg exists: the join
//! schema moved twice under a picture while `gui/ui` stood still.
//!
//! Deliberately NOT held: that the numbers inside the pictures are
//! current. A screenshot samples one run and the page claims no more;
//! every figure the page states as fact is derived text elsewhere on
//! it. Gating pixel currency would mean a `ce join` run per commit.

use crate::common::{git_out, repo_root};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

/// The pictures the homepage carries, in the order it shows them.
const SHOTS: [&str; 3] = ["gui-structure.png", "gui-tree.png", "gui-candidates.png"];

/// The shipped app's default window, as `scripts/shoot_gui.js`
/// captures it.
const WINDOW: (u32, u32) = (1424, 892);

/// What a failure tells the operator to run. One string, because four
/// legs quote it and a stale instruction is its own bug.
const REGEN: &str = "node scripts/shoot_gui.js --out site/assets";

/// Both homepages, which are the pictures' only home (they left the
/// READMEs at v1.3.0).
const PAGES: [&str; 2] = ["site/index.html", "site/zh/index.html"];

/// The receipt `scripts/shoot_gui.js` leaves beside the pictures.
const RECEIPT: &str = "contracts/gui-shots.json";

/// The rendering surface the pictures are photographs of.
const UI: &str = "gui/ui";

/// The last commit that touched any of `paths`. A clone with no
/// history cannot answer when a picture was taken, so it refuses by
/// name rather than passing vacuously — the same stance
/// `graph_provenance.rs` takes, and why both CI jobs check out with
/// `fetch-depth: 0`.
fn last_commit(root: &Path, paths: &[&str]) -> String {
    let mut args = vec!["log", "-1", "--format=%H", "--"];
    args.extend_from_slice(paths);
    let (ok, out) = git_out(root, &args);
    let sha = out.trim().to_string();
    assert!(
        ok && !sha.is_empty(),
        "no commit history for {paths:?} — a shallow clone cannot say \
         when the screenshots were taken, and a gate that cannot \
         measure must not pass"
    );
    sha
}

/// Width and height out of the PNG header, having first checked that
/// the file really is one: the signature, then the IHDR chunk that the
/// format requires to come first.
fn png_window(bytes: &[u8], name: &str) -> (u32, u32) {
    const MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
    assert!(
        bytes.len() > 24 && bytes.starts_with(MAGIC) && &bytes[12..16] == b"IHDR",
        "{name} is not a PNG"
    );
    let read = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().expect("four bytes"));
    (read(16), read(20))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The `<figure>` blocks that carry a shot, as (alt, caption).
///
/// Block-oriented, not line-oriented: the caption sits on its own line
/// under the `<img>`, and reading only the image's line would have
/// left the visible half of every figure ungated. Splitting on the tag
/// also survives an attribute reflow, which a line scan does not.
fn shot_figures(page: &str) -> Vec<(String, String)> {
    page.split("<figure")
        .filter(|block| block.contains("/assets/gui-"))
        .map(|block| {
            let pick = |open: &str, close: &str| {
                block
                    .split_once(open)
                    .and_then(|(_, rest)| rest.split_once(close))
                    .map(|(text, _)| text.to_string())
                    .unwrap_or_else(|| panic!("a shot figure with no {open}…{close}"))
            };
            (pick(" alt=\"", "\""), pick("<figcaption>", "</figcaption>"))
        })
        .collect()
}

/// Leg 1: no picture is older than the screens it shows. If `gui/ui`
/// moved after a picture was taken, the homepage is showing a window
/// the product no longer has.
///
/// Per picture, not per set: `git log -1` over all three answers the
/// NEWEST, so a partial re-shoot would have refreshed the other two by
/// association.
#[test]
fn the_pictures_are_no_older_than_the_screens_they_show() {
    let root = repo_root();
    assert!(
        root.join(UI).is_dir(),
        "{UI} is gone — this gate watches a path that no longer exists \
         and would pass forever. Repoint it."
    );
    let ui = last_commit(&root, &[UI]);
    for name in SHOTS {
        let path = format!("site/assets/{name}");
        let taken = last_commit(&root, &[&path]);
        let (fresh, _) = git_out(&root, &["merge-base", "--is-ancestor", &ui, &taken]);
        assert!(
            fresh,
            "{UI} last moved in {ui}, after {name} was taken in {taken} — \
             the homepage is showing a window the product no longer has. \
             Re-shoot:\n  {REGEN}"
        );
    }
}

/// Leg 2: each picture is a whole app window at the shipped default
/// size. A hand-cropped or hand-scaled replacement is exactly the
/// posing this road removed, and it reads as an ordinary file swap in
/// review.
#[test]
fn every_picture_is_a_whole_app_window() {
    let root = repo_root();
    for name in SHOTS {
        let path = root.join("site/assets").join(name);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            png_window(&bytes, name),
            WINDOW,
            "{name} is not the app's window — re-shoot it:\n  {REGEN}"
        );
    }
}

/// Leg 2b: each page reserves the picture's box before its bytes land.
/// `width`/`height` on the `<img>` are the window, so a lazy picture
/// arriving does not move the text under it — and a re-shoot at some
/// other window would leave the attributes lying about the file.
#[test]
fn every_page_reserves_the_window_before_the_picture_loads() {
    let root = repo_root();
    let attrs = format!("width=\"{}\" height=\"{}\"", WINDOW.0, WINDOW.1);
    for page in PAGES {
        let text =
            std::fs::read_to_string(root.join(page)).unwrap_or_else(|e| panic!("{page}: {e}"));
        for name in SHOTS {
            let tag = format!("<img src=\"/assets/{name}\" {attrs} ");
            assert!(
                text.contains(&tag),
                "{page}: the <img> for {name} does not reserve the app window ({attrs})"
            );
        }
    }
}

/// Leg 3: no page re-types beside a picture a number the picture
/// already carries. Both halves of the figure are held — the alt text
/// a screen reader hears AND the caption everyone else reads; the
/// English and Chinese alt text both said `854/1000` while the
/// structure face had moved to 832, and neither page could notice.
///
/// `is_numeric`, not `is_ascii_digit`, so the Chinese page is covered
/// where it plausibly needs to be: fullwidth ８３２ is `Nd` and 〇 is
/// `Nl`, both caught. The ideographic 八三二 is NOT — those are
/// `Lo`, letters — and they are deliberately left out rather than
/// added to the predicate: 一, 十 and 百 are ordinary words in ordinary
/// prose (this page's own caption says 同一个尺子), so a predicate that
/// flagged them would fire on every sentence and be switched off. A
/// score written 八三二 on a marketing page is not the failure mode;
/// `854/1000` was.
#[test]
fn no_page_hand_types_a_number_beside_a_picture() {
    let root = repo_root();
    for page in PAGES {
        let text =
            std::fs::read_to_string(root.join(page)).unwrap_or_else(|e| panic!("{page}: {e}"));
        let figures = shot_figures(&text);
        assert_eq!(
            figures.len(),
            SHOTS.len(),
            "{page} shows {} of the {} pictures — a gate with nothing to \
             guard is vacuous",
            figures.len(),
            SHOTS.len()
        );
        for (alt, caption) in figures {
            for (what, prose) in [("alt text", &alt), ("caption", &caption)] {
                assert!(
                    !prose.chars().any(char::is_numeric),
                    "{page} hand-types a number in a picture's {what}: {prose:?}\n\
                     The picture carries its own numbers; the words beside it \
                     name what is shown."
                );
            }
        }
    }
}

/// Leg 4: the receipt names the schemas this code declares, and the
/// bytes actually shot.
///
/// Leg 1 alone would not have caught the bug that started this road.
/// The candidates screen showed `ce.join-report/0.1.0` through two
/// schema bumps, and `gui/ui` never had to change for that: the SHAPE
/// a screen renders can move underneath a picture while the rendering
/// code stands still. So the shoot leaves a receipt, read here against
/// the constants themselves — and against the files on disk, because a
/// receipt nothing binds to the pixels is three strings anyone can
/// edit green.
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
}

/// Leg 5: nothing the site serves is unnamed by a gate. An asset no
/// test mentions is precisely what these three pictures were for three
/// months — public, fact-bearing, and re-derivable by nobody.
///
/// Derived, not listed: a hand-kept ledger of owners is a comment that
/// reads like a check, and it would go stale the same way the pictures
/// did. The suite's own text is the evidence — but NOT this file's own
/// text. `SHOTS` is the carve-out, and it is a real one: whatever sits
/// in `SHOTS` is held by the four legs above, so naming it here IS
/// gating it. Every other asset must be named somewhere else in the
/// suite, or a stray filename in one of these comments would be enough
/// to declare a picture gated that nothing gates.
#[test]
fn every_asset_the_site_serves_is_named_by_a_gate() {
    let root = repo_root();
    let suite = root.join("cli/tests/it");
    let mut sources = Vec::new();
    common_rs(&suite, &mut sources);
    let corpus: String = sources
        .iter()
        .filter(|p| p.file_name().is_some_and(|n| n != "site_screenshots.rs"))
        .map(|p| std::fs::read_to_string(p).unwrap_or_default())
        .collect();

    let mut orphans = Vec::new();
    for entry in std::fs::read_dir(root.join("site/assets")).expect("site/assets") {
        let entry = entry.expect("a directory entry");
        if !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let gated_here = SHOTS.contains(&name.as_str());
        if name.starts_with('.') || !(gated_here || corpus.contains(&name)) {
            orphans.push(name);
        }
    }
    assert!(
        orphans.is_empty(),
        "the site serves assets no gate names: {orphans:?}\nAn asset here \
         must be re-derived or read by some test, or it is a claim on a \
         public page that nothing can check."
    );
}

/// Every `.rs` under the suite, recursively.
fn common_rs(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a suite directory").flatten() {
        let path = entry.path();
        if path.is_dir() {
            common_rs(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }
}
