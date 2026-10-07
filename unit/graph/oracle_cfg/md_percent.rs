//! md_slug.rs's `percent_decode` as it stood at a0cb6e13, byte for byte
//! (plan v2.33 W2-text stage H): its last live readers, the HTML rungs
//! and the Markdown twins they called, moved into the core
//! (CE.Resolve.Url), so the frozen Markdown and HTML rungs read it here,
//! mounted as the frozen Markdown module's `slug` (oracle/md.rs).

/// `%XX` escapes decoded — a destination is percent-encoded in the
/// source and plain in the tree; an escape that is not two hex digits
/// stays as written, and a result that is not UTF-8 leaves the whole
/// text untouched — never a guess.
pub(crate) fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(b) => {
                out.push(b);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}
