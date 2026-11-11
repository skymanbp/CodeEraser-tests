//! The unit table of the dead-file exam's classifiers, second half
//! (eval_deadfile_k1.rs reads it; layout in k1_cases.rs).

/// Lua, R, Java, C / C++.
pub const OTHERS: &str = r#"
==== lua | declares | declares | - | - | rule 1 inert
-- a comment
==== lua | runs-alone | runs-alone | - | function_call@2 | rule 1 active
-- a comment
print(1)
==== lua | runs-alone | runs-alone | - | variable_declaration@1 | rule 2 active: require is a call
local json = require("json")
==== lua | declares | declares | - | - | rule 3 inert
function f() end
local function g() end
function M.h() end
==== lua | runs-alone | runs-alone | - | function_call@2 | rule 3 active
function f() end
f()
==== lua | declares | declares | - | - | rule 5 inert
local M = {}
local a, b = 1, "s"
M.f = function() end
X = { name = "x", 1, [2] = true }
local n = -1
local p = a.b.c
==== lua | runs-alone | runs-alone | - | variable_declaration@1 | rule 5 active
local t = setmetatable({}, mt)
==== lua | declares | declares | - | - | rule 6 inert: return a name
local M = {}
return M
==== lua | declares | declares | - | - | rule 6 inert: return a table
return { name = "x" }
==== lua | runs-alone | runs-alone | - | return_statement@1 | rule 6 active
return require("y")
==== lua | declares | runs-alone | 2 | - | ruling 2 inert
do local x = 1 end
==== lua | runs-alone | runs-alone | 2 | function_call@1 | ruling 2 active
do f() end
==== lua | declares | runs-alone | 3 | - | ruling 3 inert
local a = 1;
==== lua | runs-alone | runs-alone | 3 | function_call@1 | ruling 3 active
f();
==== lua | declares | runs-alone | 4 | - | ruling 4 inert
#!/usr/bin/lua
local a = 1
==== lua | runs-alone | runs-alone | 4 | function_call@2 | ruling 4 active
#!/usr/bin/lua
main()
==== lua | declares | runs-alone | 5 | - | ruling 5 inert
global <const> *
global x
==== lua | runs-alone | runs-alone | - | variable_declaration@1 | ruling 5 active
global x = f()
==== lua | declares | runs-alone | 6 | - | ruling 6 inert
::top::
==== lua | runs-alone | runs-alone | 6 | goto_statement@2 | ruling 6 active: goto stays active
::top::
goto top
==== lua | declares | runs-alone | 8 | - | ruling 8 inert
local a = (1)
==== lua | runs-alone | runs-alone | 8 | variable_declaration@1 | ruling 8 active
local a = (f())
==== lua | runs-alone | runs-alone | 9 | variable_declaration@1 | ruling 9 active
local s = "a" .. "b"
==== lua | declares | declares | 13 | - | ruling 13 inert
local x
==== lua | runs-alone | runs-alone | - | variable_declaration@1 | ruling 13 active
local x = f()
==== lua | runs-alone | runs-alone | 15 | variable_declaration@1 | ruling 15 active
local a = t[1]
==== r | declares | declares | - | - | rule 1 inert
# a comment
==== r | runs-alone | runs-alone | - | call@2 | rule 1 active
# a comment
print(1)
==== r | runs-alone | runs-alone | - | call@1 | rule 2 active: library is a call
library(stats)
==== r | declares | declares | - | - | rule 4 inert
"a bare string"
==== r | runs-alone | runs-alone | - | call@1 | rule 4 active
paste("a")
==== r | declares | declares | - | - | rule 5 inert
f <- function(x) x
x = 1
y <<- "s"
z <- NULL
w <- -1
t <- TRUE
==== r | runs-alone | runs-alone | - | binary_operator@1 | rule 5 active: c() is a call
x <- c(1, 2)
==== r | runs-alone | runs-alone | - | call@1 | control: source
source("a.R")
==== r | declares | runs-alone | 8 | - | ruling 8 inert
x <- (1)
==== r | runs-alone | runs-alone | 8 | binary_operator@1 | ruling 8 active
x <- (f())
==== r | runs-alone | runs-alone | 9 | binary_operator@1 | ruling 9 active
x <- 3:20
==== r | declares | runs-alone | 10 | - | ruling 10 inert
x <- a$b
y <- pkg::name
z <- a@b
w <- pkg:::n
v <- a$b$c
==== r | runs-alone | runs-alone | - | binary_operator@1 | ruling 10 active: rooted at a call
x <- f()$b
==== r | declares | declares | 11 | - | ruling 11 inert
x <- NA
y <- Inf
z <- NaN
w <- NA_integer_
==== r | runs-alone | runs-alone | - | binary_operator@1 | ruling 11 active neighbour
x <- is.na(y)
==== r | runs-alone | runs-alone | 15 | binary_operator@1 | ruling 15 active: subset
x <- df[1]
==== r | runs-alone | runs-alone | 15 | binary_operator@1 | ruling 15 active: subset2
x <- l[[1]]
==== r | runs-alone | runs-alone | 15 | binary_operator@1 | ruling 15 active: if as a value
x <- if (a) 1 else 2
==== r | runs-alone | runs-alone | 15 | identifier@1 | ruling 15 active: bare name
p
==== java | declares | declares | - | - | rule 1 inert
// a comment
/* another */
==== java | declares | declares | - | - | rule 2 inert
package a;
import b.C;
==== java | declares | declares | - | - | rule 2 inert: a module header
module m {}
==== java | declares | declares | - | - | rule 3 inert
@Deprecated
public class D {}
interface I {}
enum E { A }
record R(int a) {}
@interface An {}
==== cpp | declares | declares | - | - | rule 1 inert
/* a comment */
// another
==== cpp | declares | declares | - | - | rule 2 inert
#include <stdio.h>
#include "a.h"
==== cpp | runs-alone | runs-alone | - | declaration@2 | rule 2 active
#include <stdio.h>
int x = f();
==== cpp | declares | declares | - | - | rule 3 inert
#define X 1
#define F(a) (a)
int f(void);
int g(void) { return 0; }
struct S { int a; };
typedef int T;
enum E { A };
namespace n { int a = f(); }
template<class T> T id(T t);
class K {};
using U = int;
==== cpp | declares | declares | - | - | rule 5 inert
int x = 1;
static const char *s[] = {"a", "b"};
int y = -1;
==== cpp | runs-alone | runs-alone | - | declaration@1 | rule 5 active
int x = f();
==== cpp | declares | runs-alone | 1 | - | ruling 1 inert: an include guard
#ifndef H
#define H
int f(void);
#endif
==== cpp | runs-alone | runs-alone | 1 | declaration@3 | ruling 1 active: every branch judged
#ifndef H
#define H
int x = f();
#else
int y = 1;
#endif
==== cpp | runs-alone | runs-alone | 1 | declaration@4 | ruling 1 active: the elif branch
#if A
int a = 1;
#elif B
int b = g();
#endif
==== cpp | declares | runs-alone | 1 | - | ruling 1 inert: a linkage block
extern "C" { int g(void); }
==== cpp | declares | runs-alone | 1 | - | ruling 1 inert: another directive
#pragma once
==== c | declares | runs-alone | 1 | - | ruling 1 inert in C
#ifdef X
int f(void);
#endif
==== cpp | declares | runs-alone | 8 | - | ruling 8 inert
int x = (1);
==== cpp | runs-alone | runs-alone | 8 | declaration@1 | ruling 8 active
int x = (f());
==== cpp | runs-alone | runs-alone | 9 | declaration@1 | ruling 9 active
int x = 2 + 3;
==== cpp | declares | declares | 13 | - | ruling 13 inert
int y;
extern int z;
==== cpp | runs-alone | runs-alone | - | declaration@1 | ruling 13 active
int y = g();
==== cpp | runs-alone | runs-alone | 15 | declaration@1 | ruling 15 active
int a = b[0];
==== cpp | declares | runs-alone | 1,16 | - | ruling 16 inert: a fragment is no witness
#if A
#define lua_readline(L,b,p)	\
	((void)L, fputs(p, stdout),  /* show prompt */ \
	fgets(b, 1, stdin) != NULL)
#endif
==== cpp | runs-alone | runs-alone | 1,16 | declaration@6 | ruling 16 active: a statement beside the fragment
#if A
#define lua_readline(L,b,p)	\
	((void)L, fputs(p, stdout),  /* show prompt */ \
	fgets(b, 1, stdin) != NULL)
#endif
int x = f();
"#;
