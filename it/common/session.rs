//! One child spoken to a line at a time over piped stdio — the trio
//! (spawn, ask, finish) the MCP server session (mcp.rs), the wire
//! golden round trip (core_wire.rs) and the golden regenerator
//! (fixture_contract.rs) share. It stood as two spawn stanzas, the
//! MCP session's and core_wire.rs's, until the write guard pointed at
//! the third the regenerator would have been (plan v2.31 step 1).

use std::io::{BufRead, BufReader, Write as _};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

/// A child with its stdin and stdout piped; stderr is the caller's
/// `Command`'s choice (the MCP server's is dropped, the core's shows).
pub struct LineSession {
    child: Child,
    stdin: ChildStdin,
    lines: std::io::Lines<BufReader<ChildStdout>>,
}

impl LineSession {
    pub fn spawn(mut cmd: Command, what: &str) -> LineSession {
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn {what}: {e}"));
        let stdin = child.stdin.take().expect("stdin");
        let lines = BufReader::new(child.stdout.take().expect("stdout")).lines();
        LineSession {
            child,
            stdin,
            lines,
        }
    }

    /// One request line in, the reply line out, bytes untouched — the
    /// goldens compare on this.
    pub fn ask_line(&mut self, line: &str) -> String {
        writeln!(self.stdin, "{line}").expect("write");
        self.stdin.flush().expect("flush");
        self.lines.next().expect("reply").expect("line")
    }

    /// One JSON request, its reply parsed.
    pub fn ask(&mut self, req: serde_json::Value) -> serde_json::Value {
        serde_json::from_str(&self.ask_line(&req.to_string())).expect("json")
    }

    /// EOF ends the child; its exit status comes back.
    pub fn finish(mut self) -> ExitStatus {
        drop(self.stdin);
        self.child.wait().expect("wait")
    }
}

/// The core at CE_CORE_BIN over its own stdio wire.
pub fn core_session() -> LineSession {
    LineSession::spawn(Command::new(super::core_bin()), "ce-core")
}
