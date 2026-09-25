//! The templates a Lua file writes into `package.path`, read the walk's
//! way (plan v2.30 step 4): literal text only — a string, or the string
//! operands of a `..` chain — assigned to a target spelled
//! `package.path`, relative, with one `?` and a `.lua` ending, a
//! Windows backslash read as a separator.

use super::{Template, read};
use crate::testutil::blocks;

/// Each block: `lua @@ templates @@ why` over a Lua text — the
/// templates `read` must find, in document order, each spelled
/// `dir/?suffix` and joined by ` | ` (`-` = none). The absolute
/// templates are fixture samples of what the reader refuses, no
/// machine's paths.
const CASES: &str = r#"
lua @@ frontend/?.lua | common/?.lua @@ a literal prepended to the running path
package.path = "frontend/?.lua;common/?.lua;" .. package.path
====
lua @@ a/?.lua | b/?/init.lua @@ parenthesised `..` operands
package.path = ("a/?.lua;" .. ("b/?/init.lua;")) .. package.path
====
lua @@ lib/?.lua @@ a literal appended, its `./` dropped
package.path = package.path .. ";./lib/?.lua"
====
lua @@ deps/?.lua @@ a long string
package.path = [[deps/?.lua]]
====
lua @@ ?.lua | ?/init.lua @@ the tree root's own templates; `;;` (the default path) is none
package.path = "?.lua;?/init.lua;;"
====
lua @@ - @@ only the spelling `package.path` is the target, the prefilter's own
package . path = "spaced/?.lua"
====
lua @@ x/?.lua | y/?.lua @@ each target takes the value in its own position
x, package.path = 1, "x/?.lua"
package.path, y = "y/?.lua", "z/?.lua"
====
lua @@ ?.lua | lib/?.lua @@ a Windows backslash is a separator
package.path = ".\\?.lua;lib\\?.lua"
====
lua @@ - @@ a computed value states no template
package.path = string.format("%s/?.lua;%s", dir, package.path)
====
lua @@ - @@ templates outside the tree (root, drive, variable, home) are refused
package.path = "/usr/share/lua/5.1/?.lua;C:/lua/?.lua;$LUA_DIR/?.lua;~lua/?.lua"
====
lua @@ - @@ two `?`, another suffix, no separator before the `?`, no `?` at all
package.path = "a/??.lua;b/?.luac;c?.lua;d/x.lua"
====
lua @@ - @@ cpath, a comment and a plain local assign no package.path
package.cpath = "c/?.so;" .. package.cpath
-- package.path = "commented/?.lua"
local p = "local/?.lua"
"#;

/// The templates a column spells.
fn expected(spelled: &str) -> Vec<Template> {
    spelled
        .split(" | ")
        .filter(|t| *t != "-")
        .map(|t| {
            let (dir, suffix) = t.split_once('?').expect("a template holds its `?`");
            Template {
                dir: dir.trim_end_matches('/').to_string(),
                suffix: suffix.to_string(),
            }
        })
        .collect()
}

#[test]
fn a_file_states_the_templates_it_assigns_literally() {
    for (_, [want, why], text) in blocks(CASES) {
        assert_eq!(read(text), expected(want), "{why}\n--- text ---\n{text}");
    }
}
