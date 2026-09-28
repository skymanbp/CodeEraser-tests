//! TypeScript's module-level `const` / `let` / `var` (plan v2.30 step
//! 5b-6; fourclass/declared.rs): the lexical declaration is the unit
//! and its word is read by the same climbs as a function's — the
//! `export` wrapper, the namespace chain under the module / script
//! split, and the ambient rule. One text table through the shared
//! runner (tests.rs), as the C family's and Java's are.

use super::super::tests::{check_units, run_table};

/// `ts <source> ⇒ <name:letters …>` per line, newlines spelled `\n`,
/// `#` lines commentary — measured over every unit the register keys.
const ALL: &str = r#"
# `export` names the binding; a plain one is private; `var` reads alike;
# a grouped declaration binds each name.
ts export const A = 1, B = 2;\nconst c = 3;\nvar v = 4;\nexport var w = 5; ⇒ A:ES B:ES c:- v:- w:ES
# A destructuring export binds each identifier the pattern names — the
# pair's value, the rest's target, the defaulted shorthand — never the
# key or the default. K26 (the L round's criterion) pinned such an
# export to zero symbols because no declaration node existed to hang
# one on; the lexical declaration is that node now, and the words of
# the functions inside such exports (tests.rs's guarded climb) do not
# move.
ts export const { p, q: r, ...rest } = z;\nexport const [s, , t = 1] = a;\nexport const { u = 2 } = z; ⇒ p:ES r:ES rest:ES s:ES t:ES u:ES
ts export const [a, b] = pair();\nexport const { x } = obj(); ⇒ a:ES b:ES x:ES
# The function extractor already names an arrow or an anonymous
# function expression after its declarator, so the declaration mints no
# second symbol; a named function expression keeps its own name and the
# binding is a unit of its own.
ts export let f = () => 1;\nexport const g = function h() {};\nexport const k = function () {}; ⇒ f/0:ES g:ES h/0:- k/0:ES
# The namespace chain: a module file's private namespace closes bit 1,
# `export namespace` opens it, a script's top-level namespace is global;
# a namespace-private binding is nothing.
ts export {};\nnamespace N { export const inN = 1; const priv = 2; } ⇒ inN:E priv:-
ts export namespace N { export const inN = 1; } ⇒ inN:ES
ts namespace N { export const inN = 1; } ⇒ inN:ES
# Ambient contexts read the spec: `declare const` at top level still
# needs `export`, the elements of `declare global` and of a string-named
# `declare module` are exported without it.
ts export declare const D: number;\ndeclare const E: number; ⇒ D:ES E:-
ts export {};\ndeclare global { const G: number; }\ndeclare module "m" { const inM: number; } ⇒ G:ES inM:ES
"#;

#[test]
fn typescript_lexical_words_follow_the_table() {
    run_table(ALL, check_units);
}
