use super::*;

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// v2.35 G8 (R1-8): the doc reader's fences follow the Markdown
/// scanner's rule (`graph::md::Fence`) — a ```` ``` ```` inside a `~~~`
/// block is content and does not close it, so the call after it is
/// still code (it used to toggle, closing the block early).
#[test]
fn a_fence_of_the_other_kind_is_content() {
    let doc = lines("prose\n~~~\n```\nfoo()\n~~~\nafter");
    assert_eq!(fenced(&doc), ["```", "foo()"]);
    assert_eq!(code_blocks(Lang::Java, &doc), ["```", "foo()"]);
    let longer = lines("````\n```\nbar()\n````");
    assert_eq!(
        fenced(&longer),
        ["```", "bar()"],
        "a shorter run closes nothing"
    );
}
