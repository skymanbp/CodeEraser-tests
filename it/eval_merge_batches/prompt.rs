//! The clone-merge audit's prompt (plan v2.31 step 7, design booklet
//! §6.5; booklet 18 §6 "The audit"), one text per batch: what a
//! suggestion is, the six reasons in the order the product reads them,
//! the reading rules (merge generation 2, ruling R9: rule for rule the
//! core's rulings R2-R7 in the judge's words, so the judge and the core
//! read one table of definitions), and the answer format. Split from
//! batches.rs so the renderer reads as code and the prompt as the one
//! text a judge reads; split in three at the 50-line function line
//! (`ce scan` counts a literal binding as a function).

/// What the auditor is asked and how a suggestion is read.
pub const PROMPT_HEAD: &str = r##"# Blind audit — clone-merge suggestions (batch {BATCH_ID})

You are an independent reader of source code. Each question below is a
group of code fragments that repeat — the members of one clone group —
and asks whether the members could be merged into ONE function that
every member's place then calls. Nobody has told you what any tool
decided; the feasibility, the reason and the lines saved are withheld.
Answer from the members' source as shown.

For each group you see every member's identity (`path:unit` for a whole
function, `path:start-end` for a fragment of one), the lines the merge
would fold (`run`) and its source, and — as the tool found them — the
parameters a merged function would take: for each parameter, the text
every member has at that place. You may reject that parameterisation:
give your own parameter count.

## How to judge

A merge is feasible when the members differ only at places a parameter
can stand for — an expression (a value: an identifier used as a value,
a literal, a call, an operator expression) or a declared name — and
every such place is one parameter per distinct combination of the
members' texts (two places where every member has the same pair of
texts are one parameter). Read the reasons in this order and answer the
first that applies:

1. `spans_statements` — some place where the members differ holds a
   whole statement or more on at least one side (a statement one member
   has and the other lacks, a different run of statements).
2. `position` — some place where they differ is neither an expression
   nor a name: a statement keyword, an operator, a structural piece.
3. `type` — some place where they differ is a type (a type annotation,
   a generic argument, a cast's target type).
   (For 1-3, take the differing places in the order the tree walks
   them: a place inside another first, siblings left to right, and a
   construct's own operator / keyword / punctuation after everything
   inside it; 1 outranks 2 and 3 at the same place.)
4. `too_many_params` — every differing place could be a parameter, but
   there are more than six parameters.
5. `no_savings` — the merge would save no line: the members' run lines
   summed, less the merged function's lines and one call line per
   member, is zero or less.
6. `ok` — none of the above: the merge is feasible.

`feasible` is true exactly when the reason is `ok`. Whether the
differing values share a type is not asked: a feasible merge is a shape
that folds, not a promise that the folded function compiles.
"##;

/// The reading rules (ruling R9), one per core ruling R2-R7.
pub const PROMPT_RULES: &str = r##"## Reading rules

- A difference in an operator, a keyword or a punctuation mark is a
  `position` difference, even when everything around it matches.
- A member name, a method name, a field name or the content of a string
  literal is never a parameter by itself: read the difference as the
  smallest enclosing expression's (`a.foo` vs `a.bar`, `"x"` vs `"y"`,
  in Java `o.foo(p)` vs `o.bar(p)`) — one value parameter whose values
  are those expressions — unless that expression is the target of an
  assignment, where the difference is `position`.
- An argument, an element, a clause or a statement one member has and
  the other lacks is a structural difference: `spans_statements` when
  it holds a statement, else `position`.
- Parameters: one per distinct combination of the members' texts at a
  place, whitespace ignored; the same text in two places of different
  kinds is still one parameter.
- The merged function's lines: for whole functions the kept member's;
  for fragments the kept lines plus the helper's head and closing lines
  (Python and Haskell 1, every other language 2); plus one call line per
  member.
"##;

/// The answer format and the questions.
pub const PROMPT_ANSWER: &str = r##"## Answer format

Write one JSON object per line to {ANSWER_FILE}, in the order given,
one line per question, nothing else in the file:

{"id": "<the question's id, copied exactly>", "feasible": <true or false>, "reason": "<ok, position, type, spans_statements, too_many_params or no_savings>", "params": <your parameter count, an integer>, "note": "<one sentence, at most 200 characters, naming the decisive difference>"}

All five fields are required. Do not add fields. Do not answer for a
question you did not read. When you finish, reply with the count of
lines written and nothing else about the answers.

## Questions

{QUESTIONS}
"##;
