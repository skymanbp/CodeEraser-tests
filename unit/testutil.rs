//! Unit-test scratch directories (compiled under cfg(test) only):
//! the one tmp-dir scaffold the in-crate test modules share — the
//! dedup ratchet caught join::churn_unit growing a second copy of
//! deadcode's observe-test setup, so the scaffold became a throat.

use std::path::{Path, PathBuf};

/// A family's schema id as the core's document catalogue states it
/// (audit C row 18), asked of the core once: the unit legs read the
/// id the package carries rather than restate it.
pub fn catalogue_schema(family: &str) -> serde_json::Value {
    static TABLES: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    let tables = TABLES.get_or_init(|| {
        let core = crate::daemon::judge::core_bin().expect("a core");
        let (mut link, _) = crate::corelink::Link::open(&core).expect("the core");
        link.request("tables", serde_json::json!({}))
            .expect("tables/1")
    });
    tables["document"][family]["schema"].clone()
}

/// A literal case table, one `input => expected` per line, read and
/// counted (the count catches a line the separator missed).
pub fn case_table(table: &str, n: usize) -> Vec<(&str, &str)> {
    let cases: Vec<(&str, &str)> = table
        .trim()
        .lines()
        .map(|l| l.split_once(" => ").expect("a case line"))
        .collect();
    assert_eq!(cases.len(), n, "every case line reads");
    cases
}

/// One case of a refusal table: `ok` expects an answer (handed back),
/// anything else is text the refusal must contain.
pub fn holds<T>(case: &str, says: &str, got: anyhow::Result<T>) -> Option<T> {
    match (says, got) {
        ("ok", Ok(t)) => Some(t),
        ("ok", Err(e)) => panic!("{case} refused: {e:#}"),
        (_, Ok(_)) => panic!("{case} accepted; expected: {says}"),
        (_, Err(e)) => {
            assert!(format!("{e:#}").contains(says), "{case}: {e:#}");
            None
        }
    }
}

/// Fresh empty scratch dir keyed by tag + pid (parallel-test safe);
/// callers remove it themselves when the assertion needs a clean end.
pub fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ce-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// A scratch tree from (relative path, text) pairs, parents made —
/// the manifest-and-file setup the target and mount facts read from
/// disk, written once here rather than as a per-test mkdir/write run
/// (the clone gate's next-most-rhyming shape after the tuple table).
pub fn write_tree(root: &Path, files: &[(&str, &str)]) {
    for (rel, text) in files {
        let at = root.join(rel);
        at.parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&at, text))
            .unwrap_or_else(|e| panic!("{rel}: {e}"));
    }
}

/// A case table written as ONE literal, as (language, the N header
/// columns after the extension, the body): blocks separated by a `====`
/// line, each a header `ext @@ column @@ …` over its body. One literal
/// rather than a slice of typed rows: rows of any one tuple or struct
/// shape repeat every dozen tokens, and the clone gate reads such a
/// table as clones of itself. The column count is the table's shape,
/// so it is checked here once — each caller destructures its columns
/// in the loop pattern, where every caller used to convert them.
pub fn blocks<const N: usize>(
    table: &str,
) -> impl Iterator<Item = (crate::scan::lang::Lang, [&str; N], &str)> {
    sections(table).map(|(head, body)| {
        let mut cols = head.split(" @@ ");
        let ext = cols.next().expect("an extension column");
        let lang = crate::scan::lang::Lang::from_path(Path::new(&format!("x.{ext}")))
            .expect("a known extension");
        let cols: Vec<&str> = cols.collect();
        let cols = <[&str; N]>::try_from(cols)
            .unwrap_or_else(|c| panic!("{N} columns after the extension, not {}: {head}", c.len()));
        (lang, cols, body)
    })
}

/// A table's `====`-separated blocks as (header line, body): what
/// `blocks` reads with a language column, for the tables whose blocks
/// name no language (the C-family reader tables).
pub fn sections(table: &str) -> impl Iterator<Item = (&str, &str)> {
    table
        .trim()
        .split("\n====\n")
        .map(|block| block.split_once('\n').expect("a header line over a body"))
}

/// The visible markers a single-literal table spells control bytes
/// with — `{LF}` `{CR}` `{TAB}` `{NUL}` `{BOM}` — expanded, so a row
/// keeps its own backslashes literal (the reader tables of cmdline.rs,
/// compdb.rs and c_head.rs). One table over a fold rather than a chain
/// of `replace` calls: the clone gate read that chain as the (since
/// retired) bench dashboard's HTML escaper, and the write guard
/// refused it.
pub fn unmarked(text: &str) -> String {
    const MARKS: [(&str, &str); 5] = [
        ("{LF}", "\n"),
        ("{CR}", "\r"),
        ("{TAB}", "\t"),
        ("{NUL}", "\0"),
        ("{BOM}", "\u{feff}"),
    ];
    MARKS
        .iter()
        .fold(text.to_string(), |s, (mark, byte)| s.replace(*mark, byte))
}

/// A node literal for wire-side tests.
pub fn node(path: &str, unit: &str, kind: i64) -> crate::graph::nodes::Node {
    crate::graph::nodes::Node {
        path: path.into(),
        unit: unit.into(),
        kind,
        foreign: false,
        asset: false,
    }
}

/// The graph wire the canvas and deadcode-document tests share: two
/// files, a section of a.rs and a package (node ids 0..3), over
/// `edges`, seven unresolved sites, and none of the export surface or
/// advisory tables — neither test reads them.
pub fn four_node_wire(edges: &[[i64; 4]]) -> crate::graph::deadcode::GraphWire {
    use crate::graph::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
    crate::graph::deadcode::GraphWire {
        nodes: vec![
            node("a.rs", "", GRAN_FILE),
            node("b.rs", "", GRAN_FILE),
            node("a.rs", "Intro", GRAN_SECTION),
            node("pkg", "", GRAN_PACKAGE),
        ],
        rows: vec![],
        edges: edges.iter().copied().collect(),
        unresolved_sites: 7,
        unres: vec![],
        symbols: Default::default(),
        unmentioned: None,
        mounts: None,
        scc_floor: None,
        gate: None,
    }
}

/// A similar-corpus seat whose bag holds each word once on the Name
/// channel — the advisor tests' unit (ppmi, rank).
pub fn word_doc(path: &str, words: &[&str]) -> crate::similar::corpus::Doc {
    use crate::similar::{Channel, frozen::word_term};
    let terms = words
        .iter()
        .map(|w| (word_term(Channel::Name, w), (Channel::Name, 1)))
        .collect();
    crate::similar::corpus::Doc {
        path: path.into(),
        bag: crate::similar::bag::UnitBag {
            key: words.join("_"),
            nth: 0,
            start_line: 1,
            end_line: 1,
            terms,
        },
    }
}

/// `fetch` and `load` together in four seats, `user` once beside them
/// — the co-occurrence corpus both advisor tables are tested on.
pub fn fetch_load_docs() -> Vec<crate::similar::corpus::Doc> {
    [
        ("a.rs", &["fetch", "load", "user"][..]),
        ("b.rs", &["fetch", "load"]),
        ("c.rs", &["fetch", "load"]),
        ("d.rs", &["fetch", "load"]),
    ]
    .into_iter()
    .map(|(path, words)| word_doc(path, words))
    .collect()
}

/// One per-declaration word case as one line: the source, then
/// ` ⇒`, then every unit `measure` must find, in source order, as
/// `name:letters` with `bit_of` reading each letter (`-` = none); an
/// empty right side asserts that `measure` finds nothing at all. A
/// string per case rather than a (source, table) tuple: a run of such
/// tuples is this repo's most-rhyming token shape, and its own clone
/// gate said so when the visibility tables were drafted — the mention
/// category word measures its tables through this same throat.
pub fn check_word_case(
    lang: crate::scan::lang::Lang,
    case: &str,
    measure: impl Fn(&str, crate::scan::lang::Lang) -> Vec<(String, i64)>,
    bit_of: impl Fn(char) -> i64,
) {
    let (src, want) = case.split_once(" ⇒").expect("case has ` ⇒`");
    let want: Vec<(String, i64)> = want
        .split_whitespace()
        .map(|unit| {
            let (name, letters) = unit.rsplit_once(':').expect("name:letters");
            (name.to_string(), letters.chars().map(&bit_of).sum())
        })
        .collect();
    assert_eq!(measure(src, lang), want, "{src:?}");
}

/// The one seeded stream every differential leg draws from: splitmix64
/// (Steele, Lea and Flood 2014) over the seed mixed with the leg's salt
/// — no dependency, no clock, the same words on every toolchain. Each
/// generator module keeps its own word tables and draws through this.
pub struct Draw(u64);

impl Draw {
    /// The stream of `seed` under the leg's `salt`.
    pub fn new(seed: u64, salt: u64) -> Draw {
        Draw(seed ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15))
    }

    /// The seed the environment variable `env` names (a decimal u64),
    /// else `default`, under the leg's `salt`.
    pub fn seeded(env: &str, default: u64, salt: u64) -> Draw {
        let seed = std::env::var(env).ok().and_then(|s| s.parse().ok());
        Draw::new(seed.unwrap_or(default), salt)
    }

    /// The ladder and document differentials' stream: the seed
    /// `CE_LADDER_DIFF_SEED` names, under the leg's salt.
    pub fn ladder(salt: u64) -> Draw {
        Draw::seeded("CE_LADDER_DIFF_SEED", 0x5eed_2033, salt)
    }

    /// The next word: the state steps by the golden gamma, then the
    /// finaliser's three xor-shift rounds.
    pub fn word(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A draw in `0..n` (0 when `n` is 0).
    pub fn below(&mut self, n: usize) -> usize {
        (self.word() % n.max(1) as u64) as usize
    }

    /// True `percent` times in a hundred.
    pub fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    /// One element of a slice.
    pub fn pick_from<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len())]
    }

    /// One word of a table whose words are joined by `|` (an empty word
    /// allowed).
    pub fn pick<'t>(&mut self, table: &'t str) -> &'t str {
        let k = self.below(table.split('|').count());
        table.split('|').nth(k).unwrap_or_default()
    }
}
