//! The W7 differential's join leg (plan v2.33 W7; mounted inside the
//! frozen join face): the paths, each unit row's two keys and the
//! reasons spelled by the core and by the frozen `Names`.

use super::super::kit::leg;
use super::Names;
use serde_json::json;

leg!(join_spells_as_the_frozen_face, "join", 0x7715, |leg| {
    let paths = leg.paths(0, 6);
    let keys: Vec<[String; 2]> = (0..leg.rng.below(5))
        .map(|_| [leg.word(), leg.word()])
        .collect();
    let (why, texts) = leg.reasons();
    let sent = json!({"path": paths, "key": keys, "why": texts});
    let drawn = [
        ("key", vec![keys.len(), 2]),
        ("path", vec![paths.len()]),
        ("why", vec![texts.len()]),
    ];
    let frozen = Names { paths, keys, why };
    leg.spell_drawn("join", &sent, &frozen, &drawn);
});
