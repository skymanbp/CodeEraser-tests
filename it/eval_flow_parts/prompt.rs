//! The flow exams' audit prompt (booklet §5.5 「盲判」), one text per
//! batch, and the language readings a second-generation question means
//! by (§13 item 28) — the texts batches.rs renders; split from it so
//! the renderer reads as code and the prompt as the one text a judge
//! reads.

use super::FlowExam;

/// The language readings a second-generation question means by (booklet
/// §13 item 28): what the lowering reads from commit E on, written out
/// so a judge and the product read the source the same way.
pub const READINGS_TEXT: &str = r##"## Language readings

From the exam's second generation on, these readings are part of what
a question means. Where a language's own rules leave room, they
decide. Each names something the source does implicitly; answer as if
the source spelled it out.

TypeScript / TSX
- `typeof X` in a type position (a type annotation, a type alias, a
  generic argument, `keyof typeof X`) is a read of the variable X.
- A nested function, an arrow function, a class method, and an object
  literal's getter, setter or method that mentions X reads X, however
  far below X's declaration it sits and whether or not it is ever
  called.
- A constructor parameter written with `public`, `private`,
  `protected` or `readonly` declares a field of the same name and
  assigns it from the parameter: that parameter is read.

Rust
- A format-style macro (`format!`, `println!`, `eprintln!`, `write!`,
  `writeln!`, `panic!`, `assert!` / `assert_eq!` messages, `format_args!`
  and the rest) whose string literal - raw strings included - names a
  variable inside braces reads that variable: `{name}`, `{name:?}`,
  `{name:>8}`, and a variable used as a width or precision, `{:width$}`,
  `{:.prec$}`. `{{` and `}}` are literal braces and read nothing.

C++
- A constructor's member initializer list (`: m(x), n(y)`) reads x and
  y. A declaration with a parenthesised initializer (`T v(x, y);`)
  reads x and y the way a call reads its arguments.

R
- A call to `UseMethod`, `NextMethod`, `standardGeneric` or
  `callNextMethod` reads every formal parameter of the function it sits
  in: dispatch passes the whole argument list on.
"##;

/// The readings an exam's batches are rendered under: 2 (READINGS_TEXT)
/// from the second generation on; the first generation's prompt had no
/// such section and its manifest no `readings` key, read as 1.
pub fn readings(exam: &FlowExam) -> Option<u32> {
    (exam.generation >= 2).then_some(2)
}

/// The audit prompt, verbatim; the six `{…}` placeholders are filled
/// per batch (`{READINGS}` with READINGS_TEXT and a blank line from the
/// second generation on, with nothing at the first, whose batches are
/// re-rendered byte for byte as their judges read them).
pub const PROMPT: &str = r##"# Blind audit — flow questions (batch {BATCH_ID}, language {LANG})

You are an independent reader of source code. You answer questions about
one function at a time by reading the pinned source tree below. Nobody
has told you what any tool answered; there is no tool answer in this
batch. Answer from the source alone.

Source tree (read-only, already checked out at the pinned commit):
{CLONE_ROOT}
Do not run git, do not modify files, do not read anything outside this tree.

## The four kinds of question

Every question names a file, a function (its name and its line range),
and a place inside that function. Read the whole function before
answering; read the file's imports / includes when a name's meaning
depends on them (for example whether `exit` is the standard one).

kind 0 — reachability. "The statement starting on line L (the N-th
statement that starts on that line, counting from 0 left to right)."
Answer `unreachable` if no execution path from the function's entry can
reach that statement; `reachable` if some path can. Paths follow the
language's own control flow: return / throw / raise / break / continue /
goto, loops whose condition is a constant true, `switch` fallthrough,
try / catch / finally, and calls to functions that never return
(process exit, abort, panic, an infinite loop). A call that *may* throw
does not end a path. A statement inside a branch whose condition is
merely unlikely is reachable.

kind 1 — dead store. "The write to variable X on line L (the N-th write
to X on that line, from 0)." Answer `dead` if on every path from that
write, X is written again or the function ends before X is read; `live`
if some path reads X after that write. A read is any use of X's value:
in an expression, as a call argument, in a condition, in a string
interpolation, `x += 1` (reads then writes), a member / index / deref
access `x.f` / `x[i]` / `*x` (reads x), a nested function or closure
that mentions X (reads it), `&x` / a reference bound to x (treat as a
read). A write to a member `x.f = 1` reads x, it does not write x.

kind 2 — unused local. "Local variable X declared on line L (the N-th
declaration of X on that line, from 0)." Answer `unread` if X's value
is never read anywhere in the function (by the reads listed under kind
1, nested closures included); `read` otherwise. Being written again is
not a read.

kind 3 — unused parameter. "Parameter X of the function." Same reading
as kind 2: `unread` / `read`.

If the source truly does not let you decide (a macro that hides the
statement, a truncated file), answer `cannot_tell` and say why. Use it
rarely; "hard" is not "cannot".

{READINGS}## Answer format

Write one JSON object per line to {ANSWER_FILE}, in the order given,
one line per question, nothing else in the file:

{"id": "<the question's id, copied exactly>", "truth": "<one of the two words for the kind, or cannot_tell>", "reason": "<one sentence, at most 200 characters, quoting the decisive source line(s)>"}

The id, the truth and the reason are all required. Do not add fields.
Do not answer for a question you did not read. When you finish, reply
with the count of lines written and nothing else about the answers.

## Questions

{QUESTIONS}
"##;
