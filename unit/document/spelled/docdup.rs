//! The W7 differential's docdup leg (plan v2.33 W7; mounted inside the
//! frozen docdup judge): a segment as `path:start-end kind` spelled by
//! the core from its `segs` row and the path sent, and by the frozen
//! `name` from the row this side read — a kind past the vocabulary (a
//! stale index's) or below it included.

use super::super::binder::Lists;
use super::super::kit::leg;
use super::name;
use crate::docdup::judge::candidates::SegRow;
use serde_json::json;

leg!(
    docdup_spells_a_segment_as_the_frozen_name,
    "docdup",
    0x771a,
    |leg| {
        let kinds = crate::docdup::spec::table().kind_names.len();
        let paths = leg.paths(1, 4);
        let mut rows = Vec::new();
        let mut segs = Vec::new();
        for i in 0..leg.rng.below(8) {
            let f = leg.rng.below(paths.len());
            let (a, b) = (leg.rng.below(400) as i64 - 3, leg.rng.below(400) as i64);
            let kind = leg.rng.below(kinds + 3) as i64 - 1;
            rows.push([i as i64, f as i64, a, b, kind]);
            let path = paths[f].clone();
            segs.push(SegRow {
                path,
                kind,
                start_line: a,
                end_line: b,
                words: 0,
                set: vec![],
            });
        }
        let refs = leg.references(
            &[("seg", vec![segs.len()]), ("path", vec![paths.len()])],
            200,
        );
        let names: Vec<String> = segs.iter().map(name).collect();
        let sent = json!({"path": paths});
        let frozen = Lists(vec![("seg", names), ("path", paths)]);
        leg.spell("docdup", (&sent, &json!({"segs": rows})), &frozen, refs);
    }
);
