use super::*;

/// A Rust line with a string, a char and a comment: literals alone
/// blank the two literals and keep the comment; with comments, the
/// comment goes too. Newlines survive, so line counts hold, and code
/// outside every span is untouched.
#[test]
fn literals_and_comments_blank_to_spaces_with_newlines_kept() {
    let text = "let a = \"alpha\"; // beta\nlet c = 'x'; let d = alpha;\n";
    assert_eq!(
        blanked(text, Lang::Rust, Opaque::Literals),
        "let a =        ; // beta\nlet c =    ; let d = alpha;\n"
    );
    assert_eq!(
        blanked(text, Lang::Rust, Opaque::LiteralsAndComments),
        "let a =        ;        \nlet c =    ; let d = alpha;\n"
    );
    assert!(
        spans("x", Lang::Yaml, Opaque::Literals).is_empty(),
        "no grammar, nothing masked"
    );
}
