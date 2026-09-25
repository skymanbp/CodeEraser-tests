//! Lua legs (plan v2.30 step 4; design booklet §7 row Lua): `local`
//! against everything else, and the enclosing function, through the
//! extractor's own root. Every shape here was probed on tree-sitter-lua
//! 0.5.0 before the reading was written (the scripts/tsprobe
//! transcripts). Two text tables through the shared runner (tests.rs
//! run_tables), as the C family's and Java's are.

use super::super::tests::run_tables;

/// `lua <source> ⇒ <name:letters …>` per line, newlines spelled `\n`,
/// `#` lines commentary — measured over the functions the extractor
/// sees.
const FNS: &str = r#"
# `local` hides a function from every other file, whether it declares
# the function or binds a function value; a global, a table member and
# a table field are reachable from outside.
lua local function a() end\nlocal b = function() end\nfunction c() end\nfunction M.d() end\nfunction M:e() end\nf = function() end\nlocal t = { g = function() end } ⇒ a:- b:- c:ES M.d:ES M:e:ES f:ES g:ES
# A function inside another closes the scope bit whatever it is; an
# anonymous function value names nothing to export.
lua function outer()\n  local function inner() end\n  function global() end\n  return function() end\nend ⇒ outer:ES inner:- global:E (anonymous):-
"#;

/// The same line shape over every unit the register keys, spelled as
/// stored (`name/arity`).
const ALL: &str = r#"
lua local function a(x, ...) end\nfunction M:b(y) end ⇒ a/2:- M:b/1:ES
"#;

#[test]
fn lua_words_follow_the_tables() {
    run_tables(FNS, ALL);
}
