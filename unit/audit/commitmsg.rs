//! `ce commitmsg`'s message intake: comment lines are blanked in
//! place, never removed, so a site's line stays the file's own line;
//! the prefix is whatever the repository set, of any length.

use super::uncommented;

#[test]
fn comment_lines_are_blanked_in_place_so_line_numbers_hold() {
    let msg = "Drop x\n# a comment\n\nx is no longer needed.\n# another\n";
    assert_eq!(
        uncommented(msg, "#"),
        "Drop x\n\n\nx is no longer needed.\n"
    );
}

#[test]
fn only_a_line_that_starts_with_the_prefix_is_a_comment() {
    // the character inside a line is prose; a multi-character prefix
    // (`core.commentString`) is matched whole
    assert_eq!(uncommented("x # y\n;; c\n; d\n", ";;"), "x # y\n\n; d");
}
