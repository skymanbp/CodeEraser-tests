use super::{FRAME_MAX, serve_stream};
use serde_json::Value;
use std::io::Cursor;
use std::path::Path;

/// The frames the loop wrote back for one canned stdin. `.` as the
/// root is safe: no method exercised here reaches the filesystem.
fn run(input: &[u8]) -> Vec<Value> {
    let mut out = Vec::new();
    serve_stream(Path::new("."), Cursor::new(input.to_vec()), &mut out)
        .expect("a malformed frame must not end the loop with an error");
    String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).expect("every frame we write is JSON"))
        .collect()
}

fn code(frame: &Value) -> i64 {
    frame["error"]["code"].as_i64().expect("an error frame")
}

/// Serve `input`, then assert how many frames came back and what
/// the first one's error code was — the opening three lines of
/// every error probe below. Written out at each site they were one
/// clone block (dedup gate, this batch); `why` keeps each probe's
/// own reason for the count it demands.
fn served(input: &[u8], frames: usize, first: i64, why: &str) -> Vec<Value> {
    let out = run(input);
    assert_eq!(out.len(), frames, "{why}: {out:?}");
    assert_eq!(code(&out[0]), first);
    out
}

#[test]
fn unparseable_frame_answers_parse_error() {
    let why = "dropping it silently strands the client";
    let out = served(b"{ this is not json\n", 1, -32700, why);
    assert!(out[0]["id"].is_null(), "id is unknowable, so it is null");
}

/// The defect this closes: `method` was read before `id`, so these
/// two frames returned None and the id was never answered.
#[test]
fn id_without_usable_method_answers_invalid_request() {
    let frames = b"{\"jsonrpc\":\"2.0\",\"id\":7}\n{\"id\":8,\"method\":42}\n";
    let out = served(frames, 2, -32600, "both ids must come back");
    assert_eq!((code(&out[0]), &out[0]["id"]), (-32600, &Value::from(7)));
    assert_eq!((code(&out[1]), &out[1]["id"]), (-32600, &Value::from(8)));
}

/// Silence is still correct where JSON-RPC demands it: no id, no
/// reply — including for a notification that is itself malformed.
#[test]
fn frames_without_an_id_stay_silent() {
    let out = run(
        b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
                        {\"jsonrpc\":\"2.0\",\"id\":null,\"method\":\"nope\"}\n\
                        {\"jsonrpc\":\"2.0\"}\n",
    );
    assert!(out.is_empty(), "notifications got {} replies", out.len());
}

/// One stray byte used to reach `line?` as InvalidData and kill the
/// server, taking every later frame with it.
#[test]
fn invalid_utf8_frame_does_not_kill_the_loop() {
    let frames = b"{\"id\":1,\xff\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n";
    let why = "the frame after the bad byte must be served";
    let out = served(frames, 2, -32700, why);
    assert!(out[1]["result"]["tools"].is_array(), "id 2 answered");
}

/// Without a ceiling this line is buffered whole; with one it is
/// refused, and serving stops because its tail cannot be told apart
/// from a fresh frame.
#[test]
fn newline_free_flood_is_refused_at_the_ceiling() {
    let mut input = vec![b'x'; FRAME_MAX as usize + 8];
    input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}\n");
    served(&input, 1, -32600, "the tail must not be parsed as a frame");
}
