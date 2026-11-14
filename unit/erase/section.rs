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
    let at = |line| links_to(&ls, "b.md", text, line);
    assert_eq!((at(1), at(3), at(7), at(11)), (0, 5, 2, 1));
    assert_eq!(links_to(&ls, "c.md", text, 7), 0, "another file's links");
}

/// An HTML segment's section is the innermost element with an `id`
/// that holds its first line.
#[test]
fn an_html_segment_counts_the_links_to_its_innermost_id() {
    let text = "<div id=\"outer\">\n<p>a</p>\n<section id=\"inner\">\n<p>b</p>\n</section>\n</div>\n<p>c</p>\n";
    let ls = links(&[("p.html", "outer", 1), ("p.html", "inner", 3)]);
    let at = |line| links_to(&ls, "p.html", text, line);
    assert_eq!((at(2), at(4), at(7)), (1, 3, 0));
}
