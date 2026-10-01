//! One MCP server session over a project directory — the harness the
//! catalog round-trip (mcp_precommit.rs) and every per-tool face test
//! (similar_face.rs) drive. Lifted out of the catalog test when the
//! similar face needed the same spawn/ask/finish trio: a second copy
//! would have been this repo's own clone verdict. The trio itself is
//! session.rs's `LineSession` since the core's golden regenerator
//! needed it too (plan v2.31 step 1); this type adds the `tools/call`
//! envelope over it.

use super::session::LineSession;
use std::process::{Command, Stdio};

/// Server over a project + a request method; EOF on `finish` ends
/// `serve()`. The server's stderr is dropped: its notices are not the
/// catalog under test.
pub struct McpSession(LineSession);

impl McpSession {
    pub fn over(dir: &std::path::Path) -> McpSession {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ce"));
        cmd.arg("mcp").arg(dir).stderr(Stdio::null());
        McpSession(LineSession::spawn(cmd, "mcp"))
    }

    pub fn ask(&mut self, req: serde_json::Value) -> serde_json::Value {
        self.0.ask(req)
    }

    /// One `tools/call` — the reply's `result` object.
    pub fn call(&mut self, id: u64, name: &str, args: serde_json::Value) -> serde_json::Value {
        let got = self.ask(serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }));
        got["result"].clone()
    }

    /// One `tools/call` that must succeed — its text, for comparing
    /// with the document the library face prints.
    pub fn relayed(&mut self, id: u64, name: &str, args: serde_json::Value) -> String {
        let got = self.call(id, name, args);
        assert_eq!(got["isError"], false, "{name}: {got}");
        got["content"][0]["text"]
            .as_str()
            .expect("text")
            .to_string()
    }

    pub fn finish(self) {
        self.0.finish(); // EOF ends serve()
    }
}
