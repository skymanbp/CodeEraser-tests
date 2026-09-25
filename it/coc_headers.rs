//! The header-flat rule of Cognitive Complexity (user ruling
//! 2026-09-24, "conditions never count"; booklet language-expansion.md
//! §14 item 16, register D31): a structure's header — its condition, a
//! loop's clause, a switch's value, a catch's parameter — scores at the
//! structure's own level, and only the body its LangSpec coc_nesting_kinds
//! entry names raises nesting. One block per language and header kind, each
//! why stating the arithmetic and the score before the ruling; the Java
//! rows add PMD 7.27.0's reading, which agrees for an if's own condition
//! and nests every other header. The rows whose body is an unnamed
//! child (a Python except's block, a Go switch's cases, a Haskell case's
//! alternatives) pin that the body still nests.

use crate::common;

const ROWS: &str = r#"
py @@ coc=2 nesting=1 @@ the ternary in the if's condition pays +1 at the if's level (before the ruling +2, coc 3)
def f(a, b, c):
    if (b if a else c):
        return 1
    return 0
====
py @@ coc=3 nesting=1 @@ an elif's condition sits at the chain's level, where the if's does: if +1, elif +1, its ternary +1 (before +2, coc 4)
def g(a, b, c):
    if a:
        return 1
    elif (b if c else a):
        return 2
    return 0
====
py @@ coc=2 nesting=1 @@ a for's iterable is its header: the ternary pays +1 (before +2)
def k(a, xs, ys):
    for x in (xs if a else ys):
        print(x)
====
py @@ coc=3 nesting=2 @@ an except's block is an unnamed child, named by its kind in its entry: the if inside it still nests, +2
def e(a):
    try:
        return 1
    except ValueError:
        if a:
            return 2
    return 0
====
ts @@ coc=2 nesting=1 @@ the ternary in the if's condition pays +1 at the if's level (before +2)
function f(a: boolean, b: boolean, c: boolean) {
  if (a ? b : c) {
    return 1;
  }
  return 0;
}
====
ts @@ coc=2 nesting=1 @@ a switch's value is its header: the ternary pays +1 (before +2)
function s(a: boolean, x: number, y: number) {
  switch (a ? x : y) {
    case 1:
      return 1;
    default:
      return 0;
  }
}
====
ts @@ coc=3 nesting=2 @@ a catch's parameter is its header and its body still nests: the if inside pays +2
function t(a: boolean) {
  try {
    return 1;
  } catch (e) {
    if (a) {
      return 2;
    }
  }
  return 0;
}
====
rs @@ coc=3 nesting=1 @@ a match in the if's condition pays +1 at the if's level, the else +1 (before +2, coc 4)
fn f(x: i32) -> i32 {
    if match x { 0 => true, _ => false } {
        1
    } else {
        0
    }
}
====
rs @@ coc=3 nesting=1 @@ an if expression in a while's condition pays +1 at the while's level, its else +1 (before +2, coc 4)
fn g(v: &mut Vec<i32>, a: bool) {
    while if a { v.len() > 1 } else { v.len() > 2 } {
        v.pop();
    }
}
====
rs @@ coc=4 nesting=2 @@ a match's arms are its body: the if in an arm pays +2, its else +1
fn m(x: Option<i32>) -> i32 {
    match x {
        Some(v) => if v > 0 { 1 } else { 2 },
        None => 0,
    }
}
====
go @@ coc=3 nesting=2 @@ a func literal in the if's condition raises nesting from the if's level: the if inside it pays +2 (before +3, coc 4)
package p

func f(a bool) int {
	if func() bool {
		if a {
			return true
		}
		return false
	}() {
		return 1
	}
	return 0
}
====
go @@ coc=3 nesting=2 @@ a switch's cases are unnamed children, named by their kinds in its entry: the if in a case still nests, +2
package p

func s(x int) int {
	switch x {
	case 1:
		if x > 0 {
			return 1
		}
	}
	return 0
}
====
go @@ coc=3 nesting=2 @@ a type switch's cases likewise
package p

func t(v interface{}) int {
	switch v.(type) {
	case int:
		if v != nil {
			return 1
		}
	}
	return 0
}
====
java @@ coc=2 nesting=1 @@ the ternary in the if's condition pays +1 at the if's level (before +2); PMD reads 2 too
int f(boolean a, boolean b, boolean c) {
    if (a ? b : c) { return 1; }
    return 0;
}
====
java @@ coc=8 nesting=1 @@ four loops (+1 each) with a ternary in each header (+1 each: the while's condition, the for's init, the for-each's value, the do-while's trailing condition); PMD reads 12, nesting every loop's header
int l(boolean a, int n, int[] v, int[] w) {
    while ((a ? n : 0) > 0) { n--; }
    for (int i = a ? 0 : 1; i < 3; i++) { n++; }
    for (int x : a ? v : w) { n += x; }
    do { n--; } while ((a ? n : 0) > 0);
    return n;
}
====
java @@ coc=2 nesting=1 @@ a switch's value is its header: the ternary pays +1; PMD reads 3
int s(boolean a, int x, int y) {
    switch (a ? x : y) {
        case 1: return 1;
        default: return 0;
    }
}
====
java @@ coc=3 nesting=1 @@ an else-if's condition sits at the chain's level: if +1, else-if +1, its ternary +1; PMD reads 4, nesting the else-if's condition
int e(boolean a, boolean b, boolean c) {
    if (a) { return 1; } else if (b ? c : a) { return 2; }
    return 0;
}
====
java @@ coc=3 nesting=2 @@ a lambda in the if's condition raises nesting from the if's level: its ternary pays +2 (before +3); PMD reads 3
int m(java.util.List<Integer> v) {
    if (v.stream().anyMatch(x -> x > 0 ? true : false)) { return 1; }
    return 0;
}
====
c @@ coc=6 nesting=1 @@ an if, a for and a switch, each with a ternary in its header: +1 each structure, +1 each ternary (before +2 each, coc 9)
int f(int a, int b, int n) {
    if (a ? b : n) {
        n++;
    }
    for (int i = a ? 0 : 1; i < 3; i++) {
        n++;
    }
    switch (a ? b : n) {
    case 1:
        return 1;
    }
    return n;
}
====
c @@ coc=3 nesting=2 @@ a switch's body still nests: the if inside it pays +2
int s(int x) {
    switch (x) {
    case 1:
        if (x > 0) {
            return 1;
        }
    }
    return 0;
}
====
cpp @@ coc=3 nesting=2 @@ a lambda in the if's condition raises nesting from the if's level: its ternary pays +2 (before +3, coc 4)
int f(std::vector<int> v) {
    if (std::any_of(v.begin(), v.end(), [](int x) { return x > 0 ? true : false; })) {
        return 1;
    }
    return 0;
}
====
cpp @@ coc=2 nesting=1 @@ a range-for's range is its header: the ternary pays +1 (before +2)
int g(bool a, std::vector<int> v, std::vector<int> w) {
    int s = 0;
    for (int x : a ? v : w) {
        s += x;
    }
    return s;
}
====
cpp @@ coc=3 nesting=2 @@ a catch's parameter is its header and its body still nests: the if inside pays +2
int t(bool a) {
    try {
        return 1;
    } catch (...) {
        if (a) {
            return 2;
        }
    }
    return 0;
}
====
hs @@ coc=2 nesting=1 @@ a case's scrutinee is its header: the if in it pays +1 (before +2, coc 3)
f :: Bool -> Int -> Int
f a x = case (if a then x else 0) of
  0 -> 1
  _ -> 2
====
hs @@ coc=3 nesting=2 @@ a case's alternatives are its body: the if in one pays +2
g :: Int -> Int
g x = case x of
  0 -> if x > 0 then 1 else 2
  _ -> 3
"#;

#[test]
fn only_a_structures_body_raises_nesting() {
    common::assert_metric_table(ROWS);
}
