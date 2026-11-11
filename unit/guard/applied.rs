use super::*;
use crate::guard::envelope::{Envelope, ToolInput};
use crate::testutil::scratch;

fn edit(path: &std::path::Path, old: &str, new: &str, all: bool) -> Envelope {
    Envelope {
        tool_name: "Edit".into(),
        tool_input: ToolInput {
            file_path: path.to_string_lossy().into_owned(),
            old_string: old.into(),
            new_string: new.into(),
            replace_all: all,
            ..ToolInput::default()
        },
        ..Envelope::default()
    }
}

/// An Edit says why it cannot apply (GD3): the target is absent, a
/// directory sits at the path (unreadable), `old_string` is empty or
/// matches nothing, or it matches twice without `replace_all`.
#[test]
fn an_edit_that_cannot_apply_says_why() {
    let dir = scratch("guard-applied-why");
    let f = dir.join("a.py");
    std::fs::write(&f, "x = 1\ny = 1\n").expect("seed");
    let why = |e: Envelope| apply(&e).err().map(Unapplied::code);
    assert_eq!(why(edit(&dir.join("gone.py"), "x", "z", false)), Some("absent"));
    assert_eq!(why(edit(&dir, "x", "z", false)), Some("unreadable"));
    assert_eq!(why(edit(&f, "", "z", false)), Some("no_match"));
    assert_eq!(why(edit(&f, "q = 2", "z", false)), Some("no_match"));
    assert_eq!(why(edit(&f, " = 1", "z", false)), Some("ambiguous"));
    assert_eq!(why(edit(&f, " = 1", " = 2", true)), None);
}

/// The Edit's text is applied on the CRLF-normalised file and its size
/// before comes off the same read; a Write's size before is counted
/// from the disk only when asked, absent = None (absent is not empty).
#[test]
fn the_size_before_comes_off_the_one_read() {
    let dir = scratch("guard-applied-before");
    let f = dir.join("a.py");
    std::fs::write(&f, "x = 1\r\ny = 2\r\nz = 3").expect("seed");
    let a = apply(&edit(&f, "y = 2\n", "", false)).expect("applies");
    assert_eq!((a.text.as_str(), a.before), ("x = 1\nz = 3", Some(3)));
    let write = |p: &std::path::Path| Envelope {
        tool_name: "Write".into(),
        tool_input: ToolInput {
            file_path: p.to_string_lossy().into_owned(),
            content: "new\n".into(),
            ..ToolInput::default()
        },
        ..Envelope::default()
    };
    let w = write(&f);
    let a = apply(&w).expect("a write always applies");
    assert_eq!((a.text.as_str(), a.before), ("new\n", None));
    assert_eq!(before_lines(&w, &a), Ok(Some(3)));
    let n = write(&dir.join("new.py"));
    assert_eq!(before_lines(&n, &apply(&n).expect("applies")), Ok(None));
}
