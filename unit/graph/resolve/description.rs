//! The DESCRIPTION reader's leg (plan v2.33 W2-text stage B): the frozen
//! c96ab3f6 reader (unit/graph/oracle_cfg/description.rs, mounted under
//! the frozen R rungs) reads each text from disk, lossy, as the ladder and
//! the declared-target pass did; the core reads the same text through
//! `inspect.description`. Ten thousand questions: every fourth a real
//! DESCRIPTION of the trees CE_RESOLVE_DIFF_TREES names, mutated (a
//! byte-order mark, CRLF, a cut, a doubled or dropped line, a non-ASCII or
//! White_Space character), the rest drawn line by line — `Package` spelled
//! empty, over a continuation line, as two words, after `Packaged:`, in
//! the wrong case; `Collate` names bare, quoted with either quote, with a
//! space inside, a quote never closed, over continuation lines that start
//! with a space or a tab.

use super::draw::{ODD, real_texts};
use super::files::{on_disk, real_or, scratch};
use super::{questions, table};
use crate::graph::ladder::frozen::description;
use serde_json::json;

/// Lines a DESCRIPTION is drawn from.
const LINES: &str = "Package: pkg¦Package:   spaced  ¦Package:¦  pkg2¦Package: two words¦Packaged: 2024-01-01; x¦Package:pkg3¦package: lower¦ Package: indented¦Package: n\u{a0}x¦Package: é¦Collate:¦Collate: a.R b.R¦Collate: 'a b.R' \"c.R\"¦Collate: 'unclosed.R¦    'x.R'¦\t\"y.R\" z.R¦  cont¦\tR/d.R¦Collate:\u{3000}a.R\u{85}b.R¦Title: A¦Version: 1.0¦Encoding: latin1¦Description: ü¦Collate: ''¦Collate: \"\" e.R¦";

#[test]
#[ignore = "needs a core: the differential gate"]
fn descriptions_agree() {
    let [lines, seps, rels] = [
        LINES,
        "\n¦\r\n¦\n\n¦\r",
        "DESCRIPTION¦pkg/DESCRIPTION¦é/DESCRIPTION",
    ]
    .map(table);
    let real = real_texts("DESCRIPTION");
    let root = scratch("description");
    let qs = questions(9, |d, i| {
        let mut text = real_or(d, &real, i, |d| d.words(&lines, &seps, 8) + d.one(ODD));
        if d.chance(5) {
            text.insert(0, '\u{feff}');
        }
        json!([d.one(&rels), text])
    });
    println!("real DESCRIPTION texts: {}", real.len());
    on_disk("description", &qs, &root, 0, |root, rel| {
        json!(description::parse(root, rel).map(|d| (d.dir, d.package, d.collate)))
    });
}
