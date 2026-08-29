use super::*;
use std::path::PathBuf;

/// A scratch root holding exactly `files`. The in-scope set is
/// minted apart from it (`live_of`): a probe's live names and its
/// on-disk files deliberately differ — a .cts source with no file
/// is still in scope, and the twin beside it is the fact tested.
fn fixture(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = crate::testutil::scratch(tag);
    for (name, body) in files {
        std::fs::write(root.join(name), body).expect("fixture write");
    }
    root
}

fn live_of(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

/// The phase-2 key as the sweep computes it — the twin and
/// node_modules facts are INPUTS to it, never a separate answer.
/// Spelling that join out at all four read sites was this batch's
/// clone (dedup gate).
fn key(root: &Path, live: &BTreeSet<String>) -> i64 {
    resolve_key(live, &ts_fs_facts(root, live))
}

/// R2 stats the .mjs twin of an in-scope .mts source, so its
/// appearance must move the key. While ts_fs_facts only stripped
/// .ts/.tsx, .mts/.cts sources contributed no twin fact at all and
/// the rewrite verdict froze at whatever the first sweep saw.
#[test]
fn mjs_twin_beside_mts_moves_the_key() {
    let root = fixture("keys-mts-twin", &[("a.mts", "export {}")]);
    let live = live_of(&["a.mts"]);
    twin_refires(&root, &live, "a.mjs", "export {}");
}

/// Write `twin` and assert the key moves — the shared tail of both
/// twin-table probes (the dedup gate refused the second copy).
fn twin_refires(root: &Path, live: &std::collections::BTreeSet<String>, twin: &str, body: &str) {
    let before = key(root, live);
    std::fs::write(root.join(twin), body).expect("write twin");
    assert_ne!(
        before,
        key(root, live),
        "a {twin} twin must re-fire the sweep"
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

/// The .cts row of the same table; and the twin list is per-source
/// exact — a stray .mjs next to a .cts is not one of R2's probes,
/// so it must not be collected under the .cts source.
#[test]
fn cts_takes_cjs_twin_only() {
    let root = fixture("keys-cts-twin", &[("b.mjs", "export {}")]);
    let bare = fixture("keys-cts-bare", &[]);
    let live = live_of(&["b.cts"]);
    assert_eq!(
        key(&root, &live),
        key(&bare, &live),
        "a .mjs is not a .cts probe"
    );
    std::fs::remove_dir_all(&bare).expect("cleanup");
    twin_refires(&root, &live, "b.cjs", "module.exports={}");
}
