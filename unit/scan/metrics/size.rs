use super::*;

/// The line-count rule's two entries agree with `str::lines().count()`
/// (the guard's hard budget and the scan's hard line count alike): on
/// the edge texts, on bytes that are not UTF-8 (a Latin-1 file is not
/// unread), and on a buffer that ends exactly on the stream's chunk
/// edge, with and without a final newline.
#[test]
fn both_entries_count_lines_as_str_lines_does() {
    let stream = |b: &[u8]| count_lines_in(b).expect("a slice reads");
    for text in [
        "",
        "a",
        "a\n",
        "a\r\n",
        "\n",
        "a\n\nb",
        "\r",
        "a\nb",
        "a\r\nb\r\n",
    ] {
        let want = text.lines().count();
        assert_eq!(
            (total_lines(text.as_bytes()), stream(text.as_bytes())),
            (want, want),
            "{text:?}"
        );
    }
    assert_eq!((total_lines(b"caf\xe9\n"), stream(b"caf\xe9\n")), (1, 1));
    for tail in ["\n", "x"] {
        let edge = format!("{}{tail}", "y".repeat(CHUNK - 1));
        let twice = edge.repeat(2);
        for text in [&edge, &twice] {
            let want = text.lines().count();
            assert_eq!(
                (total_lines(text.as_bytes()), stream(text.as_bytes())),
                (want, want)
            );
        }
    }
}
