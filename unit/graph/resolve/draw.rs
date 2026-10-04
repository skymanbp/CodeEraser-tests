//! The seeded draws the reader legs share: a splitmix64 stream, picks
//! from a word list, strings over an alphabet, and the mutations a real
//! configuration text goes through (a byte-order mark, CRLF, a comment,
//! a continuation, a non-ASCII or White_Space character, a cut, a
//! doubled or dropped line).

pub(super) struct Draw {
    state: u64,
}

impl Draw {
    pub(super) fn seeded(leg: u64) -> Draw {
        let seed = std::env::var("CE_RESOLVE_DIFF_SEED")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0x7e47_2033);
        Draw {
            state: seed.rotate_left(17) ^ leg.wrapping_mul(0xbf58_476d_1ce4_e5b9),
        }
    }

    fn step(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0..n`.
    pub(super) fn under(&mut self, n: usize) -> usize {
        (self.step() % n.max(1) as u64) as usize
    }

    /// True `pct` times in a hundred.
    pub(super) fn chance(&mut self, pct: usize) -> bool {
        self.under(100) < pct
    }

    pub(super) fn one<'a>(&mut self, words: &[&'a str]) -> &'a str {
        words[self.under(words.len())]
    }

    /// Up to `max` characters of an alphabet.
    pub(super) fn text(&mut self, alphabet: &[char], max: usize) -> String {
        let n = self.under(max + 1);
        (0..n)
            .map(|_| alphabet[self.under(alphabet.len())])
            .collect()
    }

    /// Up to `max` words of a list, each followed by one separator.
    pub(super) fn words(&mut self, list: &[&str], seps: &[&str], max: usize) -> String {
        let n = self.under(max + 1);
        let mut out = String::new();
        for _ in 0..n {
            out.push_str(self.one(list));
            out.push_str(self.one(seps));
        }
        out
    }

    /// A real text, mutated a few times.
    pub(super) fn mutate(&mut self, text: &str) -> String {
        let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        for _ in 0..self.under(4) {
            let at = self.under(lines.len());
            match self.under(8) {
                0 => lines[at].insert(0, '\u{feff}'),
                1 => lines[at].push('\r'),
                2 => lines[at].push_str(" // note é"),
                3 => lines[at].push('\\'),
                4 => lines.insert(at, self.one(ODD).to_string()),
                5 => lines[at] = lines[at].chars().take(self.under(12)).collect(),
                6 => lines.insert(at, lines[at].clone()),
                _ => {
                    lines.remove(at);
                    if lines.is_empty() {
                        lines.push(String::new());
                    }
                }
            }
        }
        lines.join(if self.chance(30) { "\r\n" } else { "\n" })
    }
}

/// Characters the readers treat specially or must not: every kind of
/// space Rust's White_Space holds and GHC's isSpace does not, combining
/// and letter-number marks, a lone separator.
pub(super) const ODD: &[&str] = &[
    "\u{85}", "\u{a0}x", "\u{2028}", "\u{3000}", "\u{345}", "\u{2170}", "\u{1680}", "é", "", "\t",
    "=>", ")", "(", "\"",
];

/// The real configuration texts named `name` under the trees
/// CE_RESOLVE_DIFF_TREES lists (`;`-separated), at most two hundred,
/// build, dependency and VCS directories skipped.
pub(super) fn real_texts(name: &str) -> Vec<String> {
    let roots = std::env::var("CE_RESOLVE_DIFF_TREES").unwrap_or_default();
    let skip = [
        "node_modules",
        ".git",
        "target",
        ".venv",
        "site-packages",
        "dist-newstyle",
    ];
    let mut stack: Vec<(std::path::PathBuf, usize)> = roots
        .split(';')
        .filter(|r| !r.is_empty())
        .map(|r| (r.into(), 0))
        .collect();
    let mut out = Vec::new();
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let (p, file) = (e.path(), e.file_name());
            if p.is_dir() && depth < 3 && !skip.iter().any(|s| file == *s) {
                stack.push((p, depth + 1));
            } else if file == name
                && out.len() < 200
                && let Ok(t) = std::fs::read_to_string(&p)
            {
                out.push(t);
            }
        }
    }
    out
}
