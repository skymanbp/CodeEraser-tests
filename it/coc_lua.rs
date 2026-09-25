//! Plan v2.30 step 4 Lua metric battery: the launch-language rows of
//! metrics.rs re-spelled in Lua where the construct exists, plus one
//! row per stance the Lua table records (scan/spec_lua.rs, the booklet's
//! register D3 / D5 / D8) — each why cites the whitepaper page or the
//! register entry — and a `units=` block pinning what the table makes a
//! unit, how a binding names it (scan/binding.rs) and what it reads as
//! each unit's parameters. Recursion is the core's and is asserted
//! through it (coc_recursion.rs); these rows read the pre-cycle value.

use crate::common;

const ROWS: &str = r#"
lua @@ cc=5 coc=7 nesting=3 lines=10 @@ metrics.rs's Python row in Lua: cc 1 + if + and + for + if; coc if +1, and +1, for +2, if +3; nesting if > for > if
local function f(a, b)
  if a > 0 and b > 0 then
    for i = 1, 10 do
      if i > 5 then
        return i
      end
    end
  end
  return 0
end
====
lua @@ cc=5 coc=5 nesting=1 lines=9 @@ metrics.rs's TypeScript row in Lua: `elseif` is a flat hybrid (p.7) that CC counts as a branch, and the trailing `else` pays +1 without nesting
local function g(a, b)
  if a > 0 and b > 0 then
    return a + b
  elseif a > 0 or b > 0 then
    return 1
  else
    return 0
  end
end
====
lua @@ cc=5 coc=4 nesting=1 lines=8 @@ a numeric for, a generic for, a while and a repeat-until are loops like any other, none nesting another
local function l(n, t)
  local s = 0
  for i = 1, n do s = s + i end
  for _, v in pairs(t) do s = s + v end
  while s > 0 do s = s - 1 end
  repeat s = s + 1 until s >= 3
  return s
end
====
lua @@ cc=5 coc=10 nesting=3 lines=10 @@ a goto is a jump to a label, a fundamental +1 without nesting (p.8; register D5), while a bare `break` names no label and pays nothing
local function j(m)
  for _, row in ipairs(m) do
    for _, v in ipairs(row) do
      if v < 0 then goto done end
      if v == 0 then break end
    end
  end
  ::done::
  return 0
end
====
lua @@ cc=3 coc=2 nesting=0 lines=3 @@ Lua's `a and b or c` idiom is two operators, not a ternary: each change of operator in the run pays +1 (p.5), and nothing nests
local function t(a, b, c)
  return a and b or c
end
====
lua @@ cc=1 coc=0 nesting=0 lines=4 @@ `pcall` and `error` are calls, invisible to the syntax: the protected call branches nothing the table can see (register D8)
local function p(f)
  local ok, err = pcall(f)
  error(err)
end
====
lua @@ units=a/2, b/1, M.c/2, M:d/1, e/0, f/1, (anonymous)/0, g/0, h/2, (anonymous)/1 @@ what the Lua table makes a unit (booklet §4): a declaration by its own name, a function value by the statement binding it — a local, a table field (a string key by its content), the variable at its position in a multiple assignment — and an unbound value `(anonymous)` (register D3: an anonymous function is a unit of its own). A vararg counts as one parameter; the `self` a colon method takes is no formal parameter
local function a(x, ...) end
local b = function(y) end
function M.c(p, q) end
function M:d(r) end
local t = { e = function() end, ["f"] = function(s) end, [k] = function() end }
g, h = function() end, function(u, v) end
return function(w) end
"#;

#[test]
fn lua_metrics_follow_the_table() {
    common::assert_metric_table(ROWS);
}
