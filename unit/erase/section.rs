use super::*;

fn links(rows: &[(&str, &str, i64)]) -> Links {
    rows.iter()
        .map(|(p, u, n)| ((p.to_string(), u.to_string()), *n))
        .collect()
}

/// A Markdown segment's section is the nearest heading at or above its
/// first line, named by the ladder's slug (a repeated heading takes its
/// -N suffix); a segment above the first heading has no section.
#[test]
fn a_markdown_segment_counts_the_links_to_its_heading() {
    let text = "intro\n\n# B\n\n## Install\n\nbody\n\n## Install\n\nagain\n";
    let ls = links(&[
        ("b.md", "install", 2),
        ("b.md", "install-1", 1),
        ("b.md", "b", 5),
    ]);
    let at = |line| links_to(&ls, "b.md", text, (line, line));
    assert_eq!((at(1), at(3), at(7), at(11)), (0, 5, 2, 1));
    assert_eq!(
        links_to(&ls, "c.md", text, (7, 7)),
        0,
        "another file's links"
    );
}

/// An HTML segment's section is the innermost element with an `id`
/// that holds its first line.
#[test]
fn an_html_segment_counts_the_links_to_its_innermost_id() {
    let text = "<div id=\"outer\">\n<p>a</p>\n<section id=\"inner\">\n<p>b</p>\n</section>\n</div>\n<p>c</p>\n";
    let ls = links(&[("p.html", "outer", 1), ("p.html", "inner", 3)]);
    let at = |line| links_to(&ls, "p.html", text, (line, line));
    assert_eq!((at(2), at(4), at(7)), (1, 3, 0));
}

/// The targets a Markdown segment answers for besides its heading
/// (release 1.10.0), one row each: (text, segment rows, the
/// count). Links: `install` 2, `setup` 3, `in` 5, `below` 7, `sub` 11,
/// `dup` 13.
#[test]
fn a_markdown_segment_counts_the_anchors_and_headings_it_would_remove() {
    let ls = links(&[
        ("d.md", "install", 2),
        ("d.md", "setup", 3),
        ("d.md", "in", 5),
        ("d.md", "below", 7),
        ("d.md", "sub", 11),
        ("d.md", "dup", 13),
    ]);
    let rows: [(&str, (i64, i64), i64); 6] = [
        // an anchor on the section's heading row
        ("## Install <a id=\"setup\"></a>\n\nbody\n", (3, 3), 5),
        // an anchor inside the segment
        ("## Install\n\nbody <a id=\"in\"></a> more\n", (3, 3), 7),
        // an anchor below the segment in the same section: not counted
        (
            "## Install\n\nbody\n\n<a id=\"below\"></a> tail\n",
            (3, 3),
            2,
        ),
        // a heading inside the segment's rows (none above it)
        ("intro\n\n## Sub\n\nx\n", (1, 5), 11),
        // one id written twice: one target
        (
            "## Install\n\n<a id=\"dup\"></a> a <a id=\"dup\"></a>\n",
            (3, 3),
            15,
        ),
        // no heading above, an anchor inside
        ("lead <a id=\"in\"></a> text\n", (1, 1), 5),
    ];
    for (text, seg, want) in rows {
        assert_eq!(links_to(&ls, "d.md", text, seg), want, "{text:?} {seg:?}");
    }
}

/// An HTML segment answers for every id-carrying element that opens on
/// one of its rows besides its innermost holder: a nested id opening
/// inside the segment counts, a sibling id after it does not.
#[test]
fn an_html_segment_counts_the_ids_that_open_inside_it() {
    let text = "<div id=\"outer\">\n<p>a</p>\n<span id=\"nested\">b</span>\n<p>c</p>\n</div>\n<p id=\"after\">d</p>\n";
    let ls = links(&[
        ("p.html", "outer", 1),
        ("p.html", "nested", 2),
        ("p.html", "after", 4),
    ]);
    assert_eq!(links_to(&ls, "p.html", text, (2, 4)), 3);
}
