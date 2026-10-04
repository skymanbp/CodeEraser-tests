use super::*;
use std::collections::BTreeMap;

fn doc(path: &str, key: &str, tf: u32) -> Doc {
    let mut terms = BTreeMap::new();
    terms.insert(1, (Channel::Name, tf));
    Doc {
        path: path.into(),
        bag: UnitBag {
            key: key.into(),
            nth: 0,
            start_line: 1,
            end_line: 1,
            terms,
        },
    }
}

/// Seat order is the tie order the core breaks by, so a corpus whose
/// seats are not strictly ascending by (path, key, nth) is refused; the
/// average length floors and never drops below one.
#[test]
fn seats_ascend_by_identity_and_the_average_floors() {
    let c = Corpus::build(vec![
        doc("a.rs", "f", 2),
        doc("a.rs", "g", 3),
        doc("b.rs", "f", 0),
    ])
    .expect("ascending");
    assert_eq!((c.n_docs(), c.avg_len(), c.df(1).expect("df")), (3, 1, 3));
    assert_eq!(c.posting(1).expect("posting"), [(0, 2), (1, 3), (2, 0)]);
    for docs in [
        vec![doc("b.rs", "f", 1), doc("a.rs", "f", 1)],
        vec![doc("a.rs", "f", 1), doc("a.rs", "f", 1)],
    ] {
        let err = Corpus::build(docs).err().expect("refused").to_string();
        assert!(err.contains("strictly ascending"), "{err}");
    }
}
