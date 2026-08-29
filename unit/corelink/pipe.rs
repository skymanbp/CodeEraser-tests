use std::io::Cursor;

/// A newline-free flood is refused at the cap instead of being
/// accumulated: the pump takes the existing Err road (the
/// caller's degraded path, A9f) and hangs up, rather than growing
/// one String for the whole deadline window.
#[test]
fn a_newline_free_flood_is_refused_not_accumulated() {
    let rx = super::pump(Cursor::new(vec![b'x'; 200]), 64);
    let first = rx.recv().expect("the pump answers once");
    assert!(first.is_err(), "over-cap line must not arrive as a reply");
    assert!(rx.recv().is_err(), "the pump hangs up after refusing");
}

/// …and a line AT the ceiling still rides — the cap detects a
/// wedge, it does not budget legitimate replies (an off-by-one
/// here would silently shrink the contract's frame size).
#[test]
fn a_line_at_the_ceiling_still_arrives() {
    let mut input = vec![b'y'; 64];
    input.push(b'\n');
    let rx = super::pump(Cursor::new(input), 64);
    assert_eq!(rx.recv().expect("delivered").expect("no error").len(), 64);
}
