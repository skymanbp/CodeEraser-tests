//! The id_hash ↔ ids coupling of the live page reader (plan v2.30 step
//! 5; moved here from html_head_tests.rs when plan v2.33 W2-text stage H
//! froze the rest of that battery onto the a0cb6e13 copy): the hashed
//! projection and the consulted projection must move together, or an id
//! edit stops re-firing the sweep (the md slug_hash discipline,
//! md_tests.rs). One text, `why @@ a @@ b` per row (the clone gate reads
//! a tuple table's rhythm; a text has none): two documents whose hashes
//! must differ exactly when their consulted id lists differ. The hash is
//! a resolve_key input, so it must also be the a0cb6e13 one, byte for
//! byte (the frozen reader).

use super::{id_hash, ids};
use crate::graph::ladder::frozen::html_head as frozen;

const COUPLING: &str = r#"
a body edit holds the hash @@ <h2 id="a">x</h2> @@ <h2 id="a">y</h2>
an id edit moves it @@ <h2 id="a">x</h2> @@ <h2 id="b">x</h2>
an id added moves it @@ <h2 id="a">x</h2> @@ <h2 id="a">x</h2><p id="c"></p>
document order is part of the projection @@ <p id="a"></p><p id="b"></p> @@ <p id="b"></p><p id="a"></p>
a duplicate id is a second entry @@ <p id="a"></p> @@ <p id="a"></p><p id="a"></p>
an empty id is no entry @@ <p id="a"></p> @@ <p id="a"></p><p id=""></p>
"#;

#[test]
fn id_hash_moves_exactly_when_the_id_set_moves() {
    for line in COUPLING.trim().lines() {
        let [why, a, b] = <[&str; 3]>::try_from(line.split(" @@ ").collect::<Vec<_>>())
            .unwrap_or_else(|_| panic!("why @@ a @@ b: {line}"));
        assert_eq!(
            id_hash(a) == id_hash(b),
            ids(a) == ids(b),
            "projection coupling broke: {why}"
        );
        for doc in [a, b] {
            assert_eq!(
                id_hash(doc),
                frozen::id_hash(doc),
                "the key input moved: {why}"
            );
        }
    }
    assert_eq!(
        ids(r#"<p id="a"></p><div><p id="a"></p></div>"#),
        ["a", "a"],
        "every id, in document order"
    );
}
