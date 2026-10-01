//! The emitter's battery, mounted beside its module: what one
//! classified node states — its class bits, its nearest emitted
//! ancestor, the DIRECT and IN_ALT relation bits, the header / body
//! position and an if's alternative class — on the shapes the core's
//! rules turn on (CE.Scan.Complexity): an else-if under a flat
//! clause, an else-if in the if's own `alternative` field, a plain
//! else there, Python's elif chain, a let chain's operand count, a
//! labelled jump beside a plain one, a boolean chain's operator ids
//! in source order with the operator nodes inside it, a nested unit
//! left out with its structure, a ternary nesting whole, and an
//! opaque `#if` condition never reaching the stream. The whitepaper
//! register (contracts/fixtures/scan/whitepaper.ndjson) pins the
//! emitter on the page's examples; this table pins the bits by name.
//!
//! One literal rather than typed rows (the coc_c.rs lesson: rows of
//! one tuple shape read as clones of themselves). A block is
//! `ext @@ events @@ why` over a source; `events` spells the units in
//! extraction order joined by ` ; `, a unit's events joined by ` | `,
//! an event as `seq parent pos flags aux[ ops=…]` — pos `h` for the
//! header and `b` for the body, flags as letters in bit order
//! (N nesting, F flat, O nest-only, J labelled jump, I if kind,
//! C chain, K cc kind, P cc operator, L logic root, D direct, A in
//! alternative) followed by the alternative class digit when it is
//! not 0.

use super::*;
use crate::scan::{ast, functions, lang::Lang, spec};
use std::path::Path;

const LETTERS: [(u16, char); 11] = [
    (NESTING, 'N'),
    (FLAT, 'F'),
    (NEST_ONLY, 'O'),
    (LABELLED_JUMP, 'J'),
    (IF_KIND, 'I'),
    (CHAIN, 'C'),
    (CC_KIND, 'K'),
    (CC_OP, 'P'),
    (LOGIC_ROOT, 'L'),
    (DIRECT, 'D'),
    (IN_ALT, 'A'),
];

fn show(e: &Event) -> String {
    let letters: String = LETTERS
        .iter()
        .filter(|(bit, _)| e.flags & bit != 0)
        .map(|(_, c)| *c)
        .collect();
    let alt = match (e.flags >> ALT_SHIFT) & 3 {
        0 => String::new(),
        n => n.to_string(),
    };
    let ops = match e.ops.as_slice() {
        [] => String::new(),
        ops => format!(
            " ops={}",
            ops.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
        ),
    };
    let pos = if e.pos == 1 { 'b' } else { 'h' };
    let parent = e.parent.map_or(-1, i64::from);
    format!("{} {parent} {pos} {letters}{alt} {}{ops}", e.seq, e.aux)
}

/// Every unit of `src`, its events rendered as the table spells them.
fn rendered(ext: &str, src: &str) -> String {
    let lang = Lang::judged_path(Path::new(&format!("x.{ext}"))).expect("a judged extension");
    let sp = spec::spec(lang);
    let tree = ast::parse_lang(src, lang).expect("a grammar");
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .iter()
        .map(|u| {
            emit(u.node, src.as_bytes(), sp)
                .iter()
                .map(show)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect::<Vec<_>>()
        .join(" ; ")
}

const ROWS: &str = r#"
ts @@ 0 -1 h NIK2 0 | 1 0 b FDA 0 | 2 1 b NIKD2 0 | 3 2 b FDA 0 @@ an else clause is flat, direct and in the if's alternative; the if under it is direct (the core reads that pair as an else-if) and its own else clause follows the same way; an if whose alternative holds a flat kind states class 2
function f(a: number): number {
  if (a > 0) {
    return 1;
  } else if (a < 0) {
    return -1;
  } else {
    return 0;
  }
}
====
go @@ 0 -1 h NIK3 0 @@ Go hangs the else body straight off the alternative field: a block no class names, so the if states class 3 and the core pays the else on the if
func f(a int) int {
	if a > 0 {
		return 1
	} else {
		return 0
	}
}
====
go @@ 0 -1 h NIK1 0 | 1 0 b NIKDA 0 @@ the next if in the alternative field is direct and in-alt under an if stating class 1 — the field form of an else-if
func g(a int) int {
	if a > 0 {
		return 1
	} else if a < 0 {
		return -1
	}
	return 0
}
====
py @@ 0 -1 h NIK2 0 | 1 0 b FKDA 0 | 2 0 b FD 0 @@ an elif clause is flat and a cyclomatic kind, direct and in-alt (the field's first child); the else clause is flat and direct but not in-alt, the field's first child being the elif
def f(a):
    if a > 0:
        return 1
    elif a < 0:
        return -1
    else:
        return 0
====
rs @@ 0 -1 h NIK2 0 | 1 0 h CD 2 | 2 0 b FDA 0 @@ a let chain is a chain of two operands in the if's header, with no operator field and so no logic root; the else clause is flat, direct and in-alt
fn f(x: Option<i32>, y: Option<i32>) -> i32 {
    if let Some(a) = x && let Some(b) = y { a + b } else { 0 }
}
====
go @@ 0 -1 h NK 0 | 1 0 b NK 0 | 2 1 b NIK 0 | 3 2 b J 0 @@ a labelled statement is transparent, so the loop it wraps is the first event; each body is a block no class names, so nothing under a loop is direct; the labelled continue is a jump event and the plain continue is no event at all
func f() {
OUT:
	for i := 0; i < 3; i++ {
		for j := 0; j < 3; j++ {
			if j == 1 {
				continue OUT
			}
			continue
		}
	}
}
====
ts @@ 0 -1 h PL 0 ops=0,1 | 1 0 h PD 0 @@ the outermost operator node is the logic root and carries the chain's operator ids in source order (0 = &&, 1 = ||); the inner operator node is an operator event of its own, direct under the root
function f(a: boolean, b: boolean, c: boolean): boolean {
  return a && b || c;
}
====
ts @@ 0 -1 h NIK 0 ; 0 -1 h NK 0 @@ the arrow function inside the call is a unit of its own: its ternary is the second unit's event and never the first's
function outer(xs: number[]): number[] {
  if (xs.length > 0) {
    return xs.map((x) => (x > 0 ? x : -x));
  }
  return [];
}
====
ts @@ 0 -1 h NK 0 | 1 0 b NK 0 @@ a ternary nests whole: every child of it is body, so the inner ternary is in the outer one's body though it sits in its condition
function f(a: number): number {
  return (a > 5 ? a : 0) > 0 ? 2 : 1;
}
====
c @@ 0 -1 h NIK 0 @@ the `#if` condition is an opaque field: its `&&` never reaches the stream, while the if under the directive does, under the unit itself
int f(int a) {
#if defined(X) && defined(Y)
  if (a) { return 1; }
#endif
  return 0;
}
"#;

#[test]
fn every_shape_states_the_bits_the_rules_read() {
    for block in ROWS.trim().split("\n====\n") {
        let (head, src) = block.split_once('\n').expect("a header over a source");
        let [ext, want, why]: [&str; 3] = head
            .split(" @@ ")
            .collect::<Vec<_>>()
            .try_into()
            .expect("ext @@ events @@ why");
        assert_eq!(rendered(ext, src), want, "{why}\n--- source ---\n{src}");
    }
}

/// The emitter never descends a unit node, so a nest-only kind that
/// is also a unit kind could never fire — the dead-entry rule the
/// launch tables state (CE.Lang.Python, .TypeScript, .Rust, .Go),
/// held for every grammar.
#[test]
fn no_nest_only_kind_is_a_unit_kind() {
    for lang in Lang::with_grammar() {
        let sp = spec::spec(lang);
        let dead: Vec<_> = sp
            .coc_nest_only_kinds
            .iter()
            .filter(|k| sp.fn_kinds.contains(k))
            .collect();
        assert!(
            dead.is_empty(),
            "{lang:?}: nest-only kinds that are units: {dead:?}"
        );
    }
}
