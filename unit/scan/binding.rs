use super::*;
use crate::testutil::blocks;

/// Each block: a header `ext @@ reading | reading … @@ why` over a
/// source, one reading per function node in document order (`reading`
/// below). One literal rather than rows of typed tuples: rows of one
/// shape repeat every dozen tokens, and the clone gate reads such a
/// table as clones of itself.
const CASES: &str = r#"
lua @@ f local in chunk | g local in chunk | y local in chunk | - of M/p | - of M/q | M.h in chunk of M/h | a in chunk | b in chunk @@ Lua names a function value by the statement around it: a `local function` and a `local` assignment bind a local (the value at position i takes the variable at position i), a plain assignment a global or a table member, while `function M.p` and `function M:q` name themselves by their own member-shaped name
local function f() end
local g = function() end
local x, y = 1, function() end
function M.p() end
function M:q() end
M.h = function() end
a, b = function() end, function() end
====
lua @@ k in table_constructor | s in table_constructor | - | - | - @@ a table field names its value by its key, and a string key in brackets by its content; the grammar gives the computed key `[k]` the same `name: identifier` as the plain key `k`, so inside brackets only a string names anything, and a positional field names nothing
local t = { k = function() end, ["s"] = function() end, [k] = function() end, [1 + 1] = function() end, function() end }
====
lua @@ outer local in chunk | inner local in block @@ a local declared inside a function body lives in that body's block, not in the file
local function outer()
  local function inner() end
end
====
r @@ f in program | g in program | s in program | x$m in program of x/m | k in program | - | - | - @@ R names a function value by its assignment: `<-` by its left side, `->` by its right (only through parentheses — without them `function(y) y -> z` is a body that assigns y), a string target by its content, `x$m` as a member of x, a chain by its innermost target; an argument value and a call target (`attr(x, "a") <-`) name nothing
f <- function(x) x
(function(x) x) -> g
"s" <- function() 1
x$m <- function() 1
h <- k <- function() 1
function(y) y -> z
lapply(xs, function(v) v)
attr(x, "a") <- function() 1
"#;

/// One function node as the table spells it: the name its binding
/// gives it (`-` for none), `local` for a Lua local, the kind of the
/// node the name lives in, and `of object/member` for a member-shaped
/// name.
fn reading(node: Node<'_>, src: &[u8]) -> String {
    let mut out = of(node, src).map_or("-".to_string(), |b| {
        let local = if b.is_local() { " local" } else { "" };
        let name = spelled(b.name, src).unwrap_or_default();
        format!("{name}{local} in {}", b.scope.kind())
    });
    if let Some((object, member)) = member_of(node, src) {
        out.push_str(&format!(" of {object}/{member}"));
    }
    out
}

#[test]
fn a_statement_names_the_function_value_it_binds() {
    for (lang, [want, why], body) in blocks(CASES) {
        let tree = ast::parse_lang(body, lang).expect("a grammar");
        let mut seen = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if matches!(node.kind(), "function_declaration" | "function_definition") {
                seen.push(reading(node, body.as_bytes()));
            }
            stack.extend(ast::children(node).into_iter().rev());
        }
        assert_eq!(seen.join(" | "), want, "{why}\n--- source ---\n{body}");
    }
}
