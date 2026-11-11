use super::*;
use crate::dedup::Params;
use crate::dedup::index::Index;
use crate::testutil::scratch;
use std::path::PathBuf;

fn tree(lab: &[u64], lld: &[i64]) -> UnitTree {
    UnitTree {
        lab: lab.to_vec(),
        lld: lld.to_vec(),
        ..Default::default()
    }
}

/// The key is the judge's input and nothing else: equal (lab, lld)
/// share it; a relabel, a reshape under the same labels and a longer
/// tree each move it.
#[test]
fn the_key_reads_labels_and_shape_alone() {
    let a = tree_key(&tree(&[1, 2, 3], &[0, 1, 0]));
    assert_eq!(a, tree_key(&tree(&[1, 2, 3], &[0, 1, 0])));
    let moved = [
        tree(&[1, 2, 4], &[0, 1, 0]),
        tree(&[1, 2, 3], &[0, 0, 0]),
        tree(&[1, 2, 3, 3], &[0, 1, 0, 0]),
    ];
    assert!(
        moved.iter().all(|t| tree_key(t) != a),
        "a relabel, a reshape, a longer tree"
    );
}

/// One slot per unordered tree pair: the two readers of (0, 1) and
/// (1, 0) find the same row with the sizes in slot order, exactly one
/// of them swapped; a forest carries no key.
#[test]
fn a_pair_reads_one_slot_from_either_side() {
    let built = vec![
        Outcome::Tree(tree(&[9, 9], &[0, 0])),
        Outcome::Tree(tree(&[1], &[0])),
        Outcome::Forest,
    ];
    let keys = Keys::of(&built);
    let (s01, sw01, n01) = keys.slot(0, 1);
    let (s10, sw10, n10) = keys.slot(1, 0);
    assert_eq!((s01, n01), (s10, n10));
    assert_ne!(sw01, sw10);
    assert_eq!(keys.live().len(), 2);
}

/// A fresh index under `tag` at generation 7.2.0, three trees and the
/// two pairs (0, 1) and (1, 2) over them; the core's answer for them
/// never changes — (0, 1) scored, (1, 2) no row (Below).
struct Seed {
    root: PathBuf,
    idx: Index,
    keys: Keys,
    pairs: [PairRow; 2],
}

const SCORED: [(usize, usize, ScoredTed); 1] = [(0, 1, (1, 2, 3, true))];

fn seeded(tag: &str) -> Seed {
    let root = scratch(tag);
    std::fs::create_dir_all(root.join(".ce")).unwrap();
    let idx = Index::open(&root.join(".ce/index.db"), Params::default()).unwrap();
    open_generation(idx.raw(), "7.2.0").unwrap();
    let built = vec![
        Outcome::Tree(tree(&[1, 2], &[0, 0])),
        Outcome::Tree(tree(&[3, 4, 5], &[0, 1, 0])),
        Outcome::Tree(tree(&[6], &[0])),
    ];
    let pair = |a, b| PairRow { a, b, sources: 1 };
    Seed {
        root,
        idx,
        keys: Keys::of(&built),
        pairs: [pair(0, 1), pair(1, 2)],
    }
}

impl Seed {
    fn conn(&self) -> &Connection {
        self.idx.raw()
    }

    fn sendable(&self) -> Vec<&PairRow> {
        self.pairs.iter().collect()
    }

    /// Rows held right now.
    fn held(&self) -> usize {
        load(self.conn()).unwrap().len()
    }

    /// Remember the core's answer for `sent` over what the table holds,
    /// every built tree live — the shape judge.rs calls it in: `put` for
    /// the answered request, `sweep` once at the end.
    fn remember(&self, sent: &[&PairRow]) {
        let held = load(self.conn()).unwrap();
        put(self.conn(), &fresh(&self.keys, sent, &SCORED)).unwrap();
        sweep(self.conn(), &self.keys.live(), &held).unwrap();
    }

    fn done(self) {
        drop(self.idx);
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Cold: nothing replays. Remembered: the scored row comes back with
/// the core's bit in the reader's own orientation and the Below pair
/// as no row, both counted replayed.
#[test]
fn a_remembered_run_replays_scored_and_below_alike() {
    let s = seeded("t3cache-roundtrip");
    let (rows, send, replayed) = replay(&Table::new(), &s.keys, &s.sendable());
    assert!(rows.is_empty() && replayed == 0 && send.len() == 2, "cold");
    s.remember(&send);
    let held = load(s.conn()).unwrap();
    let (rows, send, replayed) = replay(&held, &s.keys, &s.sendable());
    assert_eq!(
        (rows, send.len(), replayed),
        (SCORED.to_vec(), 0, 2),
        "warm"
    );
    s.done();
}

/// Another proto empties the table before a row is read; a sweep
/// drops the rows whose trees left and keeps the rest.
#[test]
fn another_generation_empties_and_a_sweep_drops_the_departed() {
    let s = seeded("t3cache-generation");
    s.remember(&s.sendable());
    assert_eq!(s.held(), 2, "both slots held");
    open_generation(s.conn(), "8.0.0").unwrap();
    assert_eq!(s.held(), 0, "another proto empties the table");
    s.remember(&s.sendable());
    let mut live = s.keys.live();
    live.remove(&tree_key(&tree(&[6], &[0])));
    sweep(s.conn(), &live, &load(s.conn()).unwrap()).unwrap();
    assert_eq!(s.held(), 1, "the row whose tree left is swept");
    s.done();
}

/// A 200-node chain whose first label is `t`'s own: every tree its own
/// key, so every pair its own slot.
fn chain(t: u64) -> UnitTree {
    let lab: Vec<u64> = (0..200)
        .map(|i| if i == 0 { 100 + t } else { i % 5 })
        .collect();
    tree(&lab, &[0; 200])
}

/// Plan v2.35 C3 and C4, over the product's own judge and a real core:
/// the pairs are planned by work into several requests, the last one
/// holding the only pair of tree `g`. Built with tree `g` malformed (a
/// label too many), the core refuses that request by name — after the
/// first request was answered and remembered. Rebuilt well-formed, the
/// next judgment replays the first request's pairs and sends only the
/// rest.
/// The leg's pairs over trees 0 ..= g: every pair of 0 .. g, then the
/// only pair of g, (g − 1, g), last in the ascending order the candidate
/// pass hands over; `g` the least count whose pairs pass one request's
/// work. Returns g, the pairs, and how many pairs ride before the last
/// request of the judge's own plan over the malformed tree g (201 nodes).
fn interrupted_plan(fam: &crate::lockstep::Family<'_>) -> (usize, Vec<PairRow>, u64) {
    let per_chunk = (limits().work_cap / (200 * 200)) as usize;
    let g = (2..).find(|g| g * (g - 1) / 2 > per_chunk).unwrap();
    let pair = |a, b| PairRow { a, b, sources: 1 };
    let mut pairs: Vec<PairRow> = (0..g)
        .flat_map(|a| (a + 1..g).map(move |b| pair(a, b)))
        .collect();
    pairs.push(pair(g - 1, g));
    let send: Vec<&PairRow> = pairs.iter().collect();
    let size = |t: usize| if t == g { 201 } else { 200 };
    let chunks = crate::lockstep::by_work(fam, &send, limits().work_cap, |p| size(p.a) * size(p.b));
    let last = chunks.last().expect("a plan");
    assert!(
        chunks.len() >= 2 && last.iter().any(|p| p.b == g),
        "tree {g} rides the last request"
    );
    let first = (send.len() - last.len()) as u64;
    (g, pairs, first)
}

#[test]
fn an_answered_request_is_remembered_before_the_next_is_asked() {
    let core = crate::daemon::judge::core_bin().expect("a core");
    let fam = super::super::wire::family(&core);
    let (g, pairs, first) = interrupted_plan(&fam);
    let send: Vec<&PairRow> = pairs.iter().collect();
    let built = |g_tree: UnitTree| -> Vec<Outcome> {
        (0..g as u64)
            .map(|t| Outcome::Tree(chain(t)))
            .chain([Outcome::Tree(g_tree)])
            .collect()
    };
    let mut bad = chain(g as u64);
    bad.lab.push(0);
    let root = scratch("t3cache-interrupted");
    std::fs::create_dir_all(root.join(".ce")).unwrap();
    let idx = Index::open(&root.join(".ce/index.db"), Params::default()).unwrap();
    let open = || crate::corelink::judged::open(fam.core, fam.cap).expect("open core");
    let cut = super::super::judge::judge(open(), &core, idx.raw(), &built(bad), &send);
    let refused = cut
        .err()
        .expect("the request holding the malformed tree is refused")
        .to_string();
    assert!(refused.contains("lab/lld length mismatch"), "{refused}");
    assert_eq!(
        load(idx.raw()).unwrap().len() as u64,
        first,
        "the first request's pairs are held"
    );
    let (_, [judged, prefiltered, cached], _, _) =
        super::super::judge::judge(open(), &core, idx.raw(), &built(chain(g as u64)), &send)
            .expect("judge");
    assert_eq!(cached, first, "the answered request is replayed");
    assert_eq!(
        judged + prefiltered,
        pairs.len() as u64 - first,
        "only the rest is sent"
    );
    drop(idx);
    let _ = std::fs::remove_dir_all(&root);
}
