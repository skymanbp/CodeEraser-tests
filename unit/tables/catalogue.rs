//! The catalogue's hand-written reader (`TryFrom<serde_json::Value>`),
//! read through serde as the package reader reads it. One literal
//! table: a `document` value, then ` => `, then the exact sentence of
//! its refusal.

use super::DocCatalogueOwned;

const REFUSED: &str = r#"
{} => document.churn: no cochangeFileCap
{"churn": {"cochangeFileCap": -1}} => document.churn: no cochangeFileCap
{"churn": {"cochangeFileCap": "8"}} => document.churn: no cochangeFileCap
{"churn": {"cochangeFileCap": 8}} => document.erase-trail: no log
{"churn": {"cochangeFileCap": 8}, "erase-trail": {"log": "x"}} => document.erase-trail: no record
{"churn": {"cochangeFileCap": 8}, "erase-trail": {"record": "y"}} => document.erase-trail: no log
"#;

#[test]
fn a_document_without_the_cap_or_the_trail_words_is_refused_by_name() {
    for (value, says) in crate::testutil::case_table(REFUSED, 6) {
        let value: serde_json::Value = serde_json::from_str(value).expect(value);
        match serde_json::from_value::<DocCatalogueOwned>(value.clone()) {
            Ok(_) => panic!("{value} accepted; expected: {says}"),
            Err(e) => assert_eq!(e.to_string(), says, "{value}"),
        }
    }
}

#[test]
fn the_core_s_document_table_reads_whole() {
    let document = crate::testutil::package()["document"].clone();
    let read = serde_json::from_value::<DocCatalogueOwned>(document).expect("the core's catalogue");
    assert!(read.cochange_file_cap > 0, "a pairing cap");
    assert!(
        !read.trail_log.is_empty() && !read.trail_record.is_empty(),
        "the trail's two words"
    );
}
