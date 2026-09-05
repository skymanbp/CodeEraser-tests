use super::*;

/// A method inside an impl: the innermost seat of a line in the
/// method is the method, of the impl's own line the impl.
const SRC: &str = "\
struct Repo;

impl Repo {
    /// Load the user row by id.
    fn load_user(&self, id: u64) -> u64 {
        id
    }
}

fn helper() {}
";

#[test]
fn at_parses_file_line_and_refuses_what_is_not_one() {
    let ok = [
        ("src/a.rs:12", "src/a.rs", 12),
        ("C:\\w\\a.rs:3", "C:/w/a.rs", 3),
        ("a:b.rs:1", "a:b.rs", 1),
    ];
    for (spec, path, line) in ok {
        assert_eq!(
            Ask::at(spec).unwrap(),
            Ask::At {
                path: path.into(),
                line
            },
            "{spec}"
        );
    }
    for bad in ["a.rs", "a.rs:", ":7", "a.rs:x", "a.rs:0"] {
        assert!(Ask::at(bad).is_err(), "{bad}");
    }
}

#[test]
fn from_parts_wants_exactly_one_ask() {
    assert_eq!(
        Ask::from_parts(None, Some("fetch user"), None).unwrap(),
        Ask::Text("fetch user".into())
    );
    assert_eq!(
        Ask::from_parts(None, None, Some("f/1")).unwrap(),
        Ask::Unit("f/1".into())
    );
    let refused = [
        Ask::from_parts(None, None, None),
        Ask::from_parts(Some("a.rs:1"), Some("t"), None),
        Ask::from_parts(None, Some("  "), None),
        Ask::from_parts(None, None, Some("")),
    ];
    for r in refused {
        let why = r.unwrap_err().to_string();
        assert!(why.contains("exactly one"), "{why}");
    }
}

#[test]
fn resolution_finds_the_innermost_seat_and_names_the_ambiguous_key() {
    // the on-disk road (walk + refresh over a scratch tree): the seats
    // a face resolves against are the ones `ce similar` itself reads
    let dir = crate::testutil::scratch("similar-query");
    std::fs::write(dir.join("r.rs"), SRC).expect("r.rs");
    std::fs::write(dir.join("s.rs"), "fn helper() {}\n").expect("s.rs");
    let (idx, _db) = crate::dedup::refreshed_index(&dir, None).expect("index");
    let reader = Reader::open(&idx).expect("reader");
    let method = resolve(&reader, &Ask::at("r.rs:6").unwrap()).expect("method line");
    assert_eq!(method.label, "r.rs:5-7 load_user/2");
    assert!(method.seat.is_some() && !method.terms.is_empty());
    let block = resolve(&reader, &Ask::at("r.rs:3").unwrap()).expect("impl line");
    assert!(block.label.ends_with(" impl Repo"), "{}", block.label);
    let miss = resolve(&reader, &Ask::at("r.rs:2").unwrap())
        .err()
        .expect("no unit");
    assert!(miss.to_string().contains("no indexed unit at r.rs:2"));
    let one = resolve(&reader, &Ask::Unit("load_user/2".into())).expect("unique key");
    assert_eq!(one.seat, method.seat);
    let two = resolve(&reader, &Ask::Unit("helper/0".into()))
        .err()
        .expect("two seats");
    let why = two.to_string();
    assert!(
        why.contains("2 units") && why.contains("r.rs:") && why.contains("s.rs:"),
        "{why}"
    );
    assert!(resolve(&reader, &Ask::Unit("nobody/9".into())).is_err());
}

#[test]
fn a_text_is_name_and_doc_evidence_only_and_excludes_no_seat() {
    let q = text_terms("Fetch the user record");
    assert!(
        q.iter()
            .all(|t| matches!(t.channel, Channel::Name | Channel::Doc))
    );
    let stems: std::collections::HashSet<u64> = q.iter().map(|t| t.term).collect();
    assert!(stems.contains(&terms::word_term(Channel::Name, "fetch")));
    assert!(stems.contains(&terms::word_term(Channel::Doc, "user")));
    assert!(
        !stems.contains(&terms::word_term(Channel::Doc, "the")),
        "stop word"
    );
    assert!(q.iter().all(|t| t.spelled));
    assert!(text_terms("the a").is_empty());
}
