//! The dead-file exam's record, committed (lane v235_licence, part 3A):
//! the bars and their four addenda, the auditors' brief, both readings'
//! answers, the main session's settlement of the rows read two ways, and
//! the citation input of BARS §3 — each copied byte for byte from the
//! main session's review directory and pinned here by its sha256, so the
//! exam is re-read from a clone. Readers of the answers, the settlement
//! and the convention input go through this module; nothing reads the
//! review directory any more.

use super::conventions::Docs;
use crate::common::repo_root;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// `sha256 path` of every record file (all thirty-three), `path` relative to
/// `contracts/eval/`; the source file it was copied from is named in the
/// registry page (docs/EVAL-SET-DEADFILE.md).
pub const PINNED: &str = "
2934ab1c7f33f3373065a8fbc37b349a35b816481a882a4e94b7c3145a29fa65 deadfile-bars-v1.md
d42c6f7020c05dac791901f75d137fd9d6c42f29a94bee643cbc4f80a0024a31 deadfile-bars-addendum-1-v1.md
e2d5860908eb00ca722d4b83a1c557563e72feba3d1fb59f88e8c63b11baa07b deadfile-bars-addendum-2-v1.md
8bc0404efe4e4b5fc32bc41029c9c3c95bb7f55fcfecf4b2111c6668aaf59667 deadfile-bars-addendum-3-v1.md
8e496a0bfde490828defa81df1980aa303df411e4acc8298859833f5140245f9 deadfile-bars-addendum-4-v1.md
b7c9f6aa86224f381e9c0df9f2a19b24fbd49ff75a53cd67187b5ba113e83816 deadfile-settlement-v1.json
9598a85ca1ea28a3b4485048010e1ba2da658935867f559099c17738860cb71e deadfile-convention-docs-v1.json
c852aaed2f45e6c0331e23046c133987003c6669ed1b7c4172a129c435f49f8a deadfile-answers-v1/cpp-python.jsonl
95fdca32c5873237e4c8bf2a649638a136c2eeb47f2f836a0eeea4f1dfcb44a9 deadfile-answers-v1/html-1.jsonl
055f439bb4f960b968ec2ba18f10a87366fcbfeb8c58de7d3de72826b0f5de45 deadfile-answers-v1/html-2.jsonl
db3cfc46b003260375d4921ca604ab8f3a08316d9b4d2b18877c0e0905205e3d deadfile-answers-v1/java-gson.jsonl
e1471ff13f005c32557a1673365d718097d4c6a74e006a88f52bfa7f27d3096c deadfile-answers-v1/java-jsoup.jsonl
835bf42eb59da726f074834dce9a13e2a5d4277a131d9fc3672710207d761794 deadfile-answers-v1/lua-koreader-1.jsonl
abac7d91cb8c95cfef52a2beda84772d075d480809d671873c0497f66b662cb9 deadfile-answers-v1/lua-koreader-2.jsonl
3c43399d22e5f2e20dbb4fc1f9c8f45fbc60831c0be65db9ecb22e907254c225 deadfile-answers-v1/lua-lua.jsonl
07c06f0f93d04e10bd1e675bf33edc0a6f412b21cfb02298641b14b21a7bb43a deadfile-answers-v1/lua-luarocks-1.jsonl
3931a1cb0ad7919bd01f49639bb647fb665f190760f5e87ee33d02b12a8ef35c deadfile-answers-v1/lua-luarocks-2.jsonl
4511c9f69f6d321efe7830f08a9df6c93e33c5fe4a2fea60a1845f371c413a3b deadfile-answers-v1/markdown-1.jsonl
3af4b9d282dccb4e1acf13cd94a3b0b22795b649eef79f8390d7d09fe5aa8b1e deadfile-answers-v1/markdown-2.jsonl
1aa9f9d63e861f755c4839a0036b6d14510dd1f9108133a76fa44d7b5933155d deadfile-answers-v1/r-covid19model-1.jsonl
d099d06a2bb1970d7db6c8cd44e4f72e1440dff15942d036e4077d0ca7a728ab deadfile-answers-v1/r-covid19model-2.jsonl
1124069d7b1ad4d1f0adc5c50eb49028c2b33d98ff59bfc85db5467da791dabe deadfile-answers-v1/r-covid19model-3.jsonl
42dc500ccf6fd6d083dd438d458d1a863327f8040a20f059d1133e142b7a2525 deadfile-answers-v1/tsx-zod-1.jsonl
8ca83f601972e8c05138b1a6ba92fe0e5d45e0ad70721e92fbd41033ba34f6f3 deadfile-answers-v1/tsx-zod-2.jsonl
9870eb501bf213610f05937e58af277ad835d84b729eeea2c1a2678b656a23db deadfile-answers-v1/typescript-zod-1.jsonl
f141947ff844ade75dda8a045d1954074b3f4547caca376e79734e156a835e30 deadfile-answers-v1/typescript-zod-2.jsonl
f6911b4c69c54d0b6af664456018d2cbe7ac9d0b1adb58b9b0cfd2020a97502c deadfile-answers-v1/typescript-zod-3.jsonl
68df49e1e831e8afb2e430f9aee44259a7a8eb354839a17e47c7d005399d2791 deadfile-answers-v1/typescript-zod-4.jsonl
282e718bbef7e3601719b744ac9289690a8e0c87567e3811ee1f7e349cf976d5 deadfile-answers2-v1/second-1.jsonl
5fd4532769a5fab7d7cf246ba6b8abe016bcc2d9061f08ab5dd1f988fbe53af8 deadfile-answers2-v1/second-2.jsonl
223b3eb2ccca6344725d3c08b270d384c814142952c68e1406a55c22c73bc147 deadfile-answers2-v1/second-3.jsonl
bc77b61dc4de7d5e415b5f913766ddf7580be1376cf8cf2701d4c93b2ed40de1 deadfile-answers2-v1/second-4.jsonl
04d032746a79f787d41a4fa03389b05f297ea42f3277d7e2ba4880563b871372 deadfile-auditor-brief-v1.md
";

/// The two answer directories: (first reading, second reading).
pub const ANSWERS: (&str, &str) = ("deadfile-answers-v1", "deadfile-answers2-v1");

/// A record path under `contracts/eval/`.
pub fn at(rel: &str) -> PathBuf {
    repo_root().join("contracts/eval").join(rel)
}

/// The bytes of a record file, read as text (every record file is UTF-8).
pub fn text(rel: &str) -> String {
    std::fs::read_to_string(at(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// One answer and where it stands: (file, 1-based row, answer).
pub type Line = (String, usize, Value);

/// Every answer of one reading's directory, in file then row order; an
/// unreadable directory or line is refused.
pub fn answer_lines(dir: &str) -> Vec<Line> {
    let mut files: Vec<_> = std::fs::read_dir(at(dir))
        .expect(dir)
        .flatten()
        .map(|e| e.path())
        .collect();
    files.retain(|p| p.extension().is_some_and(|x| x == "jsonl"));
    files.sort();
    let mut out = Vec::new();
    for p in files {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = std::fs::read_to_string(&p).expect("answers file");
        for (i, line) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            out.push((
                name.clone(),
                i + 1,
                serde_json::from_str(line).expect("answer line"),
            ));
        }
    }
    out
}

/// A row's identity in the record: (corpus, path).
pub fn key(v: &Value) -> (String, String) {
    let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
    (s("corpus"), s("path"))
}

/// Answers, or any record rows, by (corpus, path).
pub fn by_row<'a>(rows: impl IntoIterator<Item = &'a Value>) -> BTreeMap<(String, String), Value> {
    rows.into_iter().map(|a| (key(a), a.clone())).collect()
}

/// Both readings: (first, second), each by (corpus, path).
pub fn readings() -> [BTreeMap<(String, String), Value>; 2] {
    [ANSWERS.0, ANSWERS.1].map(|d| by_row(answer_lines(d).iter().map(|(_, _, a)| a)))
}

/// The main session's settlement, by (corpus, path).
pub fn settlement() -> BTreeMap<(String, String), Value> {
    let rows: Value =
        serde_json::from_str(&text("deadfile-settlement-v1.json")).expect("settlement");
    by_row(rows.as_array().expect("settlement array"))
}

/// BARS §3's citation input: candidate name -> {url, quote, position, ...}.
pub fn convention_docs() -> Docs {
    serde_json::from_str(&text("deadfile-convention-docs-v1.json")).expect("convention docs")
}

/// Every record file with its pinned sha256: (path, sha).
pub fn pinned() -> Vec<(&'static str, &'static str)> {
    pins(PINNED)
}

/// The `(path, sha256)` pins of a `sha path` text, one a line.
pub fn pins(text: &'static str) -> Vec<(&'static str, &'static str)> {
    text.lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(h, p)| (p, h))
        .collect()
}
