use super::*;
use std::path::PathBuf;

/// A scratch root holding exactly `files`. The in-scope set is
/// minted apart from it (`live_of`): a probe's live names and its
/// on-disk files deliberately differ — a .cts source with no file
/// is still in scope, and the twin beside it is the fact tested.
fn fixture(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = crate::testutil::scratch(tag);
    crate::testutil::write_tree(&root, files);
    root
}

fn live_of(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

/// The phase-2 key as the sweep computes it — the twin, node_modules
/// and extends facts are INPUTS to it, never a separate answer.
/// Spelling that join out at all four read sites was this batch's
/// clone (dedup gate).
fn key(root: &Path, live: &BTreeSet<String>, configs: &[(String, u64)]) -> i64 {
    resolve_key(live, &ts_fs_facts(root, live, configs))
}

/// Apply `mutate` to the root and assert the key moves — the shared
/// tail of every probe here (the dedup gate refused the second copy).
fn refires(
    root: &Path,
    live: &BTreeSet<String>,
    configs: &[(String, u64)],
    mutate: impl FnOnce(&Path),
    why: &str,
) {
    let before = key(root, live, configs);
    mutate(root);
    assert_ne!(before, key(root, live, configs), "{why}");
    std::fs::remove_dir_all(root).expect("cleanup");
}

/// Write `twin` beside the source — the twin-table probes' mutation.
fn twin_refires(root: &Path, live: &BTreeSet<String>, twin: &str, body: &str) {
    let write = |root: &Path| std::fs::write(root.join(twin), body).expect("write twin");
    let why = format!("a {twin} twin must re-fire the sweep");
    refires(root, live, &[], write, &why);
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

/// The .cts row of the same table; and the twin list is per-source
/// exact — a stray .mjs next to a .cts is not one of R2's probes,
/// so it must not be collected under the .cts source.
#[test]
fn cts_takes_cjs_twin_only() {
    let root = fixture("keys-cts-twin", &[("b.mjs", "export {}")]);
    let bare = fixture("keys-cts-bare", &[]);
    let live = live_of(&["b.cts"]);
    assert_eq!(
        key(&root, &live, &[]),
        key(&bare, &live, &[]),
        "a .mjs is not a .cts probe"
    );
    std::fs::remove_dir_all(&bare).expect("cleanup");
    twin_refires(&root, &live, "b.cjs", "module.exports={}");
}

/// A tsconfig's `extends` base under any name (step 5b): the walk
/// lists only the tsconfig-named files as configs, so a base named
/// otherwise is hashed as a fact — an edit to it must move the key, or
/// the paths rung would keep answering from the old base.
#[test]
fn an_extends_base_under_any_name_moves_the_key() {
    let root = fixture(
        "keys-extends",
        &[
            ("tsconfig.json", "{\"extends\": \"./base.json\"}\n"),
            (
                "base.json",
                "{\"compilerOptions\": {\"baseUrl\": \"./a\"}}\n",
            ),
        ],
    );
    let live = live_of(&["index.ts"]);
    let configs = [("tsconfig.json".to_string(), 1)];
    let rewrite = |root: &Path| {
        let body = "{\"compilerOptions\": {\"baseUrl\": \"./b\"}}\n";
        std::fs::write(root.join("base.json"), body).expect("rewrite base");
    };
    refires(
        &root,
        &live,
        &configs,
        rewrite,
        "an edited extends base must re-fire the sweep",
    );
}

/// A node_modules under any ancestor of a live TS file (step 5b): the
/// bare rung probes every ancestor's node_modules, so a package
/// appearing in a nested one must move the key.
#[test]
fn node_modules_under_an_ancestor_directory_moves_the_key() {
    let root = fixture(
        "keys-nested-nm",
        &[("packages/app/src/index.ts", "export {}")],
    );
    let live = live_of(&["packages/app/src/index.ts"]);
    let vendor = |root: &Path| {
        let dir = root.join("packages/node_modules/vend");
        std::fs::create_dir_all(&dir).expect("vendor dir");
        std::fs::write(dir.join("index.js"), "// vendored").expect("vendor file");
    };
    refires(
        &root,
        &live,
        &[],
        vendor,
        "a nested node_modules must re-fire the sweep",
    );
}
