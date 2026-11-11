//! The CI gate walk the dead-file exam's per-language families share
//! (the samples, eval_deadfile.rs; the class docs, eval_deadfile_class.rs).

use super::{doc_path, generate};
use serde_json::{Value, json};

/// The per-language families' gate walk: every language of the
/// universes has exactly one filed doc of `family`, with `schema` and
/// `constants`, and every key `derive(lang, pool, filed)` returns is
/// the filed value.
pub fn assert_language_docs(
    family: &str,
    schema: &str,
    constants: &Value,
    derive: impl Fn(&str, &[Value], &Value) -> Value,
) {
    let (by, _) = generate::pools();
    let mut want = std::collections::BTreeSet::new();
    for (lang, pool) in &by {
        let path = doc_path(family, lang);
        let filed = crate::eval_support::load(&path);
        assert_eq!(
            (&filed["schema"], &filed["constants"]),
            (&json!(schema), constants),
            "{path}"
        );
        for (key, value) in derive(lang, pool, &filed).as_object().expect("object") {
            assert_eq!(&filed[key], value, "{path}: {key} is not re-derived");
        }
        want.insert(path);
    }
    let filed = crate::eval_support::frozen_docs(&format!("{family}-"));
    assert_eq!(
        filed.into_iter().collect::<std::collections::BTreeSet<_>>(),
        want,
        "{family} set drifted"
    );
}
