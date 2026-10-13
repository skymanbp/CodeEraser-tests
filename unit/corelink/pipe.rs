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

/// The reply cap is twice the line the core states (`limits.caps.line_bytes`
/// in the tables golden, the value `CE.Protocol.Version.maxLineBytes`
/// answers): raising the line without the cap would cut in-contract
/// replies short.
#[test]
fn the_reply_cap_is_at_least_twice_the_protocol_line() {
    let golden = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../contracts/fixtures/tables/golden.ndjson"
    ));
    let reply = golden
        .lines()
        .nth(1)
        .expect("the tables golden's reply line");
    let doc: serde_json::Value = serde_json::from_str(reply).expect("the reply is JSON");
    let line = doc["limits"]["caps"]["line_bytes"]
        .as_u64()
        .expect("limits.caps.line_bytes");
    assert!(
        super::MAX_REPLY_BYTES >= 2 * line,
        "reply cap {} B is under twice the protocol line {line} B",
        super::MAX_REPLY_BYTES
    );
}
