//! Lua lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the Lua grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
lua @@ elseif @@ rule 2 and ruling 11: an elseif chain nests in the else position, an empty branch is an empty block; a global write is no local
local function f(x)
  local y
  if x > 0 then
  elseif x < 0 then
    y = 1
  else
    y = 2
    g = y
  end
  return y
end
----
s 0 -1 1 0 0
s 1 -1 2 1 0
s 2 1 0 16 0
s 3 1 2 1 0
s 4 3 0 0 0
s 5 4 1 0 0
s 6 3 0 0 0
s 7 6 1 0 0
s 8 6 1 0 0
s 9 -1 9 0 0
v 0 -1 1 x
v 1 0 0 y
x 1 0 0
x 3 0 0
x 5 1 1
x 7 1 1
x 8 1 0
x 9 1 0
====
lua @@ loops-goto @@ rules 3, 8: numeric and generic for declare their names; a goto reaches the label of its own loop body
local function f(t)
  local s = 0
  for i = 1, #t do
    if t[i] == nil then goto continue end
    s = s + t[i]
    ::continue::
  end
  for k, v in pairs(t) do
    s = s + v
  end
  return s
end
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 2 0 0
s 4 3 0 0 0
s 5 4 13 0 7
s 6 2 1 0 0
s 7 2 14 0 0
s 8 -1 3 0 0
s 9 8 0 0 0
s 10 9 1 0 0
s 11 -1 9 0 0
v 0 -1 1 t
v 1 0 0 s
v 2 1 0 i
v 3 8 0 k
v 4 8 0 v
x 0 1 1
x 1 0 0
x 1 2 1
x 3 0 0
x 3 2 0
x 6 1 0
x 6 0 0
x 6 2 0
x 6 1 1
x 8 0 0
x 8 3 1
x 8 4 1
x 10 1 0
x 10 4 0
x 10 1 1
x 11 1 0
====
lua @@ repeat @@ rule 5: `until` reads the body's locals; `while true` and `until false` are infinite
local function f(n)
  repeat
    local d = n % 10
    n = n // 10
  until d == 0
  while true do
    if n > 5 then break end
    n = n + 1
  end
  repeat
    n = n - 1
  until false
end
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 1 1 0 0
s 4 -1 3 2 0
s 5 4 0 0 0
s 6 5 2 0 0
s 7 6 0 0 0
s 8 7 11 0 4
s 9 5 1 0 0
s 10 -1 3 2 0
s 11 10 0 0 0
s 12 11 1 0 0
v 0 -1 1 n
v 1 2 0 d
x 0 1 0
x 2 0 0
x 2 1 1
x 3 0 0
x 3 0 1
x 6 0 0
x 9 0 0
x 9 0 1
x 12 0 0
x 12 0 1
====
lua @@ error-load @@ rules 3, 7, 9: error never returns, load is dynamic, a field write and a method call read their base
function M.f(self, x)
  local t = {}
  t.x = x
  t:push(1)
  if not x then error("no x") end
  local chunk = load("return 1")
  return chunk
end
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 2 0 0
s 4 3 0 0 0
s 5 4 15 0 0
s 6 -1 1 8 0
s 7 -1 9 0 0
v 0 -1 1 self
v 1 -1 1 x
v 2 0 0 t
v 3 6 0 chunk
x 0 2 1
x 1 1 0
x 1 2 0
x 2 2 0
x 3 1 0
x 6 3 1
x 7 3 0
====
lua @@ closures @@ rules 8, 10: `local a, b = …` binds each name; a function value captures the host names it mentions
local function f(a, b)
  local x, y = a, b
  local ok = a and (function() return y end)()
  x = ok or x
  return x
end
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 0 0 x
v 3 0 2 y
v 4 1 0 ok
x 0 0 0
x 0 1 0
x 0 2 1
x 0 3 1
x 1 0 0
x 1 4 1
x 2 4 0
x 2 2 0
x 2 2 1
x 3 2 0
====
lua @@ do-block @@ rules 1, 8: `do … end` is a block whose locals shadow; an attribute is no name
local function f()
  local x = 1
  do
    local x = 2
    print(x)
  end
  local y <const> = x
  return y
end
----
s 0 -1 1 0 0
s 1 -1 0 0 0
s 2 1 1 0 0
s 3 1 1 0 0
s 4 -1 1 0 0
s 5 -1 9 0 0
v 0 0 0 x
v 1 2 0 x
v 2 4 0 y
x 0 0 1
x 2 1 1
x 3 1 0
x 4 0 0
x 4 2 1
x 5 2 0
"#;

#[test]
fn lua_lowers_as_the_table_says() {
    run_table(TABLE);
}
