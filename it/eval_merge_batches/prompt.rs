//! The clone-merge audit's prompt (plan v2.31 step 7, design booklet
//! §6.5; booklet 18 §6 "The audit"), one text per batch: what a
//! suggestion is, the six reasons in the order the product reads them,
//! and the answer format. Split from batches.rs so the renderer reads as
//! code and the prompt as the one text a judge reads; split in two at
//! the 50-line function line (`ce scan` counts a literal binding as a
//! function).

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
   (For 1-3, take the first differing place in source order — where
   one lies inside another, the inner one first; 1 outranks 2 and 3 at
   the same place.)
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
