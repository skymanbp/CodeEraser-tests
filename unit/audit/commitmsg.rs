//! `ce commitmsg`'s message intake: comment lines are blanked in
//! place, never removed, so a site's line stays the file's own line;
//! a fixed prefix is whatever the repository set, of any length, and
//! the deprecated `auto` is read back from what git wrote — the
//! `auto-*` rows below are git 2.52's own output (a commit-msg hook
//! copying its file, `core.commentChar=auto`, 2026-09-27), so the
//! reader is pinned to the writer.

use super::{Prefix, uncommented};

/// One case per `====` block: `name @@ prefix` (`auto`, or the fixed
/// string) over the message, `----`, the expected blanked message.
const CASES: &str = r#"
fixed-hash @@ #
Drop x
# a comment

x is no longer needed.
# another
----
Drop x


x is no longer needed.

====
fixed-string-matched-whole @@ ;;
x # y
;; c
; d
----
x # y

; d
====
auto-status-block @@ auto
# heading first
subject

; semi line

@ Please enter the commit message for your changes. Lines starting
@ with '@' will be ignored, and an empty message aborts the commit.
@
@ On branch master
----
# heading first
subject

; semi line





====
auto-default-hash @@ auto
subject

body line

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch master
----
subject

body line





====
auto-lone-trailing-line-is-prose @@ auto
subject

# footnote
----
subject

# footnote
====
auto-scissors-cut @@ auto
subject

# user hash line
; user semi line

@ Please enter the commit message for your changes. Lines starting
@
@ ------------------------ >8 ------------------------
@ Do not modify or remove the line above.
@ Everything below it will be ignored.
diff --git a/f b/f
+v
----
subject

# user hash line
; user semi line








====
auto-block-then-blank-tail @@ auto
subject

@ Please enter the commit message for your changes. Lines starting
@ with '@' will be ignored, and an empty message aborts the commit.
@


----
subject





====
auto-no-editor-keeps-every-line @@ auto
# heading first
subject

; semi line
----
# heading first
subject

; semi line
"#;

#[test]
fn comment_lines_are_blanked_the_way_git_strips_them() {
    for (head, body) in crate::testutil::sections(CASES) {
        let (name, prefix) = head.split_once(" @@ ").expect("name @@ prefix");
        let prefix = match prefix {
            "auto" => Prefix::Auto,
            fixed => Prefix::Fixed(fixed.to_string()),
        };
        let (message, want) = body.split_once("\n----\n").expect("message ---- expected");
        assert_eq!(uncommented(message, &prefix), want, "{name}");
    }
}
