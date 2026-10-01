//! The per-language lowering legs (plan v2.31 step 4 A2), one child per
//! language the flow family lowers, each written as a table for
//! `shape::run_table`. Their tables were mounted beside the Rust flow
//! tables until plan v2.32 step 2 moved those into the core; the legs
//! test the lowering, so they hang under it.

mod lang_c;
mod lang_cpp;
mod lang_go;
mod lang_java;
mod lang_lua;
mod lang_py;
mod lang_r;
mod lang_rs;
mod lang_ts;
