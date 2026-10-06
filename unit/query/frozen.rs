//! The frozen query front end (plan v2.33 W1): lexer.rs, program.rs,
//! columns.rs and legend.rs in oracle/ are byte copies of
//! cli/src/query/ at 1324c927, the last commit before query/1 began
//! carrying the program's texts and the core began lexing them
//! (CE.Query.Lex, CE.Query.Front); the trailing unit-test mount of each
//! copy reads `../x.rs`, the one edit, so the unit tests of the scanner,
//! the goal heads and the schema keep testing the code they were written
//! for. legend.rs is the schema half alone: its vocabulary half (node
//! kinds through `vocabulary()`) is still live in cli/src/query/legend.rs
//! and tested there (unit/query/vocabulary.rs). query/mod.rs mounts this
//! module for tests only; the differential leg (unit/query/lexed.rs)
//! holds the core's lex reply against it.
//!
//! Each copy is mounted here by `#[path]` (oracle/ holds no mod.rs: a
//! parent there would turn the copies' `super::` references into edges
//! back to it, a cycle — graph/frozen_cfg.rs, same reason). The copies
//! find each other as siblings under this module, as they did under
//! query/.

#[path = "oracle/columns.rs"]
pub mod columns;
#[path = "oracle/legend.rs"]
pub mod legend;
#[path = "oracle/lexer.rs"]
pub mod lexer;
#[path = "oracle/program.rs"]
pub mod program;
