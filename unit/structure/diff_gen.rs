//! The seeded cases of plan v2.33 W1 item 3's differential legs
//! (diff_rows.rs, diff_stale.rs, diff_arch.rs and the query
//! leg unit/query/facts/tree_diff.rs): a walked tree, a graph wire over
//! it — measured files, a stranger now and then, foreign readers, walked
//! assets, sections, packages, arcs of every kind — a `[structure]
//! layout`, line counts and `--impact` paths, each drawn from the case's
//! own stream so the legs can split the cases over threads (the loop that
//! holds the core to the frozen copies over them is diff_hold.rs).

use crate::graph::deadcode::GraphWire;
use crate::graph::nodes::Node;
use crate::graph::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The three independent seeds every seeded leg runs.
pub const SEEDS: [u64; 3] = [0x5eed_0001, 0x00c0_ffee_2033, 0x7777_dead_beef];

/// Cases per seed: `CE_I3_DIFF_N`, 10,000 unless set.
pub fn cases() -> usize {
    let n = std::env::var("CE_I3_DIFF_N").ok();
    n.and_then(|s| s.parse().ok()).unwrap_or(10_000)
}

/// Marsaglia's xorshift64, one stream per case.
pub struct Draw(u64);

impl Draw {
    /// Case `i` of `seed`'s run (the multiplier is the golden ratio's
    /// 64-bit fraction; the `| 1` keeps the state off xorshift's zero).
    pub fn case(seed: u64, i: usize) -> Draw {
        Draw((seed ^ (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)) | 1)
    }

    pub fn under(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % n.max(1) as u64) as usize
    }

    pub fn chance(&mut self, percent: usize) -> bool {
        self.under(100) < percent
    }

    /// One of a table's `|`-separated words.
    pub fn pick<'a>(&mut self, words: &'a str) -> &'a str {
        let n = words.split('|').count();
        words.split('|').nth(self.under(n)).unwrap_or("")
    }
}

/// Directory segments, file stems, extensions and whole names that set a
/// convention bit (README, the config files): every character class the
/// shape bits read, a dot, a dash, a capital, non-ASCII.
const SEGMENTS: &str = "src|lib|a|b|tests|docs|my-dir|Src|x_y|é|中文|.github|a.b|v2|deep|core";
const STEMS: &str =
    "main|mod|lib|parse_result|my-file|MyFile|x|__init__|index|util2|é|Ab_c|ZZ|a-b_c|q";
const EXTS: &str = "rs|py|md|toml|json|ts|go|hs|txt|c|h";
const WHOLE: &str =
    "README.md|Cargo.toml|package.json|ce.toml|go.mod|pyproject.toml|readme.md|LICENSE";

/// Code points (hex) a layout key draws, so the frozen side's `{:?}` of
/// it is the oracle for the core's `rustDebug`: first the 66 on which
/// rustc 1.94.1's `{:?}` and GHC 9.14.1's tables disagreed before the
/// table was measured (61 Other_Grapheme_Extend Mc / Lm / Lo, five emoji
/// modifiers), then a spread — CJK, combining and enclosing marks, two
/// Cc, two Zs, ZWNJ / ZWJ, a halfwidth sound mark, a tag, a Cn, a
/// noncharacter, a quote, a backslash, a tab, an emoji.
const ODD: &str = "9be|9d7|b3e|b57|bbe|bd7|cc0|cc2|cc7|cc8|cca|ccb|cd5|cd6|d3e|d57|dcf|ddf|1715|1734|1b35|1b3b|1b3d|1b43|1b44|1baa|1bf2|1bf3|302e|302f|a953|a9c0|111c0|11235|1133e|1134d|11357|113b8|113c2|113c5|113c7|113c8|113c9|113cf|114b0|114bd|115af|116b6|11930|1193d|11f41|16ff0|16ff1|1d165|1d166|1d16d|1d16e|1d16f|1d170|1d171|1d172|1f3fb|1f3fc|1f3fd|1f3fe|1f3ff|4e2d|6587|e9|301|302|20dd|7|1b|3000|a0|200c|200d|ff9e|e0041|378|fffe|22|5c|9|1f600";

/// One to three of ODD's code points.
fn odd(d: &mut Draw) -> String {
    (0..1 + d.under(3))
        .filter_map(|_| u32::from_str_radix(d.pick(ODD), 16).ok())
        .filter_map(char::from_u32)
        .collect()
}

/// One path: up to three directories over a name.
pub fn path(d: &mut Draw) -> String {
    let dirs: Vec<&str> = (0..d.under(4)).map(|_| d.pick(SEGMENTS)).collect();
    let name = if d.chance(15) {
        d.pick(WHOLE).to_string()
    } else if d.chance(10) {
        d.pick(STEMS).to_string()
    } else {
        format!("{}.{}", d.pick(STEMS), d.pick(EXTS))
    };
    dirs.into_iter()
        .chain([name.as_str()])
        .collect::<Vec<_>>()
        .join("/")
}

/// A directory of a path (`""` at the root).
fn dir_of(p: &str) -> String {
    p.rsplit_once('/')
        .map_or(String::new(), |(d, _)| d.to_string())
}

/// One case: the walked paths in walk order and the graph wire over
/// them; `universe` (the stale leg's committed files) replaces the drawn
/// paths when given.
pub struct Case {
    pub walked: Vec<String>,
    pub w: GraphWire,
    pub layout: BTreeMap<String, u32>,
    pub focus: Vec<String>,
}

pub fn case(d: &mut Draw, universe: Option<&[String]>) -> Case {
    let mut pool = BTreeSet::new();
    for _ in 0..1 + d.under(30) {
        pool.insert(match universe {
            Some(u) => u[d.under(u.len())].clone(),
            None => path(d),
        });
    }
    let mut walked: Vec<String> = pool.into_iter().collect();
    for i in (1..walked.len()).rev() {
        walked.swap(i, d.under(i + 1));
    }
    let nodes = nodes(d, &walked);
    let w = wire(d, nodes);
    let layout = layout(d, &walked);
    let measured = crate::graph::deadcode::measured_nodes(&w);
    let mut focus: Vec<String> = (0..d.under(4))
        .filter_map(|_| {
            measured
                .get(d.under(measured.len().max(1)))
                .map(|m| m.1.to_string())
        })
        .collect();
    if d.chance(5) {
        focus.push(path(d));
    }
    Case {
        walked,
        w,
        layout,
        focus,
    }
}

/// The graph's nodes: most walked files measured (a stranger now and
/// then — the fault road), foreign readers and walked assets anywhere,
/// sections on files, packages at directories in the tree and outside.
fn nodes(d: &mut Draw, walked: &[String]) -> Vec<Node> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    let mut file = |p: String, foreign: bool, asset: bool, out: &mut Vec<Node>| {
        if seen.insert(p.clone()) {
            let mut n = crate::testutil::node(&p, "", GRAN_FILE);
            (n.foreign, n.asset) = (foreign, asset);
            out.push(n);
        }
    };
    for p in walked.iter().filter(|_| d.chance(75)) {
        file(p.clone(), false, false, &mut out);
    }
    for k in 0..d.under(5) {
        let (stranger, foreign) = (k == 0 && d.chance(8), d.chance(50));
        let p = if stranger || d.chance(40) {
            path(d)
        } else {
            walked[d.under(walked.len())].clone()
        };
        file(p, !stranger && foreign, !stranger && !foreign, &mut out);
    }
    for _ in 0..d.under(4) {
        let p = walked[d.under(walked.len())].clone();
        out.push(crate::testutil::node(&p, d.pick(STEMS), GRAN_SECTION));
    }
    for _ in 0..d.under(4) {
        let p = if d.chance(70) {
            dir_of(&walked[d.under(walked.len())])
        } else {
            d.pick(SEGMENTS).to_string()
        };
        out.push(crate::testutil::node(&p, "", GRAN_PACKAGE));
    }
    for i in (1..out.len()).rev() {
        out.swap(i, d.under(i + 1));
    }
    out
}

/// The wire: a node row per node (the role bits deadcode/flags.rs sends —
/// foreign 1 << 7, asset 1 << 9, an entry 1 on some files) and arcs of
/// every kind and rung between any two nodes.
fn wire(d: &mut Draw, nodes: Vec<Node>) -> GraphWire {
    let rows = (nodes.iter())
        .map(|n| {
            let lang = crate::scan::lang::Lang::from_path(std::path::Path::new(&n.path))
                .map_or(crate::scan::lang::Lang::LangUnknown as i64, |l| l as i64);
            let roles = match (n.foreign, n.asset) {
                (true, _) => 1 << 7,
                (_, true) => 1 << 9,
                _ if n.kind == GRAN_FILE => i64::from(d.chance(30)),
                _ => 0,
            };
            json!([lang, n.kind, roles])
        })
        .collect();
    let n = nodes.len().max(1);
    let edges = (0..d.under(40))
        .map(|_| {
            [
                d.under(n) as i64,
                d.under(n) as i64,
                d.under(6) as i64,
                1 + d.under(3) as i64,
            ]
        })
        .filter(|e| (e[0] as usize) < nodes.len())
        .collect();
    GraphWire {
        nodes,
        rows,
        edges,
        unresolved_sites: 0,
        unres: Vec::new(),
        symbols: BTreeSet::new(),
        unmentioned: None,
        mounts: None,
        scc_floor: None,
    }
}

/// A `[structure] layout`: walked directories, with and without a
/// trailing `/`, the root as `.`, and now and then a directory the tree
/// does not hold — drawn paths, and walked directories with a run of
/// ODD's code points appended (the fault spells the key by `{:?}`).
fn layout(d: &mut Draw, walked: &[String]) -> BTreeMap<String, u32> {
    (0..d.under(4))
        .map(|_| {
            let dir = dir_of(&walked[d.under(walked.len())]);
            let key = match d.under(7) {
                0 => ".".to_string(),
                1 => format!("{}/", path(d)),
                6 => format!("{dir}{}", odd(d)),
                2 => format!("{dir}/"),
                _ if dir.is_empty() => "./".to_string(),
                _ => dir,
            };
            (key, d.under(10) as u32)
        })
        .collect()
}

/// A wire value's numbers as one kind (the oracle's u64 rows and the
/// core's answers compare as JSON).
pub fn plain(v: impl serde::Serialize) -> Value {
    serde_json::to_value(v).expect("a JSON value")
}
