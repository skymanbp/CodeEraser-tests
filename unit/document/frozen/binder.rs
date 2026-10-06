//! Frozen W7 oracle (plan v2.33 W7; test-only): the binder: cli/src/document.rs and cli/src/document/lines.rs, copied byte for
//! byte from 1324c927 (spans cli/src/document.rs 33-37, 225-240, 242-259, 261-265, 267-283, 285-288, 290-295, 297-303, 305-313; cli/src/document/lines.rs 24-29, 31-36, 46-59, 61-96) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use crate::document::SINCE;
use anyhow::{Result, anyhow, bail};
use serde_json::Value;

#[path = "../spelled/binder.rs"]
mod spelled;

/// A face's strings, by the class and the integers a reference
/// carries; None = a class it does not hold or an integer outside it.
pub trait Resolve {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String>;
}

/// Every reference replaced by its string; anything else as it is. A
/// reference is an object whose one key is `$`.
pub fn bind(v: Value, r: &dyn Resolve) -> Result<Value> {
    Ok(match v {
        Value::Array(xs) => {
            Value::Array(xs.into_iter().map(|x| bind(x, r)).collect::<Result<_>>()?)
        }
        Value::Object(o) if o.len() == 1 && o.contains_key("$") => resolved(&o["$"], r)?,
        Value::Object(o) => Value::Object(
            o.into_iter()
                .map(|(k, x)| Ok((k, bind(x, r)?)))
                .collect::<Result<_>>()?,
        ),
        other => other,
    })
}

fn resolved(reference: &Value, r: &dyn Resolve) -> Result<Value> {
    let parts = reference.as_array().filter(|p| !p.is_empty());
    let Some((class, ints)) = parts.and_then(|p| Some((p[0].as_str()?, &p[1..]))) else {
        bail!("document: a reference is not [class, integers…]: {reference}");
    };
    let ints: Vec<i128> = ints
        .iter()
        .map(|i| {
            i.as_i64()
                .map(i128::from)
                .or_else(|| i.as_u64().map(i128::from))
        })
        .collect::<Option<_>>()
        .ok_or_else(|| anyhow!("document: {reference} holds a non-integer"))?;
    r.resolve(class, &ints)
        .map(Value::String)
        .ok_or_else(|| anyhow!("document: no string for {reference}"))
}

/// This side's own texts a document names by index (the class `why`):
/// the reason a judgment did not happen, a fault's place and message,
/// a refusal's reason. One owner for the four faces' texts.
#[derive(Default)]
pub struct Why(Vec<String>);

impl Why {
    /// The text's index.
    pub fn add(&mut self, text: String) -> usize {
        self.0.push(text);
        self.0.len() - 1
    }

    /// The range the request declares for the class.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// The text a `why` reference names.
    pub fn at(&self, i: &[i128]) -> Option<String> {
        at(&self.0, i)
    }
}

/// A face's strings held as lists, one per reference class, each read
/// by a one-integer reference — the resolver of every face whose
/// references are indices.
pub struct Lists(pub Vec<(&'static str, Vec<String>)>);

impl Resolve for Lists {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let (_, list) = self.0.iter().find(|(c, _)| *c == class)?;
        at(list, ints)
    }
}

/// The string at `i` of `list`, for a resolver.
pub fn at(list: &[String], i: &[i128]) -> Option<String> {
    match i {
        [i] => usize::try_from(*i).ok().and_then(|i| list.get(i)).cloned(),
        _ => None,
    }
}

/// Each string's place in their joint sort order (ties share one).
pub fn ranks<'a>(strings: impl IntoIterator<Item = &'a str>) -> Vec<usize> {
    let all: Vec<&str> = strings.into_iter().collect();
    let mut sorted = all.clone();
    sorted.sort_unstable();
    all.iter()
        .map(|s| sorted.partition_point(|x| x < s))
        .collect()
}

/// Where a line goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// One bound line: its stream and its text, every hole filled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub stream: Stream,
    pub text: String,
}

/// The reply's lines, bound through `r`, and its `exit.fail`. A reply
/// without `lines` / `exit` came from a core older than the contract.
pub fn bind_lines(reply: &Value, r: &dyn Resolve) -> Result<(Vec<Line>, bool)> {
    let (Some(lines), Some(fail)) = (reply["lines"].as_array(), reply["exit"]["fail"].as_bool())
    else {
        bail!("document: the reply carries no lines / exit (a pre-{SINCE} core)");
    };
    let lines = lines
        .iter()
        .enumerate()
        .map(|(i, l)| bound(l, r).map_err(|e| anyhow!("document: line {i}: {e}")))
        .collect::<Result<_>>()?;
    Ok((lines, fail))
}

/// One `[stream, text, reference…]`, its holes filled.
fn bound(l: &Value, r: &dyn Resolve) -> Result<Line> {
    let Some([stream, text, refs @ ..]) = l.as_array().map(Vec::as_slice) else {
        bail!("not [stream, text, reference…]: {l}");
    };
    let stream = match stream.as_u64() {
        Some(0) => Stream::Out,
        Some(1) => Stream::Err,
        _ => bail!("stream {stream} is not 0 or 1"),
    };
    let Some(text) = text.as_str() else {
        bail!("text {text} is not a string");
    };
    let holes = text.matches("{}").count();
    if holes != refs.len() {
        bail!("{holes} hole(s) and {} reference(s)", refs.len());
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    for reference in refs {
        let is_ref = reference
            .as_object()
            .is_some_and(|o| o.len() == 1 && o.contains_key("$"));
        let (true, Value::String(s)) = (is_ref, super::bind(reference.clone(), r)?) else {
            bail!("{reference} is not a reference");
        };
        let (head, tail) = rest
            .split_once("{}")
            .expect("one hole per reference, counted");
        out.push_str(head);
        out.push_str(&s);
        rest = tail;
    }
    out.push_str(rest);
    Ok(Line { stream, text: out })
}
