//! One MCP server session over a project directory — the harness the
//! catalog round-trip (mcp_precommit.rs) and every per-tool face test
//! (similar_face.rs) drive. Lifted out of the catalog test when the
//! similar face needed the same spawn/ask/finish trio: a second copy
//! would have been this repo's own clone verdict.

use std::io::{BufRead, BufReader, Write as _};
use std::process::{Command, Stdio};

/// Server over a project + a request method; EOF on `finish` ends
/// `serve()`.
pub struct McpSession {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    lines: std::io::Lines<BufReader<std::process::ChildStdout>>,
}

impl McpSession {
    pub fn over(dir: &std::path::Path) -> McpSession {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
            .arg("mcp")
            .arg(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mcp");
        let stdin = child.stdin.take().expect("stdin");
        let lines = BufReader::new(child.stdout.take().expect("stdout")).lines();
        McpSession {
            child,
            stdin,
            lines,
        }
    }

    pub fn ask(&mut self, req: serde_json::Value) -> serde_json::Value {
        writeln!(self.stdin, "{req}").expect("write");
        self.stdin.flush().expect("flush");
        serde_json::from_str(&self.lines.next().expect("reply").expect("line")).expect("json")
    }

    /// One `tools/call` — the reply's `result` object.
    pub fn call(&mut self, id: u64, name: &str, args: serde_json::Value) -> serde_json::Value {
        let got = self.ask(serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        got["result"].clone()
    }

    pub fn finish(mut self) {
        drop(self.stdin); // EOF ends serve()
        let _ = self.child.wait();
    }
}
