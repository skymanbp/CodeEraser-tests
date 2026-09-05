use super::*;
use crate::similar::terms::{feature_term, word_term};

const RUST: &str = r#"
/// Fetches the user record by id.
#[allow(dead_code)]
fn fetch_user(id: u32) -> Option<String> {
    let row = query("select", id);
    if row.is_some() { Some("x".to_string()) } else { None }
}

struct Repo;

impl Repo {
    /// Loads one user.
    fn load_user(&self, id: u32) -> Option<String> {
        self.query(id).map(|r| r.trim().to_string())
    }
    fn query(&self, _id: u32) -> Option<String> { None }
}

fn lookup(_q: &str, _id: u32) -> Option<u32> { Some(1) }
"#;

const PYTHON: &str = r#"
class Store:
    """A store."""

    def fetch_user(self, id):
        """Fetch the user by id."""
        return self.query(id, True)

    def query(self, id, strict):
        return None


def helper():
    return 1
"#;

/// `<lang> <unit key> <channel letter> <spelling> <+|->` per line (`~`
/// stands for the space in `impl Repo`) — a
/// word channel takes the spelling as a (stemmed) word, a feature
/// channel as the feature. One text for both corpora and all six
/// channels: a tuple table of this length is its own clone row by
/// row, and the per-corpus assertion runs before it were each
/// other's clone as prose. Read with the comments:
///   fetch_user — name words; fn shape with arity and return; the bare
///     call and the member calls' last segments as callees; the
///     leading `///` block across the attribute, stop word dropped;
///   load_user — `&self` counts in the arity; declared in an impl; its
///     own doc, not its neighbour's; `trim` is the closure's, `map` its own;
///   impl Repo — keywords dropped, a method's doc never reaches it, own
///     nodes stop at nested units;
///   Python — docstrings by kind, the class's stays the class's.
const EXPECT: &str = "\
rs fetch_user/1 N fetch +
rs fetch_user/1 N user +
rs fetch_user/1 P k:fn +
rs fetch_user/1 P p:1 +
rs fetch_user/1 P ret:1 +
rs fetch_user/1 C query +
rs fetch_user/1 C some +
rs fetch_user/1 C string +
rs fetch_user/1 D fetch +
rs fetch_user/1 D record +
rs fetch_user/1 D the -
rs fetch_user/1 L l:str +
rs load_user/2 P k:method +
rs load_user/2 D load +
rs load_user/2 D fetch -
rs load_user/2 C query +
rs load_user/2 C map +
rs load_user/2 C trim -
rs impl~Repo N repo +
rs impl~Repo N impl -
rs impl~Repo D load -
rs impl~Repo C trim -
rs lookup/2 L l:num +
py fetch_user/2 P k:method +
py fetch_user/2 D fetch +
py fetch_user/2 D user +
py fetch_user/2 L l:bool +
py fetch_user/2 C query +
py Store P k:class +
py Store D store +
py Store D fetch -
py helper/0 P k:fn +
py helper/0 P ret:0 +";

#[test]
fn bags_read_the_six_channels_as_tabled() {
    let corpora = [
        ("rs", file_bags(RUST, Lang::Rust)),
        ("py", file_bags(PYTHON, Lang::Python)),
    ];
    for line in EXPECT.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let (lang, key, ch, spelling, present) =
            (f[0], f[1].replace('~', " "), f[2], f[3], f[4] == "+");
        let bags = &corpora.iter().find(|(l, _)| *l == lang).expect("corpus").1;
        let bag = bags
            .iter()
            .find(|b| b.key == key)
            .unwrap_or_else(|| panic!("{line}"));
        let ch = *Channel::ALL
            .iter()
            .find(|c| c.label() == ch)
            .expect("channel");
        let term = if ch.is_words() {
            word_term(ch, spelling)
        } else {
            feature_term(ch, spelling.as_bytes())
        };
        assert_eq!(bag.terms.contains_key(&term), present, "{line}");
    }
    let f = corpora[0]
        .1
        .iter()
        .find(|b| b.key == "fetch_user/1")
        .expect("unit");
    assert!(f.channel(Channel::Structure).len() > 3, "kind histogram");
}

#[test]
fn anonymous_units_carry_no_name_evidence_and_markdown_none_at_all() {
    let ts = "const f = (a: number) => a + 1;\nfunction g() { return [1].map(x => x); }\n";
    let bags = file_bags(ts, Lang::TypeScript);
    let anon = bags
        .iter()
        .find(|b| b.key.starts_with("(anonymous)"))
        .expect("closure");
    assert!(anon.channel(Channel::Name).is_empty());
    assert!(
        anon.terms
            .contains_key(&feature_term(Channel::Shape, b"k:lambda"))
    );
    assert!(file_bags("# Title\n\ntext\n", Lang::Markdown).is_empty());
}

#[test]
fn the_bag_universe_is_the_unitsig_universe() {
    let facts = crate::dedup::unitcache::unit_facts(RUST, Lang::Rust);
    let bags = file_bags(RUST, Lang::Rust);
    let a: Vec<(String, i64)> = facts.iter().map(|f| (f.key.clone(), f.nth)).collect();
    let b: Vec<(String, i64)> = bags.iter().map(|b| (b.key.clone(), b.nth)).collect();
    assert_eq!(a, b, "same units, same nth, same order");
}
