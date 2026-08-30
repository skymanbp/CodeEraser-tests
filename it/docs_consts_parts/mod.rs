//! The source-constant harvest the how-page consts gate binds to
//! (split out of docs_consts.rs at the 300-line dogfood wall, plan
//! v2.21 S9): every `const NAME = value` in cli/src, every array
//! arity `const NAME: [T; N]` as `NAME.len`, every top-level Haskell
//! binding `name = value` in core/app, and the numeric reading of a
//! value through one hop of aliasing. The leaf owns the shape; the
//! parent references downward.

use crate::common::files_with_ext;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Def {
    pub file: PathBuf,
    pub name: String,
    pub value: String,
}

/// The ONE walk-and-parse shell every grammar shares; the per-line
/// grammar is the only thing that differs, so it is the parameter.
pub fn defs_in(
    root: &Path,
    dir: &str,
    ext: &str,
    parse: fn(&str) -> Vec<(String, String)>,
) -> Vec<Def> {
    let mut files = Vec::new();
    files_with_ext(&root.join(dir), ext, &mut files);
    files.sort();
    files
        .into_iter()
        .flat_map(|file| {
            let text = std::fs::read_to_string(&file).expect("read source file");
            text.lines()
                .flat_map(parse)
                .map(|(name, value)| Def {
                    file: file.clone(),
                    name,
                    value,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Both facts a `const NAME` line can carry: its value, and — for
/// `const NAME: [T; N]` — the declared arity as `NAME.len`, so a count
/// chip binds to the array's type (collision routing) instead of
/// number-hunting the array body.
pub fn rust_line(line: &str) -> Vec<(String, String)> {
    let Some((name, rest)) = line
        .find("const ")
        .and_then(|at| ident(&line[at + "const ".len()..]))
    else {
        return Vec::new();
    };
    let value = rest
        .split_once('=')
        .map(|(_, v)| v.trim().trim_end_matches(';').trim())
        .filter(|v| !v.is_empty())
        .map(|v| (name.to_string(), v.to_string()));
    let arity = rest
        .split_once(": [")
        .and_then(|(_, r)| r.split_once(']'))
        .and_then(|(inner, _)| inner.rsplit_once(';'))
        .map(|(_, n)| n.trim())
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map(|n| (format!("{name}.len"), n.to_string()));
    value.into_iter().chain(arity).collect()
}

pub fn haskell_line(line: &str) -> Vec<(String, String)> {
    ident(line.trim_start())
        .and_then(|(name, after)| {
            let value = after.trim_start().strip_prefix('=')?.trim();
            (!name.is_empty() && !value.is_empty()).then(|| (name.to_string(), value.to_string()))
        })
        .into_iter()
        .collect()
}

/// The leading identifier of `s` (possibly empty) and what follows it;
/// None when the identifier runs to the end of the line.
fn ident(s: &str) -> Option<(&str, &str)> {
    let len = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    Some(s.split_at(len))
}

pub fn normalize(mut value: String) -> String {
    value = value.replace('_', "");
    while value.ends_with(".0") {
        value.truncate(value.len() - 2);
    }
    value
}

pub fn first_number(value: &str) -> Option<String> {
    let start = value
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)?;
    let tail = &value[start..];
    let end = tail
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit() || *c == '_' || *c == '.')
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(1);
    Some(normalize(tail[..end].to_string()))
}

/// The numeric value of a definition, following one alias hop per
/// name (`seen` stops a cycle).
pub fn value_for(def: &Def, defs: &[Def], seen: &mut BTreeSet<String>) -> Option<String> {
    if let Some(value) = first_number(&def.value) {
        return Some(value);
    }
    let alias = def.value.rsplit("::").next()?.trim();
    if !seen.insert(alias.to_string()) {
        return None;
    }
    let next = defs.iter().find(|d| d.name == alias)?;
    value_for(next, defs, seen)
}
