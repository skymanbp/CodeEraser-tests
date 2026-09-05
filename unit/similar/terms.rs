use super::*;

/// The term road is the one thing index and query share: a change to
/// splitting moves every term below, so the cases here are the ones
/// the spec names (§三 拆词).
#[test]
fn identifiers_split_at_case_underscore_and_digit_boundaries() {
    let cases: [(&str, &[&str]); 7] = [
        ("parseJSONFile", &["parse", "json", "file"]),
        ("http2_server", &["http", "2", "server"]),
        ("(T) add", &["t", "add"]),
        ("getX", &["get", "x"]),
        ("ABC", &["abc"]),
        ("__init__", &["init"]),
        ("", &[]),
    ];
    for (ident, want) in cases {
        assert_eq!(split_ident(ident), want, "{ident}");
    }
}

#[test]
fn prose_drops_stop_words_but_never_identifier_pieces() {
    assert_eq!(
        prose_words("Returns the parsed JSON of a file"),
        ["returns", "parsed", "json", "file"]
    );
    // `is` / `get` are role words when they come from a name
    assert_eq!(split_ident("is_ready"), ["is", "ready"]);
}

#[test]
fn a_term_is_channel_tagged_and_stemmed() {
    assert_ne!(
        word_term(Channel::Name, "fetch"),
        word_term(Channel::Callee, "fetch"),
        "same word, two channels, two terms"
    );
    assert_eq!(
        word_term(Channel::Doc, "fetching"),
        word_term(Channel::Doc, "fetch"),
        "stemming folds inflections"
    );
    assert_ne!(
        feature_term(Channel::Shape, b"p:3"),
        feature_term(Channel::Shape, b"p:4")
    );
}

#[test]
fn the_evidence_row_order_is_n_p_c_d_s_l() {
    let labels: Vec<&str> = Channel::ALL.iter().map(|c| c.label()).collect();
    assert_eq!(labels, ["N", "P", "C", "D", "S", "L"]);
    assert_eq!(Channel::Callee.index(), 2);
    assert_eq!(
        (
            Channel::Name.weight(),
            Channel::Callee.weight(),
            Channel::Doc.weight()
        ),
        (3, 2, 1)
    );
}
