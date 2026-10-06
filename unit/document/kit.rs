//! The W7 differential's kit (plan v2.33 W7; test-only): one `Leg` per
//! family draws faces from the seeded stream, asks the core to spell
//! every drawn reference three ways over the face's strings — alone, in
//! a document, in a console line (`document/1` `inspect`, which the
//! product never sends) — and holds each answer to the frozen resolver
//! spelling the same reference the 1324c927 way (`bind`, `bind_lines`):
//! a string both ways, or none both ways. The tally names how many
//! references spelled, so a leg that only ever agrees on refusals shows.

use super::binder::{Resolve, Why, bind, bind_lines};
use super::rng::Rng;
use crate::corelink::Link;
use serde_json::{Value, json};

/// The words a face's strings are drawn from: plain names, the empty
/// string, holes and braces, quotes and backslashes, wide and astral
/// characters, and every whitespace Rust's `split_whitespace` reads
/// (and three it does not: U+180E, U+200B, U+FEFF).
pub(super) const WORDS: &str = "a|src|lib.rs|main.py|README.md||a b|  lead|trail  |{}|x{}y|{|}|\\|\"q\"|中文|\
名字 空格|é|😀|tab\there|nl\nline|\u{a0}nbsp|\u{3000}ideo|\u{2028}ls|\u{85}nel|\u{1680}ogham|\u{180e}mvs|\
\u{200b}zw|\u{feff}bom|x/y|.|./|a/b/c.py|Z|_|#|f#0|\r\n|\u{b}vt|\u{c}ff|\u{2009}thin";

/// References per leg (`CE_W7_DIFF_N`, default 10,000).
pub(super) fn total() -> usize {
    let n = std::env::var("CE_W7_DIFF_N").ok();
    n.and_then(|n| n.parse().ok()).unwrap_or(10_000)
}

/// One leg: a fresh `Leg`, `case` drawn until the references are in,
/// the tally.
pub(super) fn run(name: &'static str, salt: u64, mut case: impl FnMut(&mut Leg)) {
    let mut leg = Leg::new(name, salt);
    while !leg.done() {
        case(&mut leg);
    }
    leg.finish();
}

/// A leg's test: `leg!(test_name, "leg name", salt, |leg| { … })`,
/// one `#[ignore]`d instrument that names a core.
macro_rules! leg {
    ($test:ident, $name:literal, $salt:literal, $case:expr) => {
        #[test]
        #[ignore = "W7 differential: names a core (CE_CORE_BIN)"]
        fn $test() {
            super::super::kit::run($name, $salt, $case);
        }
    };
}
pub(super) use leg;

/// One family's leg: its stream, its link, its tally.
pub(super) struct Leg {
    pub name: &'static str,
    pub rng: Rng,
    link: Link,
    asked: usize,
    spelled: usize,
    missed: usize,
    misses: Vec<String>,
}

impl Leg {
    pub(super) fn new(name: &'static str, salt: u64) -> Leg {
        let core = crate::daemon::judge::core_bin().expect("a core (CE_CORE_BIN)");
        let (link, _) = Link::open(&core).expect("the core answers");
        let rng = Rng::new(salt);
        let (asked, spelled, missed, misses) = (0, 0, 0, Vec::new());
        Leg {
            name,
            rng,
            link,
            asked,
            spelled,
            missed,
            misses,
        }
    }

    /// Whether the leg drew its references.
    pub(super) fn done(&self) -> bool {
        self.asked >= total()
    }

    pub(super) fn word(&mut self) -> String {
        self.rng.pick(WORDS).to_string()
    }

    /// `n` words below `max` (0..max).
    pub(super) fn words(&mut self, max: usize) -> Vec<String> {
        let n = self.rng.below(max);
        (0..n).map(|_| self.word()).collect()
    }

    /// A face's reasons (class `why`): as the frozen face held them,
    /// and as it sends them.
    pub(super) fn reasons(&mut self) -> (Why, Vec<String>) {
        let texts = self.words(3);
        let mut why = Why::default();
        for t in &texts {
            why.add(t.clone());
        }
        (why, texts)
    }

    /// A path of one to three words.
    pub(super) fn path(&mut self) -> String {
        let n = 1 + self.rng.below(3);
        (0..n).map(|_| self.word()).collect::<Vec<_>>().join("/")
    }

    /// Below `max` draws of `one`.
    pub(super) fn many<T>(&mut self, max: usize, mut one: impl FnMut(&mut Leg) -> T) -> Vec<T> {
        let n = self.rng.below(max);
        (0..n).map(|_| one(self)).collect()
    }

    /// `least` paths and below `more` besides.
    pub(super) fn paths(&mut self, least: usize, more: usize) -> Vec<String> {
        let n = least + self.rng.below(more);
        (0..n).map(|_| self.path()).collect()
    }

    /// An index into a list of `size`: mostly within it, now and then
    /// one or two past it, negative, or at the edges of i64 / u64.
    pub(super) fn int(&mut self, size: usize) -> Value {
        match self.rng.below(20) {
            0 => json!(-1 - self.rng.below(3) as i64),
            1 => [json!(i64::MIN), json!(i64::MAX), json!(u64::MAX)][self.rng.below(3)].clone(),
            2..=4 => json!(size + self.rng.below(2)),
            _ => json!(self.rng.below(size.max(1))),
        }
    }

    /// `[class, integers…]` over lists of `sizes`, its arity now and
    /// then one off, now and then holding a non-integer.
    pub(super) fn reference(&mut self, class: &str, sizes: &[usize]) -> Value {
        let n = match self.rng.below(20) {
            0 => sizes.len() + 1,
            1 => sizes.len().saturating_sub(1),
            _ => sizes.len(),
        };
        let mut v = vec![json!(class)];
        for k in 0..n {
            v.push(self.int(sizes.get(k).copied().unwrap_or(3)));
        }
        if self.rng.chance(1) {
            v.push(json!("7"));
        }
        Value::Array(v)
    }

    /// `count` references, each of a class drawn from `classes` (with
    /// its list sizes) or, one time in twenty, a class no face holds.
    pub(super) fn references(
        &mut self,
        classes: &[(&str, Vec<usize>)],
        count: usize,
    ) -> Vec<Value> {
        (0..count)
            .map(|_| match self.rng.below(20) {
                0 => self.reference("nope", &[2]),
                _ => {
                    let (class, sizes) = &classes[self.rng.below(classes.len())];
                    self.reference(class, sizes)
                }
            })
            .collect()
    }

    /// The core's `inspected` answer over a face's strings and rows.
    pub(super) fn ask(&mut self, family: &str, face: (&Value, &Value), inspect: Value) -> Value {
        let body = json!({
            "family": family, "ranges": {}, "rows": face.1, "facts": {}, "degraded": null,
            "lang": 0, "strings": face.0, "inspect": inspect,
        });
        let reply = self
            .link
            .request(crate::document::KIND, body)
            .expect("a reply");
        let inspected = reply["inspected"].clone();
        assert!(
            inspected.is_object(),
            "{}: no inspected answer: {reply}",
            self.name
        );
        inspected
    }

    /// Every reference spelled by the core (alone, in a document, in a
    /// line) and by `frozen`, compared; the core's rows answered.
    pub(super) fn spell(
        &mut self,
        family: &str,
        face: (&Value, &Value),
        frozen: &dyn Resolve,
        refs: Vec<Value>,
    ) -> Value {
        let docs: Vec<Value> = refs.iter().map(document).collect();
        let lines: Vec<Value> = (refs.iter().enumerate()).map(|(i, r)| line(i, r)).collect();
        let inspect = json!({"refs": refs, "docs": docs, "lines": lines});
        let got = self.ask(family, face, inspect);
        for (i, r) in refs.iter().enumerate() {
            let want = bind(json!({"$": r}), frozen).ok().unwrap_or(Value::Null);
            self.spelled += usize::from(want.is_string());
            self.hold(r, "ref", &want, &got["refs"][i]);
            let want = bind(docs[i].clone(), frozen).ok().unwrap_or(Value::Null);
            self.hold(r, "doc", &want, &got["docs"][i]);
            self.hold(r, "line", &bound(&lines[i], frozen), &got["lines"][i]);
        }
        self.asked += refs.len();
        got["rows"].clone()
    }

    /// Two hundred references drawn over `drawn` spelled as `spell`
    /// does, for a face whose request carries no rows.
    pub(super) fn spell_drawn(
        &mut self,
        family: &str,
        sent: &Value,
        frozen: &dyn Resolve,
        drawn: &[(&str, Vec<usize>)],
    ) -> Value {
        let refs = self.references(drawn, 200);
        self.spell(family, (sent, &json!({})), frozen, refs)
    }

    /// Lines alone (`[stream, text, reference…]`), each held to the
    /// frozen `bound` through `frozen`.
    pub(super) fn lines(
        &mut self,
        family: &str,
        face: &Value,
        frozen: &dyn Resolve,
        lines: Vec<Value>,
    ) {
        let got = self.ask(family, (face, &json!({})), json!({"lines": lines}));
        for (i, l) in lines.iter().enumerate() {
            let want = bound(l, frozen);
            self.spelled += usize::from(!want.is_null());
            self.hold(l, "line", &want, &got["lines"][i]);
        }
        self.asked += lines.len();
    }

    /// A measured table held to the frozen measure.
    pub(super) fn table(&mut self, what: &Value, want: &Value, got: &Value) {
        self.asked += 1;
        self.spelled += 1;
        self.hold(what, "rows", want, got);
    }

    fn hold(&mut self, what: &Value, form: &str, want: &Value, got: &Value) {
        if want != got {
            self.missed += 1;
            if self.misses.len() < 20 {
                self.misses
                    .push(format!("{form} {what}: frozen {want}, core {got}"));
            }
        }
    }

    /// The tally, printed verbatim; any mismatch fails the leg.
    pub(super) fn finish(self) {
        let seed = std::env::var("CE_LADDER_DIFF_SEED").unwrap_or_else(|_| "default".into());
        let (asked, spelled, missed) = (self.asked, self.spelled, self.missed);
        println!(
            "w7 diff {} (seed {seed}): {asked} cases, {spelled} spelled, {missed} mismatches",
            self.name
        );
        assert!(
            missed == 0,
            "{}: {missed} mismatch(es), first: {:#?}",
            self.name,
            self.misses
        );
        assert!(
            spelled * 4 >= asked,
            "{}: only {spelled} of {asked} spelled",
            self.name
        );
    }
}

/// The reference twice in a document, beside an object that only looks
/// like one and a string that only looks like a hole.
fn document(r: &Value) -> Value {
    json!({"a": [1, {"$": r}, "{}"], "b": {"$": r, "c": null}, "d": {"$": r}})
}

/// The reference in a line between two words, on stream `i % 2`.
fn line(i: usize, r: &Value) -> Value {
    json!([i % 2, "<{}>", {"$": r}])
}

/// A line as the frozen binder spelled it (`[stream, text]`), or null.
fn bound(l: &Value, frozen: &dyn Resolve) -> Value {
    let reply = json!({"lines": [l], "exit": {"fail": false}});
    match bind_lines(&reply, frozen) {
        Ok((lines, _)) => {
            let stream = u8::from(lines[0].stream == super::binder::Stream::Err);
            json!([stream, lines[0].text])
        }
        Err(_) => Value::Null,
    }
}
