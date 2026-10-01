use super::*;
use crate::scan::lang::Lang;

/// The first node of `kind` in preorder, with the source it was parsed
/// from kept beside it.
fn find<'t>(root: Node<'t>, kind: &str) -> Node<'t> {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.kind() == kind {
            return n;
        }
        stack.extend(ast::children(n).into_iter().rev());
    }
    panic!("no {kind}")
}

/// (own, text) of the first `kind` node in a TypeScript snippet.
fn columns(src: &str, kind: &str) -> (u64, u64) {
    let tree = ast::parse_lang(src, Lang::TypeScript).expect("parses");
    let root = tree.root_node();
    let toks = Tokens::of(root, src, Extras::Without);
    let n = find(root, kind);
    (toks.own(n), toks.text(n))
}

/// Ruling R1's own column: an operator is the binary expression's own
/// token, so `a < b` and `a > b` differ there — and the operands, named
/// children, never enter it.
#[test]
fn an_operator_is_its_expressions_own_token() {
    let (lt, gt) = (
        columns("x = a < b;", "binary_expression"),
        columns("x = a > b;", "binary_expression"),
    );
    assert_ne!(lt.0, gt.0, "own differs");
    assert_ne!(lt.1, gt.1, "text differs");
    let renamed = columns("x = c < d;", "binary_expression");
    assert_eq!(lt.0, renamed.0, "the operands are not own tokens");
    assert_ne!(lt.1, renamed.1, "but they are the text's");
    assert_eq!(lt.0, fnv1a(b"<"), "one own token, its bytes");
}

/// Ruling R1's text column: the token stream, not the bytes — `f()` and
/// `f( )` agree, a leaf's text is its one token's hash, and no anonymous
/// token at all is own 0.
#[test]
fn the_text_is_the_token_stream_not_the_bytes() {
    let tight = columns("f();", "arguments");
    let spaced = columns("f( );", "arguments");
    assert_eq!(tight, spaced, "whitespace is no token");
    assert_eq!(tight.1, fnv1a(b"(\0)"), "two tokens, one 0x00 between");
    let (own, text) = columns("f();", "identifier");
    assert_eq!(
        (own, text),
        (0, fnv1a(b"f")),
        "a leaf: no own token, its bytes"
    );
}

/// Without extras a comment is no token: a comment inside an argument
/// list leaves the list's text as it was.
#[test]
fn a_comment_is_no_token_without_extras() {
    let plain = columns("f(a, b);", "arguments");
    let commented = columns("f(a, /* why */ b);", "arguments");
    assert_eq!(plain, commented);
}

/// A run's synthetic root joins its tops' streams with one 0x00 between
/// two of them; no stream at all is text 0.
#[test]
fn a_run_joins_its_streams() {
    assert_eq!(joined_text(&[b"a", b"b\0c"]), fnv1a(b"a\0b\0c"));
    assert_eq!(joined_text(&[]), 0);
}
