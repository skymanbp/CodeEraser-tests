//! The `#form` suffix of a fact id: the value grammar, the rendering
//! a document sees, and the token class a chip span is read with.

/// `#v` three-part SemVer · `#vminor` major.minor · `#digits` a bare
/// integer (rendered with thousands commas from 1,000) · `#schemaver`
/// a report id `ce.<name>/<ver>` · `#word` a count 1..=20 rendered as
/// the surface language's number word · `#Word` the same, capitalized
/// at a sentence start (Chinese has no case: identical).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    V,
    VMinor,
    Digits,
    SchemaVer,
    Word,
    WordCap,
}

impl Form {
    pub fn of(id: &str) -> Form {
        match id.rsplit_once('#') {
            Some((_, "v")) => Form::V,
            Some((_, "vminor")) => Form::VMinor,
            Some((_, "digits")) => Form::Digits,
            Some((_, "schemaver")) => Form::SchemaVer,
            Some((_, "word")) => Form::Word,
            Some((_, "Word")) => Form::WordCap,
            _ => panic!("fact id {id:?}: no known #form suffix"),
        }
    }

    /// Is `c` part of a document token of this form, on a surface in
    /// the language `zh` says? A chip span is read as maximal runs of
    /// these characters (chip.rs).
    pub fn token_char(self, zh: bool, c: char) -> bool {
        match self {
            Form::V | Form::VMinor => c.is_ascii_digit() || c == '.',
            Form::Digits => c.is_ascii_digit() || c == ',',
            Form::SchemaVer => c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-'),
            Form::Word | Form::WordCap if zh => "〇一两二三四五六七八九十".contains(c),
            Form::Word | Form::WordCap => c.is_ascii_alphabetic(),
        }
    }

    /// Does the registry value have this form's canonical shape?
    /// (`#digits` stays bare here — the commas are rendering.)
    pub fn admits(self, value: &str) -> bool {
        match self {
            Form::V => dotted(value, 3),
            Form::VMinor => dotted(value, 2),
            Form::Digits => dotted(value, 1),
            Form::SchemaVer => value
                .strip_prefix("ce.")
                .and_then(|rest| rest.split_once('/'))
                .is_some_and(|(name, ver)| {
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                        && (dotted(ver, 1) || dotted(ver, 3))
                }),
            Form::Word | Form::WordCap => {
                value.parse::<usize>().is_ok_and(|n| (1..=20).contains(&n))
            }
        }
    }

    /// The text a document carries for `value`.
    pub fn render(self, value: &str, zh: bool) -> String {
        match self {
            Form::Digits => grouped(value),
            Form::Word => word(value, zh).to_string(),
            Form::WordCap => {
                let w = word(value, zh);
                let mut out = String::with_capacity(w.len());
                let mut chars = w.chars();
                out.extend(chars.next().map(|c| c.to_ascii_uppercase()));
                out.push_str(chars.as_str());
                out
            }
            _ => value.to_string(),
        }
    }
}

/// `parts` non-empty all-digit groups joined by `.`.
fn dotted(value: &str, parts: usize) -> bool {
    let groups: Vec<&str> = value.split('.').collect();
    groups.len() == parts
        && groups
            .iter()
            .all(|g| !g.is_empty() && g.bytes().all(|b| b.is_ascii_digit()))
}

/// Thousands commas from 1,000 (README: "4,096").
fn grouped(digits: &str) -> String {
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The number word for 1..=20. Chinese 2 is 两 — every chipped count
/// stands before a measure word (两个 / 两条), where 二 is wrong.
fn word(value: &str, zh: bool) -> &'static str {
    const EN: &str = "one two three four five six seven eight nine ten eleven twelve \
                      thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
    const ZH: &str =
        "一 两 三 四 五 六 七 八 九 十 十一 十二 十三 十四 十五 十六 十七 十八 十九 二十";
    let n: usize = value.parse().expect("a #word value is 1..=20");
    let table = if zh { ZH } else { EN };
    table
        .split_whitespace()
        .nth(n - 1)
        .expect("twenty words per language")
}
