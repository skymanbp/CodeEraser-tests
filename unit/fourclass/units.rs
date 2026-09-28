use super::*;

fn keyed(src: &str, lang: Lang) -> (Vec<Unit>, Vec<String>) {
    let units = segments(src, lang);
    let keys = units.iter().map(|u| u.key.clone()).collect();
    (units, keys)
}

/// `owner` at each row: the innermost unit's key, None for a toplevel row.
fn owners(units: &[Unit], want: &[(usize, Option<&str>)]) {
    for (line, key) in want {
        let got = owner(units, *line).map(|u| u.key.as_str());
        assert_eq!(got, *key, "owner of line {line}");
    }
}

#[test]
fn python_functions_become_units() {
    let (units, keys) = keyed(
        "def alpha(a, b):\n    return a\n\ndef beta():\n    pass\n",
        Lang::Python,
    );
    assert_eq!(keys, ["alpha/2", "beta/0"]);
    owners(&units, &[(2, Some("alpha/2")), (3, None)]); // line 3 is the blank line between defs
}

/// A Markdown case: the source, its keys in order, each unit's rows and
/// the owner probes (an alias because the tuple on the `let` trips
/// clippy's `type_complexity`).
type MdCase<'a> = (
    &'a str,
    &'a str,
    &'a [(usize, usize)],
    &'a [(usize, Option<&'a str>)],
);

/// Markdown sections are the ladder's own headings (plan v2.30 step
/// 5b, boundary items 27 / 35): an ATX heading opens one and the
/// preamble is toplevel; a setext heading opens one on its text's
/// first row, a fenced `# x` opens none, a closing run drops only
/// after a space, seven `#` are text; a section runs to the row above
/// the next heading.
#[test]
fn markdown_sections_are_the_ladders_headings() {
    let docs: [MdCase<'_>; 2] = [
        (
            "intro\n# One\nbody\n## Two\nmore\n",
            "One Two",
            &[(2, 3), (4, 5)],
            &[(1, None), (3, Some("One")), (5, Some("Two"))],
        ),
        (
            "Setext\n======\nbody\n```\n# fenced\n```\n# C#\ntail\n####### seven\nlast\n",
            "Setext C#",
            &[(1, 6), (7, 10)],
            &[(9, Some("C#"))],
        ),
    ];
    for (text, want_keys, spans, rows) in docs {
        let (units, keys) = keyed(text, Lang::Markdown);
        assert_eq!(keys.join(" "), want_keys, "{text:?}");
        let got: Vec<_> = units.iter().map(|u| (u.start_line, u.end_line)).collect();
        assert_eq!(got, spans, "{text:?}");
        owners(&units, rows);
    }
}

/// Plan v2.30 step 5: an HTML document's units are its elements that
/// carry an `id`, keyed `#id` over the element's rows, nested ones
/// resolving to the innermost; a heading without an id is span, a void
/// element with one is a one-line unit, and every unit is a section
/// with the document visibility.
#[test]
fn html_units_are_the_elements_with_an_id() {
    let src = "<html>\n<body id=\"main\">\n<h1>t</h1>\n<section id=\"a\">\n<p id=\"a-p\">x</p>\n</section>\n<img id=\"pic\" src=\"i.png\">\n</body>\n</html>\n";
    let (units, keys) = keyed(src, Lang::Html);
    assert_eq!(keys, ["#main", "#a", "#a-p", "#pic"]);
    // the root element has no id
    let rows = [
        (1, None),
        (3, Some("#main")),
        (5, Some("#a-p")),
        (6, Some("#a")),
        (7, Some("#pic")),
    ];
    owners(&units, &rows);
    let section = crate::fourclass::kinds::KIND_SECTION;
    assert!(
        units
            .iter()
            .all(|u| u.kind == section && u.vis == visibility::HTML_VIS),
        "{units:?}"
    );
}

/// The owning unit key of `line` in Rust `src` ("" = toplevel).
fn rust_owner(src: &str, line: usize) -> String {
    let units = segments(src, Lang::Rust);
    owner(&units, line)
        .map(|u| u.key.clone())
        .unwrap_or_default()
}

#[test]
fn nested_functions_resolve_to_innermost() {
    let src = "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}\n";
    assert_eq!(rust_owner(src, 3), "inner/0");
    assert_eq!(rust_owner(src, 5), "outer/0");
}

/// Attack review F7: impl blocks are units (methods are span-
/// contained, not top-level), and Go methods carry their receiver
/// type in the key.
#[test]
fn impl_blocks_contain_their_methods() {
    let src = "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n\
                   impl Show for A {\n    fn show(&self) {}\n}\n";
    assert_eq!(rust_owner(src, 2), "add/1");
    let units = segments(src, Lang::Rust);
    let mut impls: Vec<&str> = units
        .iter()
        .filter(|u| u.key.starts_with("impl "))
        .map(|u| u.key.as_str())
        .collect();
    impls.sort_unstable(); // extraction order is not part of the contract
    // the trait qualifier keeps a type's inherent and trait impls
    // distinct (the FPR replay caught them colliding)
    assert_eq!(impls, ["impl A", "impl B", "impl Show for A"]);
}

/// The multi-param rows pin the M5-close arity repayment (3h
/// blind-audit defect): the receiver-collapsed count keyed every
/// method `/1`; the `parameters` field carries the real list.
/// Grouped `a, b int` declares two parameters (Go spec ParameterDecl;
/// plan v2.30 step 5b — the count used to be one per declaration).
#[test]
fn go_method_keys_carry_the_receiver_type_and_real_arity() {
    let src = "func (t T) add(x int) {}\nfunc (u *U) add(x int) {}\nfunc free(x int) {}\n\
                   func (t T) mix(x int, y string) {}\nfunc (t T) grouped(a, b int) {}\n\
                   func (t T) none() {}\n";
    let (_, keys) = keyed(src, Lang::Go);
    let want = [
        "(T) add/1",
        "(*U) add/1",
        "free/1",
        "(T) mix/2",
        "(T) grouped/2",
        "(T) none/0",
    ];
    for k in want {
        assert!(keys.contains(&k.to_string()), "missing {k}; keys: {keys:?}");
    }
}

/// Named units per language — the sorted keys one source yields, a
/// block per language (the head names the language by extension): a
/// Go const or var spec at package level is a unit per name it binds
/// while the same kinds inside a function body declare locals and no
/// unit (plan v2.30 step 5b, boundary item 25); the C family keys a
/// typedef by the leaf of its declarator chain, a type specifier only
/// when it carries a body (a `struct T *` parameter type and a forward
/// `class Fwd;` are references), a namespace and an alias by name, and
/// a macro as a declaration of its own (plan v2.30 step 2). Step 5b-6
/// widened the domain by rule (fourclass/declared.rs): a Java `static`
/// field and an interface's or annotation type's constant, each
/// declarator its own unit — never an instance field, an enum constant
/// or a local; a C / C++ file-scope variable definition — never a bare
/// `extern`, a prototype (`int (*fp)(int)` is a variable, `signal`'s
/// shape a prototype), a class member or a body local, while a
/// namespace, `extern "C"`, a template head, a preprocessor branch and
/// a structured binding are read through; a TypeScript module-level
/// `const` / `let` / `var` per bound identifier, patterns included,
/// under `export` / `declare` wrappers and namespace / module /
/// `declare global` bodies — never in a function, a loop head, a
/// branch or a bare block, never a second symbol for a function the
/// extractor already named after its declarator (`f/0`, `k/0`),
/// while `g = function h()` binds both.
const NAMED: &str = "\
go @@ A B Y c f/0 x z
package p
const A = 1
const (
\tB = 2
\tc = 3
)
var x, Y int
var z = 1
func f() {
\tconst local = 1
\tvar v, w int
\t_ = v + w + local
}
====
cpp @@ A M T f/1 fp ns
struct T { int x; };
struct T *f(struct T *t) { return t; }
typedef int (*fp)(int);
class Fwd;
#define M 1
namespace ns { using A = int; }
====
java @@ A Ann B E F I K L R Rec Z c d m/0 n s v
public class F {
  public static final int A = 1, B = 2;
  static int c;
  private static String d = \"x\";
  int inst;
  public static final Runnable R = () -> {};
  enum E { X, Y; static int n; int q; }
  interface I { int K = 1; void im(); }
  void m() { int local = 1; class L { static final int Z = 1; } }
  record Rec(int q) { static int s; }
  @interface Ann { int v = 1; }
}
====
c @@ S a b d e f fn/0 fp g s1 under_else under_if
int a;
static int b = 1;
extern int c;
extern int d = 2;
int e, *f, g[3];
int (*fp)(int);
int proto(int);
int *pproto(int);
struct S { int x; } s1;
#ifdef X
int under_if;
#else
int under_else;
#endif
void fn(void) { int local; static int slocal; for (int i = 0; i < 1; i++) {} }
void (*signal(int, void (*)(int)))(int);
====
cpp @@ K K::count anon cv cv2 ns nv obj pi ref sv x y
namespace ns { int nv = 1; static int sv; namespace { int anon; } }
extern \"C\" { int cv; }
extern \"C\" int cv2;
template <class T> constexpr T pi = T(3);
class K { static int count; int inst; public: static const int M = 3; };
int K::count = 0;
auto [x, y] = std::pair<int,int>{1, 2};
K obj(1, 2);
using ns::nv;
int& ref = il;
====
ts @@ A B C G K amb c d e f/0 fn/0 g h/0 inM inN k/0 p priv r rest s t u w
import { z } from \"./z\";
export const A = 1, B = 2;
const c = 3;
let d: number;
var e = 4;
export let f = () => 1;
export const g = function h() {};
export const k = function () {};
export const { p, q: r, ...rest } = z;
export const [s, , t = 1, ...u] = [];
export const K = class {};
declare const amb: number;
namespace N { export const inN = 1; const priv = 2; }
declare module \"m\" { const inM: number; }
declare global { const G: number; }
function fn() { const loc = 1; for (const it of []) {} for (let i = 0; i < 1; i++) {} if (c) { let blk = 1; } }
class C { static sf = 1; }
for (const top of []) {}
{ const inBlock = 1; }
export const { w = 1 } = z;
";

#[test]
fn named_units_per_language() {
    for block in NAMED.split("====\n") {
        let (head, src) = block.split_once('\n').unwrap();
        let (name, want) = head.split_once(" @@ ").unwrap();
        let lang = Lang::from_path(std::path::Path::new(&format!("x.{name}"))).unwrap();
        let (_, mut keys) = keyed(src, lang);
        keys.sort_unstable();
        assert_eq!(keys.join(" "), want, "{name}");
    }
}

/// Plan v2.30 step 5b (boundary item 28, register D12): a K&R
/// definition is a unit whose parameters are the identifiers of its
/// list — the declarations between the list and the body belong to
/// the function and key nothing of their own.
#[test]
fn c_knr_definitions_are_units_with_their_identifier_list() {
    let src = "int add(a, b)\nint a;\nint b;\n{ return a + b; }\nint zero() { return 0; }\n";
    let (units, keys) = keyed(src, Lang::C);
    assert_eq!(keys, ["add/2", "zero/0"]);
    owners(&units, &[(2, Some("add/2"))]);
}

#[test]
fn named_non_function_units_are_registered() {
    // the register's CLASSES case: a relocated pub const must be
    // attributable by name (R-L2-6's 1-in-35 structural hole)
    let src = "pub const CLASSES: [&str; 4] = [\n    \"a\",\n];\n\nstruct Row {\n    id: u64,\n}\n";
    assert_eq!(rust_owner(src, 2), "CLASSES");
    assert_eq!(rust_owner(src, 6), "Row");
}
