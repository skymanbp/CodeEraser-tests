//! The dead-file exam, generation 2 (lane v235_licence, part 4B): which
//! states of an unreferenced file may block, read from the history of
//! ten fresh corpora under bars written first
//! (`contracts/eval/deadfile-gen2-bars-v1.md`, the design it measures
//! `deadfile-gen2-design-v1.md`, both the main session's bytes, pinned
//! here). The registry (`deadfile-gen2-registry-v1.json`) holds the ten
//! corpora and their tips, committed before any replay prints a count;
//! its product commit is filled when the product build of part 4A has
//! landed, and the replay refuses to run until it is.

pub mod audit;
pub mod doc;
pub mod gitz;
pub mod judge;
pub mod model;
pub mod readings;
pub mod replay;
pub mod report;
pub mod run;
pub mod words;

use crate::common::repo_root;
use serde_json::Value;

/// `sha256 path` of the two record files, paths under `contracts/eval/`.
pub const PINNED: &str = "
8444a90834fcf8f430dbff0aa0cb76213d4b10a5934cfdb8753a991035d53a3e deadfile-gen2-bars-v1.md
224437fec7086f6cf990087dd8331ea9310ee0d52f7605f641d1ef47eacc2514 deadfile-gen2-design-v1.md
";

pub const REGISTRY: &str = "contracts/eval/deadfile-gen2-registry-v1.json";
pub const REGISTRY_SCHEMA: &str = "ce.eval-deadfile-gen2-registry/1.0.0";

/// BARS-GEN2 section 1's table: `language | first | replacement`.
pub const CORPORA: &str = "
Python | pallets/flask | psf/black
TypeScript | pmndrs/zustand | sindresorhus/ky
Rust | sharkdp/fd | clap-rs/clap
Go | spf13/viper | gin-gonic/gin
Java | junit-team/junit4 | apache/commons-cli
C | jqlang/jq | redis/hiredis
C++ | gabime/spdlog | nlohmann/json
Lua | lunarmodules/Penlight | leafo/lapis
R | tidyverse/purrr | r-lib/cli
Haskell | ndmitchell/hlint | haskell/aeson";

/// Section 1's replacement rule: fewer first-parent commits than this.
pub const MIN_FIRST_PARENT: u64 = 500;

/// The `(language, first, replacement)` rows of section 1.
pub fn corpora() -> Vec<(&'static str, &'static str, &'static str)> {
    CORPORA
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(
            |l| match l.split(" | ").map(str::trim).collect::<Vec<_>>()[..] {
                [language, first, replacement] => (language, first, replacement),
                _ => panic!("corpus row {l:?}"),
            },
        )
        .collect()
}

/// The `(path, sha256)` pins of the record.
pub fn pinned() -> Vec<(&'static str, &'static str)> {
    crate::eval_deadfile_parts::audit::record::pins(PINNED)
}

pub fn registry() -> Value {
    crate::eval_support::load(&repo_root().join(REGISTRY).to_string_lossy())
}

/// The registry's product commit; the replay runs only once it is set.
pub fn product_commit(registry: &Value) -> Option<&str> {
    registry["product"]["commit"].as_str()
}

/// A registry row's directory name under the clone root: the repository's
/// name (`pallets/flask` -> `flask`).
pub fn dir_name(row: &Value) -> &str {
    let chosen = row["chosen"].as_str().expect("chosen");
    chosen.rsplit('/').next().unwrap_or(chosen)
}
