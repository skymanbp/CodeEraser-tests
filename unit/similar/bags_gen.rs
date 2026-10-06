//! The draws of the bags/1 differential (bags_diff.rs): a seeded stream,
//! prose that reaches every step of the stemmer and every class the
//! splitter cuts on, identifiers in camel / snake / capital runs, and
//! source files of six languages whose units carry them.

/// splitmix64 over the seed: three seeds, three distinct streams.
pub(super) struct Draw(u64);

impl Draw {
    pub(super) fn new(seed: u64, salt: u64) -> Draw {
        Draw(seed.wrapping_mul(0x0100_0000_01b3) ^ salt)
    }

    /// The next word: the state steps by the golden gamma, then the
    /// finaliser's three xor-shift rounds (the last multiplies by one).
    fn word(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        [
            (30, 0xbf58_476d_1ce4_e5b9),
            (27, 0x94d0_49bb_1331_11eb),
            (31, 1),
        ]
        .iter()
        .fold(self.0, |z, &(s, m)| (z ^ (z >> s)).wrapping_mul(m))
    }

    /// A draw below `n` (usize is 64 bits wherever the tests run).
    pub(super) fn below(&mut self, n: usize) -> usize {
        self.word() as usize % n
    }

    /// One word of a table: its words sit between spaces and line breaks.
    pub(super) fn one<'t>(&mut self, table: &'t str) -> &'t str {
        let k = self.below(words(table).count());
        words(table).nth(k).unwrap_or_default()
    }

    /// Any scalar value: most from the low planes, some anywhere.
    fn scalar(&mut self) -> char {
        let top = if self.below(4) == 0 {
            0x11_0000
        } else {
            0x3_0000
        };
        loop {
            if let Some(c) = char::from_u32(self.below(top) as u32) {
                return c;
            }
        }
    }
}

/// A table's words, in order: what sits between spaces and line breaks.
fn words(table: &str) -> impl Iterator<Item = &str> {
    table.split([' ', '\n']).filter(|w| !w.is_empty())
}

/// Porter's examples and suffix cases, stop words, and the odd spellings.
const ENGLISH: &str = r"caresses ponies ties caress cats feed agreed plastered bled motoring sing
    conflated troubled sized hopping tanned falling hissing fizzed failing filing happy sky
    relational conditional rational valenci hesitanci digitizer conformabli radicalli
    differentli vileli analogousli vietnamization predication operator feudalism decisiveness
    hopefulness callousness formaliti sensitiviti sensibiliti triplicate formative formalize
    electriciti electrical hopeful goodness revival allowance inference airliner gyroscopic
    adjustable defensible irritant replacement adjustment dependent adoption homologou
    communism activate angulariti homologous effective bowdlerize probate rate cease
    controll roll bed yes y by is the a an of to and or in it its i s t one which what
    fetch user record load parse json file http server query row draw";

/// Letters, digits and marks from many scripts and classes.
const ODD: &str = "İ ı ß ẞ Σ σ ς ǅ ǆ Ǆ ﬁ Ⅻ ⅻ Ⓐ ⓐ ² ½ ٣ ০ 〇 Ꭰ ꭰ ʰ ª ᾈ ᾀ ǰ Ω ω К к ẞig \
    café cafe\u{301} 日本語 한국어 Ⅷabc abcⅧ X\u{345}y Ǳ ǲ ǳ ﬀ 𝔄 𝔞 𝟏 ꙮ Ꙍ ꙍ";

const PUNCT: &str = "_ - . , ; : / \\ ( ) [ ] < > + * = ! ? # @ $ % & | ~ ' \u{a0} \u{2028} \t";

/// One piece of an identifier.
fn piece(d: &mut Draw) -> String {
    let w = d.one(ENGLISH).to_string();
    match d.below(6) {
        0 => w.to_uppercase(),
        1 => capital(&w),
        2 => d.one(ODD).to_string(),
        3 => format!("{w}{}", d.below(100)),
        _ => w,
    }
}

fn capital(w: &str) -> String {
    let mut cs = w.chars();
    cs.next()
        .map_or(String::new(), |c| c.to_uppercase().chain(cs).collect())
}

/// An identifier of one to four pieces, joined camel / snake / glued.
pub(super) fn ident(d: &mut Draw) -> String {
    let n = 1 + d.below(4);
    let pieces: Vec<String> = std::iter::repeat_with(|| piece(d)).take(n).collect();
    let joined = match d.below(3) {
        0 => pieces.join("_"),
        1 => pieces
            .iter()
            .enumerate()
            .map(|(i, p)| if i == 0 { p.clone() } else { capital(p) })
            .collect(),
        _ => pieces.concat(),
    };
    let lead = joined.chars().next().is_some_and(char::is_alphabetic);
    if lead { joined } else { format!("x{joined}") }
}

/// A line of prose: words, identifiers, odd letters, punctuation and,
/// now and then, any scalar value at all.
pub(super) fn prose(d: &mut Draw, words: usize) -> String {
    let mut out = String::new();
    for _ in 0..words {
        match d.below(10) {
            0..=4 => out.push_str(d.one(ENGLISH)),
            5 | 6 => out.push_str(&ident(d)),
            7 => out.push_str(d.one(ODD)),
            8 => out.push(d.scalar()),
            _ => out.push_str(d.one(PUNCT)),
        }
        out.push_str(if d.below(5) == 0 { "" } else { " " });
    }
    out
}

/// A doc line safe inside every comment form the sources use.
pub(super) fn doc(d: &mut Draw) -> String {
    let n = 1 + d.below(12);
    prose(d, n)
        .replace(['\n', '\r', '\u{2028}', '\u{85}', '\u{b}', '\u{c}'], " ")
        .replace("*/", "* /")
        .replace("\"\"\"", "'")
        .replace('\\', "/")
}
