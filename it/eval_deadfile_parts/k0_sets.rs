//! K0's DECLARES sets, ported verbatim from the lane's first classifier
//! `universe_counts.py` (sha256 549337f8b658574a1181ee79836b00a1293dbc3f4a5279ca5a088822dd5dd0f4):
//! one space-separated line per language; `tsx` reads `typescript`'s,
//! `cpp` reads `c`'s plus its own six. A language without a line has no
//! set (never `declares`).

const SETS: &str = "python function_definition class_definition decorated_definition import_statement import_from_statement future_import_statement comment
typescript function_declaration class_declaration interface_declaration type_alias_declaration enum_declaration import_statement export_statement lexical_declaration abstract_class_declaration module comment ambient_declaration
rust function_item struct_item enum_item impl_item trait_item use_declaration mod_item const_item static_item type_item macro_definition attribute_item inner_attribute_item line_comment block_comment
go package_clause import_declaration function_declaration method_declaration type_declaration const_declaration var_declaration comment
java package_declaration import_declaration class_declaration interface_declaration enum_declaration record_declaration annotation_type_declaration module_declaration line_comment block_comment
c function_definition declaration type_definition struct_specifier enum_specifier preproc_include preproc_def preproc_function_def preproc_ifdef preproc_if preproc_call comment
cpp+ namespace_definition class_specifier template_declaration using_declaration alias_declaration linkage_specification
lua function_declaration variable_declaration comment return_statement
r comment
haskell header imports declarations comment pragma haddock";

fn line_of(name: &str) -> Option<Vec<&'static str>> {
    SETS.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .find(|w| w[0] == name)
        .map(|w| w[1..].to_vec())
}

/// The set of a language, owned for the call.
pub fn set(lang: &str) -> Option<Vec<&'static str>> {
    match lang {
        "tsx" => line_of("typescript"),
        "cpp" => line_of("c").map(|mut s| {
            s.extend(line_of("cpp+").unwrap_or_default());
            s
        }),
        other => line_of(other),
    }
}
