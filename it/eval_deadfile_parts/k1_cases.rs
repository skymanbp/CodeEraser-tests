//! The unit table of the dead-file exam's classifiers (eval_deadfile_k1.rs
//! reads it): one case per block, `lang | k1 class | k1-strict class |
//! rulings met | k1 witness | label`, then the text.

/// Python, TypeScript, TSX.
pub const SCRIPTED: &str = r#"
==== python | declares | declares | - | - | rule 1 inert
# a comment
==== python | runs-alone | runs-alone | - | expression_statement@2 | rule 1 active
# a comment
print(1)
==== python | declares | declares | - | - | rule 2 inert
import os
from a import b
==== python | runs-alone | runs-alone | - | expression_statement@2 | rule 2 active
import os
os.getcwd()
==== python | declares | declares | - | - | rule 3 inert
@dec
def f():
    pass
class C:
    pass
==== python | runs-alone | runs-alone | - | expression_statement@3 | rule 3 active
def f():
    pass
f()
==== python | declares | declares | - | - | rule 4 inert: a docstring
"""The module's docstring."""
==== python | runs-alone | runs-alone | - | expression_statement@1 | rule 4 active: a bare f-string
f"{x}"
==== python | declares | declares | - | - | rule 5 inert
X = 1
__all__ = ["a", "b"]
Y = -1
Z = lambda: 0
W = a.b.c
V = {"k": [1, None, True], "t": (1, 2)}
A, B = 1, "s"
==== python | runs-alone | runs-alone | - | expression_statement@1 | rule 5 active
X = f()
==== python | runs-alone | runs-alone | - | if_statement@1 | control: the main guard
if __name__ == "__main__":
    main()
==== python | declares | runs-alone | 8 | - | ruling 8 inert
X = (1)
==== python | runs-alone | runs-alone | 8 | expression_statement@1 | ruling 8 active
X = (f())
==== python | runs-alone | runs-alone | 9 | expression_statement@1 | ruling 9 active
X = 2 + 3
==== python | declares | declares | 13 | - | ruling 13 inert
x: int
==== python | runs-alone | runs-alone | - | expression_statement@1 | ruling 13 active
x: int = f()
==== python | runs-alone | runs-alone | 14 | expression_statement@2 | ruling 14 active
X = []
X += [1]
==== python | runs-alone | runs-alone | 15 | expression_statement@1 | ruling 15 active: comprehension
X = [i for i in y]
==== python | runs-alone | runs-alone | 15 | expression_statement@1 | ruling 15 active: subscript
X = y[0]
==== python | runs-alone | runs-alone | 15 | expression_statement@1 | ruling 15 active: bare name
x
==== typescript | declares | declares | - | - | rule 1 inert
// a comment
/* another */
==== typescript | runs-alone | runs-alone | - | expression_statement@2 | rule 1 active
// a comment
f();
==== typescript | declares | declares | - | - | rule 2 inert
import a from "a";
import b = require("b");
==== typescript | runs-alone | runs-alone | - | expression_statement@2 | rule 2 active
import a from "a";
a();
==== typescript | declares | declares | - | - | rule 3 inert
function f() {}
class C {}
interface I {}
type T = number;
enum E { A }
declare module "m" {}
namespace N {}
==== typescript | runs-alone | runs-alone | - | expression_statement@2 | rule 3 active
class C {}
new C();
==== typescript | declares | declares | - | - | rule 4 inert: directives
"use strict";
"use client";
==== typescript | runs-alone | runs-alone | - | expression_statement@1 | rule 4 active: a template with a substitution
`a${b}`;
==== typescript | declares | declares | - | - | rule 5 inert
const a = 1, b = "x", c = null, d = () => 0;
const o = { k: [1, -2], f() {} };
module.exports = { a };
const m = a.b;
==== typescript | runs-alone | runs-alone | - | lexical_declaration@1 | rule 5 active
const a = f();
==== typescript | declares | declares | - | - | rule 6 inert
export default { a: 1 };
export { a } from "./b";
export * from "./c";
export const d = 1;
==== typescript | runs-alone | runs-alone | - | export_statement@1 | rule 6 active
export default defineConfig({});
==== typescript | declares | runs-alone | 2 | - | ruling 2 inert: a bare block
{ const a = 1; }
==== typescript | runs-alone | runs-alone | 2 | expression_statement@1 | ruling 2 active
{ f(); }
==== typescript | declares | runs-alone | 3 | - | ruling 3 inert
const a = 1;;
==== typescript | runs-alone | runs-alone | 3 | expression_statement@2 | ruling 3 active
;
f();
==== typescript | declares | runs-alone | 4 | - | ruling 4 inert
#!/usr/bin/env node
const a = 1;
==== typescript | runs-alone | runs-alone | 4 | expression_statement@2 | ruling 4 active
#!/usr/bin/env node
main();
==== typescript | declares | declares | - | - | ruling 7 inert neighbour: a string
const r = "a+";
==== typescript | runs-alone | runs-alone | 7 | lexical_declaration@1 | ruling 7 active
const r = /a+/;
==== typescript | declares | runs-alone | 8 | - | ruling 8 inert
const a = 1 as const;
const b = x!;
const c = (2);
const t = process.env.GITHUB_TOKEN!;
==== typescript | runs-alone | runs-alone | 8 | lexical_declaration@1 | ruling 8 active
const a = f() as T;
==== typescript | runs-alone | runs-alone | 9 | lexical_declaration@1 | ruling 9 active
const a = 2 + 3;
==== typescript | declares | declares | 12 | - | ruling 12 inert
const a = `x`;
==== typescript | runs-alone | runs-alone | - | lexical_declaration@1 | ruling 12 active
const a = `x${y}`;
==== typescript | declares | declares | 13 | - | ruling 13 inert
let a;
==== typescript | runs-alone | runs-alone | - | lexical_declaration@1 | ruling 13 active
let a = f();
==== typescript | runs-alone | runs-alone | 15 | lexical_declaration@1 | ruling 15 active
const a = b[0];
const c = { ...d };
const e = x ? 1 : 2;
==== typescript | runs-alone | runs-alone | 15 | expression_statement@1 | ruling 15 active: bare name
x;
==== tsx | declares | declares | - | - | tsx rule 6 inert
export const a = 1;
==== tsx | runs-alone | runs-alone | 15 | lexical_declaration@1 | tsx ruling 15 active: JSX
const a = <div />;
"#;
