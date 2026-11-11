//! Stale candidates are re-read (v2.35 guard lane, GD5). The probe used
//! to extend each indexed fingerprint at its INDEXED token offset in
//! the candidate's current stream: a candidate edited above the region
//! since the last refresh failed that check and the twin went unseen
//! until the Stop audit. Now the index selects candidate files only and
//! each candidate is matched by the fingerprints of its current stream.
//! On an unchanged file those ARE the indexed fingerprints — the last
//! leg holds that over a real tree, so a fresh index answers exactly as
//! before.

use crate::common::{self, repo_root, tmp, write_all};
use codeeraser::dedup::{Params, index::Index, tokens, winnow};
use codeeraser::scan::lang::Lang;
use std::collections::BTreeMap;

const F: &str = "def stale_me(rows, key, limit):\n    seen = {}\n    for row in rows:\n        k = row.get(key)\n        if k is None:\n            continue\n        bucket = seen.setdefault(k, [])\n        if len(bucket) >= limit:\n            raise ValueError(\"too many rows for %s\" % k)\n        bucket.append(row)\n    total = sum(len(v) for v in seen.values())\n    return sorted(seen.items(), key=lambda kv: (len(kv[1]), kv[0])), total\n";
const H: &str = "def head_one(a, b, c):\n    out = []\n    for i in range(a):\n        if i % b == 0:\n            out.append((i, c * i))\n        elif i % 7 == 3:\n            out.extend([i, i + 1, i + 2])\n        else:\n            out.insert(0, str(i) + \"-\" + str(c))\n    return {k: v for k, v in enumerate(out)}\n";

fn padding(n: usize) -> String {
    (0..n).map(|i| format!("CONST_{i} = {i}\n")).collect()
}

/// The repro: b.py indexed, then twelve lines inserted above its
/// function on disk; a copy of the function written elsewhere is
/// refused, naming the function where it now sits.
#[test]
fn a_candidate_edited_above_the_region_is_still_matched() {
    let dir = tmp("guard-stale-above");
    write_all(&dir, &[("b.py", F)]);
    common::build_index(&dir);
    write_all(&dir, &[("b.py", &format!("{}\n{F}", padding(12)))]);
    let reason = common::expect_write_denied(&dir, "c.py", F, "b.py:14-");
    assert!(reason.contains("1 indexed region(s)"), "{reason}");
}

/// A candidate edited in the MIDDLE: its region above the edit still
/// sits at its indexed offset, the one below has moved. A copy of both
/// regions is matched against both.
#[test]
fn a_candidate_edited_in_the_middle_is_matched_on_both_sides() {
    let dir = tmp("guard-stale-middle");
    write_all(&dir, &[("b.py", &format!("{H}\n\n{F}"))]);
    common::build_index(&dir);
    write_all(&dir, &[("b.py", &format!("{H}\n{}\n{F}", padding(9)))]);
    let both = format!("{H}\n\n{F}");
    let reason = common::expect_write_denied(&dir, "c.py", &both, "indexed region(s)");
    assert!(
        reason.contains("b.py:1-") && reason.contains("b.py:22-"),
        "{reason}"
    );
}

/// Over a real tree with a fresh index, every own file's current
/// fingerprints are its indexed ones, hash and offset alike — the
/// probe's two inputs agree, so its match list is the one it was.
#[test]
fn a_fresh_index_holds_exactly_the_current_fingerprints() {
    let dir = tmp("guard-stale-fresh");
    let src = repo_root().join("cli/src/dedup");
    for e in std::fs::read_dir(&src).expect("cli/src/dedup").flatten() {
        if e.path().extension().is_some_and(|x| x == "rs") {
            std::fs::copy(e.path(), dir.join(e.file_name())).expect("copy");
        }
    }
    common::build_index(&dir);
    let p = Params::default();
    let idx = Index::open(&dir.join(".ce/index.db"), p).expect("index");
    let mut indexed: BTreeMap<String, Vec<(u64, usize)>> = BTreeMap::new();
    for i in idx.all_instances().expect("instances") {
        indexed
            .entry(i.file)
            .or_default()
            .push((i.hash, i.start_tok));
    }
    assert!(indexed.len() > 10, "a real tree: {} files", indexed.len());
    for (file, mut rows) in indexed {
        let bytes = std::fs::read(dir.join(&file)).expect("read");
        let hashes: Vec<u64> = tokens::stream(&bytes, Lang::Rust)
            .expect("tokens")
            .iter()
            .map(|t| t.hash)
            .collect();
        let mut now: Vec<(u64, usize)> = winnow::fingerprints(&hashes, p)
            .iter()
            .map(|f| (f.hash, f.start))
            .collect();
        rows.sort();
        now.sort();
        assert_eq!(rows, now, "{file}");
    }
}
