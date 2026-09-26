//! The document reader pinned (plan v2.30 step 5): what the HTML rungs
//! read off a page's own text — the origin that serves it and the tree
//! root that origin derives, its `<base href>`, the two URL readings —
//! and the id_hash ↔ ids coupling: the hashed projection and the
//! consulted projection must move together, or an id edit stops
//! re-firing the sweep (the md slug_hash discipline, md_tests.rs).
//! Each battery is one text, `a @@ b @@ …` per row (the clone gate
//! reads a tuple table's rhythm; a text has none).

use super::{decode_refs, id_hash, ids, read, split_origin};

/// The rows of one battery, each split into exactly N fields; a row
/// of another width names the shape it should have had.
fn rows<const N: usize>(table: &'static str, shape: &str) -> Vec<[&'static str; N]> {
    table
        .trim()
        .lines()
        .map(|line| {
            <[&str; N]>::try_from(line.split(" @@ ").collect::<Vec<_>>())
                .unwrap_or_else(|_| panic!("{shape}: {line}"))
        })
        .collect()
}

/// `-` is None, `.` the tree root.
fn opt(word: &str) -> Option<&str> {
    match word {
        "-" => None,
        "." => Some(""),
        w => Some(w),
    }
}

/// `path @@ lang @@ head @@ host @@ root @@ why`: the page at the tree
/// path, its `<html lang>` (`-` none), with that head, derives that host
/// and root.
const SERVED: &str = r#"
site/how/index.html @@ en @@ <meta property="og:url" content="https://codeeraser.dev/how/"> @@ codeeraser.dev @@ site @@ a directory URL serves its index page
index.html @@ - @@ <link rel="canonical" href="https://a.example/"> @@ a.example @@ . @@ the tree root itself
site/zh/index.html @@ zh-Hans @@ <link rel="alternate" hreflang="en" href="https://a.example/"><link rel="alternate" hreflang="zh" href="https://a.example/zh/index.html"> @@ a.example @@ site @@ the alternate in the page's own language names it
site/zh/index.html @@ zh-Hans @@ <link rel="alternate" hreflang="en" href="https://a.example/"> @@ - @@ - @@ another language's alternate names another page
site/zh/index.html @@ - @@ <link rel="alternate" hreflang="en" href="https://a.example/"><link rel="alternate" hreflang="zh" href="https://a.example/zh/index.html"> @@ a.example @@ site @@ no language declared: the alternate sharing the longest path
site/about.html @@ - @@ <link rel="canonical" href="https://a.example/about"> @@ a.example @@ site @@ an extensionless URL serves name.html
site/index.html @@ - @@ <link rel="canonical" href="https://a.example/other/"><meta property="og:url" content="https://a.example/"> @@ - @@ - @@ a canonical naming another page derives nothing, and it is the one read
site/index.html @@ - @@ <link rel="canonical" href="https://A.Example/site/index.html?v=2#top"> @@ a.example @@ . @@ the host lowercased; query and fragment are not the path
site/index.html @@ - @@ <link rel="canonical" href="/site/"> @@ - @@ - @@ a URL without an origin names no host
site/index.html @@ - @@ <title>x</title> @@ - @@ - @@ a page that says nothing
"#;

#[test]
fn a_page_derives_host_and_root_from_the_url_that_names_itself() {
    for [from, lang, head, host, root, why] in
        rows(SERVED, "path @@ lang @@ head @@ host @@ root @@ why")
    {
        let lang = opt(lang).map_or(String::new(), |l| format!(" lang=\"{l}\""));
        let text = format!("<!doctype html><html{lang}><head>{head}</head><body></body></html>");
        let doc = read(from, &text);
        assert_eq!(
            (doc.host.as_deref(), doc.root.as_deref()),
            (opt(host), opt(root)),
            "{why}"
        );
    }
}

#[test]
fn the_first_base_href_is_read_as_written_but_decoded() {
    let text = r#"<html><head><base href="/docs/a&amp;b/"><base href="/other/"></head></html>"#;
    assert_eq!(read("x.html", text).base.as_deref(), Some("/docs/a&b/"));
    let bare = r#"<html><head><base target="_blank"></head></html>"#;
    assert_eq!(read("x.html", bare).base, None);
}

/// `url @@ host @@ path`: the origin split off, or `-` for a URL that
/// carries none. The third row's userinfo is a fixture name, no
/// credential: whatever stands before the `@` is dropped.
const ORIGINS: &str = r#"
https://A.Example/how/#x @@ a.example @@ /how/#x
//cdn.example/x.js @@ cdn.example @@ /x.js
http://someone@h.example/ @@ h.example @@ /
https://h.example?q=1 @@ h.example @@ ?q=1
mailto:a@b.example @@ - @@ -
javascript:void(0) @@ - @@ -
how/index.html @@ - @@ -
/how/ @@ - @@ -
://x @@ - @@ -
"#;

#[test]
fn an_origin_splits_off_an_absolute_or_protocol_relative_url_only() {
    for [url, host, path] in rows(ORIGINS, "url @@ host @@ path") {
        let got = split_origin(url);
        let want = opt(host).map(|h| (h, path));
        assert_eq!(got.as_ref().map(|(h, p)| (h.as_str(), *p)), want, "{url}");
    }
    let bare = split_origin("http://someone@h.example");
    let bare = bare.as_ref().map(|(h, p)| (h.as_str(), *p));
    assert_eq!(bare, Some(("h.example", "")), "an empty path");
}

/// `written @@ decoded`.
const REFERENCES: &str = r#"
a&amp;b @@ a&b
&lt;&gt;&quot;&apos; @@ <>"'
&#38;&#x26;&#X26; @@ &&&
a&b @@ a&b
&amp @@ &amp
&nope; @@ &nope;
&#0; @@ &#0;
&#xZZ; @@ &#xZZ;
"#;

#[test]
fn character_references_decode_and_the_rest_stays_written() {
    for [raw, want] in rows(REFERENCES, "written @@ decoded") {
        assert_eq!(decode_refs(raw), want, "{raw}");
    }
}

/// `why @@ a @@ b`: two documents whose hashes must differ exactly when
/// their consulted id lists differ.
const COUPLING: &str = r#"
a body edit holds the hash @@ <h2 id="a">x</h2> @@ <h2 id="a">y</h2>
an id edit moves it @@ <h2 id="a">x</h2> @@ <h2 id="b">x</h2>
an id added moves it @@ <h2 id="a">x</h2> @@ <h2 id="a">x</h2><p id="c"></p>
document order is part of the projection @@ <p id="a"></p><p id="b"></p> @@ <p id="b"></p><p id="a"></p>
a duplicate id is a second entry @@ <p id="a"></p> @@ <p id="a"></p><p id="a"></p>
an empty id is no entry @@ <p id="a"></p> @@ <p id="a"></p><p id=""></p>
"#;

#[test]
fn id_hash_moves_exactly_when_the_id_set_moves() {
    for [why, a, b] in rows(COUPLING, "why @@ a @@ b") {
        assert_eq!(
            id_hash(a) == id_hash(b),
            ids(a) == ids(b),
            "projection coupling broke: {why}"
        );
    }
    assert_eq!(
        ids(r#"<p id="a"></p><div><p id="a"></p></div>"#),
        ["a", "a"],
        "every id, in document order"
    );
}
