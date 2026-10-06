//! The W7 differential's deadcode leg (plan v2.33 W7; mounted inside
//! the frozen deadcode document): a node's path, a section's `path#unit`
//! label, the advisory's names and the reasons spelled by the core
//! (`node_name` its own rule over the paths and units sent) and by the
//! frozen `Names`, in the deadcode and the graph-screen families.

use super::super::kit::{Leg, leg};
use super::Names;
use crate::graph::nodes::Node;
use serde_json::json;

/// A node: a file (no unit) or a section of one.
fn node(leg: &mut Leg) -> Node {
    let unit = if leg.rng.chance(40) {
        leg.word()
    } else {
        String::new()
    };
    Node {
        path: leg.path(),
        unit,
        kind: 0,
        foreign: false,
        asset: false,
    }
}

leg!(
    deadcode_spells_as_the_frozen_document,
    "deadcode",
    0x7716,
    |leg| {
        let nodes = leg.many(8, node);
        let symbols = leg.words(4);
        let (paths, units): (Vec<&String>, Vec<&String>) =
            nodes.iter().map(|n| (&n.path, &n.unit)).unzip();
        let (why, texts) = leg.reasons();
        let sent = json!({"path": paths, "node_unit": units, "symbol": symbols, "why": texts});
        let n = nodes.len();
        let drawn = [
            ("node_name", vec![n]),
            ("path", vec![n]),
            ("why", vec![texts.len()]),
            ("symbol", vec![symbols.len()]),
        ];
        let family = ["deadcode", "graphscreen"][leg.rng.below(2)];
        let frozen = Names {
            nodes: &nodes,
            symbols,
            why,
        };
        leg.spell_drawn(family, &sent, &frozen, &drawn);
    }
);
