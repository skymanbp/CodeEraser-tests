//! The whitepaper's worked examples in Lua, on the row form
//! sonar_whitepaper_c.rs states. What Lua changes: `&&` / `||` / `!`
//! are `and` / `or` / `not`, the else-if chain is `elseif`, and
//! sumOfPrimes's labeled continue is Lua's own `goto` past the inner
//! loop (a `::OUT::` label closing the outer body, the jump's
//! fundamental +1 on p.8). The one substitution is toRegexp's ternary:
//! Lua has no conditional operator and its `a and b or c` idiom is
//! two operators (coc_lua.rs pins it at +2), so that line opens with
//! an `if` at nesting 0 — the ternary's +1 — and its `or` pays the
//! line's other point. No Lua form (the booklet's "no switch / goto"
//! register, R and Haskell alike): getWords (no switch, and the
//! example's point is the switch's one increment), myMethod and
//! addVersion (no try / catch: `pcall` is a call, register D8),
//! myMethod2 (a function value is a unit of its own, register D3, so
//! the page's host-plus-lambda total has no one unit to land on; the
//! split itself is pinned by coc_lua.rs's units row).

use crate::common;

const ROWS: &str = r#"
lua @@ coc=7 cc=4 @@ p.10 margin: for +1, for +2, if +3, the jump to the label +1 (`goto OUT` where Java continues OUT); p.5 margin: cyclomatic 4, a goto is no decision
local function sumOfPrimes(max)
   local total = 0
   for i = 1, max do
      for j = 2, i - 1 do
         if i % j == 0 then
            goto OUT
         end
      end
      total = total + i
      ::OUT::
   end
   return total
end
====
lua @@ coc=4 @@ p.8 margin: if +1, then +1 for each new sequence of like operators (and, or, and)
local function sequences(a, b, c, d, e, f)
   if a
          and b and c
          or d or e
          and f then
      return true
   end
   return false
end
====
lua @@ coc=3 @@ p.8 margin: if +1, and +1, and the `and` inside the negated parentheses starts a sequence of its own +1
local function negated(a, b, c)
   if a
          and
          not (b and c) then
      return true
   end
   return false
end
====
lua @@ coc=19 @@ p.17 margin total (Appendix C, JavaSymbol.java in SonarJava): the for-each is a generic for and the nullable Boolean a nil, the structure is the page's
local function overriddenSymbolFrom(classType)
  if classType:isUnknown() then
    return Symbols.unknownMethodSymbol
  end
  local unknownFound = false
  local symbols = classType:getSymbol():members():lookup(name)
  for _, overrideSymbol in ipairs(symbols) do
    if overrideSymbol:isKind(JavaSymbol.MTH)
        and not overrideSymbol:isStatic() then
      local methodJavaSymbol = overrideSymbol
      if canOverride(methodJavaSymbol) then
        local overriding = checkOverridingParameters(methodJavaSymbol,
            classType)
        if overriding == nil then
          if not unknownFound then
            unknownFound = true
          end
        elseif overriding then
          return methodJavaSymbol
        end
      end
    end
  end
  if unknownFound then
    return Symbols.unknownMethodSymbol
  end
  return nil
end
====
lua @@ coc=20 @@ p.19 margin total (Appendix C, WildcardPattern.java in SonarQube): the ternary is an if at nesting 0 (its +1 beside the or's +1), the else-if chain is elseif
local function toRegexp(antPattern, directorySeparator)
  local escapedDirectorySeparator = "\\" .. directorySeparator
  local sb = "^"
  local i = 0
  if startsWith(antPattern, "/") or
      startsWith(antPattern, "\\") then i = 1 end
  while i < #antPattern do
    local ch = antPattern:sub(i + 1, i + 1)
    if SPECIAL_CHARS:find(ch, 1, true) ~= nil then
      sb = sb .. "\\" .. ch
    elseif ch == "*" then
      if i + 1 < #antPattern
          and antPattern:sub(i + 2, i + 2) == "*" then
        if i + 2 < #antPattern
            and isSlash(antPattern:sub(i + 3, i + 3)) then
          sb = sb .. "(?:.*" .. escapedDirectorySeparator .. "|)"
          i = i + 2
        else
          sb = sb .. ".*"
          i = i + 1
        end
      else
        sb = sb .. "[^" .. escapedDirectorySeparator .. "]*?"
      end
    elseif ch == "?" then
      sb = sb .. "[^" .. escapedDirectorySeparator .. "]"
    elseif isSlash(ch) then
      sb = sb .. escapedDirectorySeparator
    else
      sb = sb .. ch
    end
    i = i + 1
  end
  return sb .. "$"
end
"#;

#[test]
fn whitepaper_worked_examples_in_lua() {
    common::assert_metric_table(ROWS);
}
