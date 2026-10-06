//! The W7 differential's similar leg (plan v2.33 W7; mounted inside the
//! frozen similar face): the query's label, each seat's place and key
//! and the reasons spelled by the core and by the frozen `Strings`.

use super::super::kit::leg;
use super::Strings;
use serde_json::json;

leg!(
    similar_spells_as_the_frozen_face,
    "similar",
    0x7714,
    |leg| {
        let label = leg.word();
        let at = leg.paths(0, 6);
        let key = leg.words(6);
        let (why, texts) = leg.reasons();
        let sent = json!({"label": label, "at": at, "key": key, "why": texts});
        let drawn = [
            ("label", vec![]),
            ("key", vec![key.len()]),
            ("at", vec![at.len()]),
            ("why", vec![texts.len()]),
        ];
        let frozen = Strings {
            label,
            at,
            key,
            why,
        };
        leg.spell_drawn("similar", &sent, &frozen, &drawn);
    }
);
