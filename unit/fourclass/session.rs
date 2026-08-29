use super::*;

#[test]
fn name_status_pairs_and_language_filter() {
    let raw = "A\0new.rs\0D\0gone.py\0M\0kept.md\0R100\0old.rs\0moved.rs\0\
                   C100\0src.rs\0copy.rs\0M\0skip.json\0M\0after_copy.rs\0";
    let pairs = parse_name_status(raw);
    assert_eq!(
        pairs,
        vec![
            (None, Some("new.rs".into())),
            (Some("gone.py".into()), None),
            (Some("kept.md".into()), Some("kept.md".into())),
            (Some("old.rs".into()), Some("moved.rs".into())),
            (None, Some("copy.rs".into())),
            // the record AFTER the copy proves the stream stayed
            // in sync (F13's failure mode was desync from here on)
            (Some("after_copy.rs".into()), Some("after_copy.rs".into())),
        ]
    );
}
