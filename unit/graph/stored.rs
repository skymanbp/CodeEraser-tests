use super::*;

/// One case per line, `kind @@ raw @@ stored` (example fixtures: the
/// hosts are example.com, the words are placeholders): the userinfo
/// and the query's content go, a bare `?` and the fragment stay, a
/// code kind is as spelled, and a `mailto:` address is not an
/// authority.
const CASES: &str = "
url @@ https://someone:changeme@example.com/a/b?token=placeholder#frag @@ https://example.com/a/b?#frag
href @@ ../reset.html?token=placeholder @@ ../reset.html?
src @@ //cdn.example.com/x.js?v=3 @@ //cdn.example.com/x.js?
srcset @@ img@2x.png?w=2 @@ img@2x.png?
href @@ ?q=1 @@ ?
action @@ https://someone@example.com @@ https://example.com
link @@ mailto:someone@example.com @@ mailto:someone@example.com
link @@ dir//odd@name/x?y @@ dir//odd@name/x?
ref_def @@ https://example.com/x#only-a-fragment @@ https://example.com/x#only-a-fragment
use @@ a::b?c @@ a::b?c
import @@ https://esm.example.com/x@1?bundle @@ https://esm.example.com/x@1?bundle
";

#[test]
fn stored_specs_keep_what_a_rung_reads_and_nothing_a_rung_ignores() {
    for line in CASES.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.split(" @@ ").collect();
        assert_eq!(parts.len(), 3, "kind @@ raw @@ stored: {line}");
        assert_eq!(spec(parts[0], parts[1]), parts[2], "{line}");
    }
}
