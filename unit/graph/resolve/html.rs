//! The HTML rungs' readings (plan v2.33 W2-text stage H), asked of the
//! core directly against the frozen a0cb6e13 page reader
//! (ladder::frozen::html_head): a text's character references decoded
//! and its origin split off, and a page's derived base, host and tree
//! root — the page drawn as text, the live syntax-tree walk's facts of it
//! sent to the core, the frozen reader run on the same text.

use super::draw::Draw;
use super::{check, questions, table};
use crate::graph::ladder::frozen::html_head;
use crate::graph::ladder::html_head::read;
use serde_json::{Value, json};

/// What a reference text is drawn from: the named references and their
/// near misses, numeric ones in range, out of it and signed, origins with
/// a userinfo (test fixture: a placeholder name, no credential), ports and
/// odd schemes, separators.
const REFS: &str = "&amp;¦&lt;¦&gt;¦&quot;¦&apos;¦&AMP;¦&amp¦&#38;¦&#x26;¦&#X26;¦&#+38;¦&#x+2f;¦&#0;¦&#xD800;¦&#x10FFFF;¦&#x110000;¦&#4294967296;¦&#4294967295;¦&#;¦&#x;¦&#-1;¦&#٣;¦&nope;¦&¦;¦#¦https://¦HTTP://¦//¦://¦1x://¦a+b.c-d://¦mailto:¦placeholder@¦@¦A.Example¦h.example¦:8080¦/¦?q=1¦#top¦é¦ ¦x";

#[test]
#[ignore = "needs a core: the differential gate"]
fn reference_readings_agree() {
    let words = table(REFS);
    let qs = questions(7, |d, _| json!(d.words(&words, &[""], 6)));
    check("htmlRefs", &qs, |q| {
        let t = q.as_str().expect("text");
        json!([html_head::decode_refs(t), html_head::split_origin(t)])
    });
}

/// What a page's path and head are drawn from (the userinfo URL is a test
/// fixture: a placeholder name, no credential).
const FROMS: &str = "index.html¦site/index.html¦site/how/index.html¦site/how/index.htm¦site/about.html¦site/zh/index.html¦a b/é.html¦docs/sub/x.html";
const URLS: &str = "https://a.example/¦https://A.Example/how/¦//a.example/site/how/¦https://a.example/about¦https://a.example/site/about.html¦https://a.example/zh/index.html¦https://placeholder@a.example/how/index.htm¦https://a.example:8080/how/¦https://a.example/how/?v=2#top¦https://a.example/%73ite/how/¦https://a.example/a&amp;b/¦/how/¦how/¦https://a.example¦https://b.example/other/¦https://a.example//how/¦https://a.example/sub/x¦https://a.example/é.html¦https://a.example/a%20b/%C3%A9.html";
const LANGS: &str = "en¦zh-Hans¦zh¦EN-us¦¦x-default";
const BASES: &str = "/site/¦a&amp;b/¦https://a.example/x/¦¦../";

/// One drawn head element.
fn element(d: &mut Draw, urls: &[&str]) -> String {
    let (url, lang) = (d.one(urls), d.one(&table(LANGS)));
    match d.under(5) {
        0 => format!("<link rel=\"canonical\" href=\"{url}\">"),
        1 => format!("<meta property=\"og:url\" content=\"{url}\">"),
        2 => format!("<base href=\"{}\">", d.one(&table(BASES))),
        _ => format!("<link rel=\"alternate\" hreflang=\"{lang}\" href=\"{url}\">"),
    }
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn page_readings_agree() {
    let (froms, urls) = (table(FROMS), table(URLS));
    let mut texts: Vec<(String, String)> = Vec::new();
    let qs = questions(8, |d, _| {
        let from = d.one(&froms).to_string();
        let lang = d.one(&table(LANGS));
        let mut text = format!("<html lang=\"{lang}\"><head>");
        for _ in 0..d.under(5) {
            text.push_str(&element(d, &urls));
        }
        text.push_str("</head></html>");
        let h = read(&text);
        let q = json!([from, h.lang, h.base, h.canonical, h.og_url, h.alternates]);
        texts.push((from, text));
        q
    });
    let mut frozen = texts.into_iter();
    check("htmlPage", &qs, |_| {
        let (from, text) = frozen.next().expect("one text per question");
        let doc = html_head::read(&from, &text);
        json!([doc.base, doc.host, doc.root]) as Value
    });
}
