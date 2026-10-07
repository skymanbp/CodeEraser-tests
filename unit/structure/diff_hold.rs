//! The one loop every item-3 differential leg holds the core to the
//! frozen copies with (plan v2.33 W1 item 3): a seed's cases split over
//! threads, each with its own core link, every (want, got) compared and
//! every answer tallied by kind; the three seeds run by one call; and the
//! real-tree legs' core, link and roots.

use super::diff_gen::{Draw, SEEDS, cases};
use crate::corelink::Link;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// What a seed's run found: the first five mismatches and their count,
/// and how many cases the oracle answered with a fault or the core
/// refused (a leg whose cases all fault, or none do, proves less than it
/// says) — by the answer's first two words, printed with the run.
pub struct Held {
    pub bad: Vec<String>,
    pub faults: usize,
}

/// An answer's kind: `tables`, or a fault's or refusal's first two words.
fn kind(want: &Value) -> String {
    let text = want
        .get("fault")
        .or_else(|| want.get("refused"))
        .and_then(Value::as_str);
    let words = |t: &str| match t.split(' ').next() {
        Some(w) if w.ends_with(':') => w.to_string(),
        _ => t.split(' ').take(2).collect::<Vec<_>>().join(" "),
    };
    text.map_or("tables".into(), words)
}

/// Every case of `seed` (`cases()` of them, split over four threads, each
/// with its own core link) held as `want == got`. `one` answers (want,
/// got) for case `i`; a `want` carrying a `fault` key is a fault case.
pub fn hold<F>(seed: u64, one: F) -> Held
where
    F: Fn(&mut Link, &mut Draw) -> (Value, Value) + Sync,
{
    let core = crate::daemon::judge::core_bin().expect("CE_CORE_BIN or a core on PATH");
    let n = cases();
    let threads = std::env::var("CE_I3_DIFF_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let found: Vec<(Vec<String>, BTreeMap<String, usize>)> = std::thread::scope(|s| {
        let legs: Vec<_> = (0..threads)
            .map(|t| {
                let (core, one) = (&core, &one);
                s.spawn(move || {
                    let mut link = crate::document::open(core).expect("a core link");
                    let (mut bad, mut kinds) = (Vec::new(), BTreeMap::new());
                    for i in (t..n).step_by(threads) {
                        let (want, got) = one(&mut link, &mut Draw::case(seed, i));
                        *kinds.entry(kind(&want)).or_insert(0) += 1;
                        if want != got {
                            bad.push(format!("case {i}\n  want {want}\n  got  {got}"));
                        }
                    }
                    (bad, kinds)
                })
            })
            .collect();
        legs.into_iter().map(|h| h.join().expect("a leg")).collect()
    });
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for (k, c) in found.iter().flat_map(|f| f.1.iter()) {
        *kinds.entry(k.clone()).or_insert(0) += c;
    }
    let faults = n - kinds.get("tables").copied().unwrap_or(0);
    let all: Vec<String> = found.into_iter().flat_map(|f| f.0).collect();
    let mut bad: Vec<String> = all.iter().take(5).cloned().collect();
    if !all.is_empty() {
        bad.push(format!("seed {seed:#x}: {} of {n} cases differ", all.len()));
    }
    println!(
        "seed {seed:#x}: {n} cases, {} differ; answers {kinds:?}",
        all.len()
    );
    Held { bad, faults }
}

/// Every seed of SEEDS held with `one`: no case differs, and — when
/// `faulting` — some but under half the cases are faults.
pub fn every_seed<F>(one: F, faulting: bool)
where
    F: Fn(&mut Link, &mut Draw) -> (Value, Value) + Sync,
{
    for seed in SEEDS {
        let held = hold(seed, &one);
        assert!(held.bad.is_empty(), "{}", held.bad.join("\n"));
        assert!(
            !faulting || (held.faults > 0 && held.faults < cases() / 2),
            "seed {seed:#x}: {} faults",
            held.faults
        );
    }
}

/// The real-tree legs' core, a link to it, and the roots named in
/// `CE_I3_REAL_ROOTS` (`;`-separated).
pub fn real_roots() -> (String, Link, Vec<PathBuf>) {
    let core = crate::daemon::judge::core_bin().expect("a core");
    let roots = std::env::var("CE_I3_REAL_ROOTS").expect("CE_I3_REAL_ROOTS");
    let link = crate::document::open(&core).expect("a core link");
    let roots = roots
        .split(';')
        .filter(|r| !r.is_empty())
        .map(PathBuf::from)
        .collect();
    (core, link, roots)
}
