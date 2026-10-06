//! Frozen W7 oracle (plan v2.33 W7; test-only): docdup: name (cli/src/docdup/judge/mod.rs), the seg spelling, copied byte for
//! byte from 1324c927 (spans cli/src/docdup/judge/mod.rs 164-175) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use crate::docdup::judge::candidates;

#[path = "../spelled/docdup.rs"]
mod spelled;

fn name(s: &candidates::SegRow) -> String {
    // .get, not a subscript: `kind` is a stored db column, and a
    // stale or corrupt `.ce/index.db` carrying a kind past this
    // side's vocabulary would abort a report rather than name the
    // row (the deadcode VERDICT_NAMES sibling, same class).
    let kind = crate::docdup::spec::table()
        .kind_names
        .get(s.kind as usize)
        .copied()
        .unwrap_or("kind?");
    format!("{}:{}-{} {}", s.path, s.start_line, s.end_line, kind)
}
