//! The .cabal reader's leg (plan v2.33 W2-text stage D): the frozen
//! fa83a48d reader (unit/graph/oracle_cfg/cabal.rs, mounted at
//! `graph::cabal` under cfg(test)) reads each text from disk as the
//! Haskell rungs, the declared-target pass and the mounts table did; the
//! core reads the same text through `inspect.cabal`. Ten thousand
//! questions: every fourth a real .cabal of the trees
//! CE_RESOLVE_DIFF_TREES names, mutated (a byte-order mark, CRLF, a cut,
//! a doubled or dropped line, a non-ASCII or White_Space character), the
//! rest drawn line by line — `name:` before and after a header and in any
//! case, the component headers bare and named, `common` blocks and
//! `import:`, every field the reader routes over continuation lines with
//! comments inside, fields at column 0, a header with a colon, a field
//! name with a space, odd White_Space inside values, package names
//! holding an underscore, a dot, a colon, a plus or a non-ASCII letter.
//!
//! The frozen reader keeps its exposed-modules set private and answers
//! `exposes(m)`: the core's set is asked of it word by word over every
//! word the text spells and every word the core names.

use super::files::{bom_texts, scratch};
use super::{agree, inspect, link};
use crate::graph::cabal;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Lines a .cabal is drawn from.
const LINES: &str = "name: pkgA¦Name:   spaced  ¦name:¦NAME: other¦library¦Library¦library internal¦LIBRARY¦executable exe¦Executable  exe¦common shared¦common base-deps¦common¦import: shared¦  import: shared, base-deps¦  import: missing¦test-suite t¦benchmark b¦flag f¦  hs-source-dirs: src¦  hs-source-dirs: src, lib¦\ths-source-dirs:\t.¦  HS-Source-Dirs: ../x¦  hs-source-dirs: ../../..¦  hs-source-dirs:¦      app¦  exposed-modules: A¦  exposed-modules:¦    A.B¦\tData.Map, Util¦  Exposed-Modules: A B,C¦  other-modules: B¦  other-modules: A B¦  build-depends: base >=4 && <5, containers¦    , text¦  build-depends:¦\t(pkgA), -x, pkg-b ^>= 1¦  build-depends: my_pkg >= 1, a.b, lib:sub, x+y, 9lives, \u{e9}t\u{e9}¦  main-is: Main.hs¦  main-is:¦  main-is: src/Main.hs¦-- comment¦   -- indented comment¦  if os(windows)¦  else¦library:¦x y: z¦ghc-options: -Wall¦  name: late¦  default-language: Haskell2010¦source-repository head¦  exposed-modules: \u{c4}.B \u{e9}¦  hs-source-dirs: src\u{3000}lib¦  build-depends: \u{a0}base¦   ¦cabal-version: 2.4";

/// The words a text spells: each line cut at the reader's separators,
/// each piece also trimmed and also after a colon.
fn vocabulary(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.split(['\n', '\r']) {
        let after = line.split_once(':').map_or(line, |(_, v)| v);
        for piece in [line, after, line.trim(), after.trim()]
            .into_iter()
            .flat_map(|s| s.split([',', ' ', '\t']))
        {
            out.insert(piece.to_string());
            out.insert(piece.trim().to_string());
        }
    }
    out
}

/// One answer as the frozen reader gives it, the exposed set asked word
/// by word.
fn frozen(c: &cabal::Cabal, words: &BTreeSet<String>) -> Value {
    let stanzas: Vec<Value> = c
        .stanzas
        .iter()
        .map(|s| json!([s.roots, s.main_is, s.is_library]))
        .collect();
    let exposed: Vec<&String> = words.iter().filter(|w| c.exposes(w)).collect();
    json!([
        c.dir,
        c.name,
        stanzas,
        c.deps,
        c.has_library,
        c.hidden_modules,
        exposed
    ])
}

/// The core's answer with its exposed set asked the same way.
fn asked(core: &Value, words: &BTreeSet<String>) -> Value {
    let set: BTreeSet<&str> = core[6]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let library = core[4] == json!(true);
    let exposed: Vec<&String> = words
        .iter()
        .filter(|w| library && set.contains(w.as_str()))
        .collect();
    json!([
        core[0], core[1], core[2], core[3], core[4], core[5], exposed
    ])
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn cabals_agree() {
    let qs = bom_texts(
        10,
        "*.cabal",
        LINES,
        "x.cabal¦pkg/p.cabal¦\u{e9}/q.cabal¦a/b/c.cabal",
        14,
    );
    let root = scratch("cabal");
    let got = inspect(&mut link(), "cabal", &qs);
    let (mut want, mut core) = (Vec::new(), Vec::new());
    for (q, g) in qs.iter().zip(&got) {
        let (rel, text) = (q[0].as_str().expect("rel"), q[1].as_str().expect("text"));
        super::files::put(&root, rel, text);
        let c = cabal::parse(&root, rel).expect("readable");
        let mut words = vocabulary(text);
        let named = g[6]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str);
        words.extend(named.map(str::to_string));
        want.push(frozen(&c, &words));
        core.push(asked(g, &words));
    }
    agree("cabal", &qs, &want, &core);
}
