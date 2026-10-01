//! A stub core that shakes hands and offers nothing but `hello`: for
//! the advisory families' legs where the core has no such family,
//! which must come back named absent, never read as an empty answer.
//! Two forms, a Windows `.cmd` and a unix `.sh`, each reading the
//! hello line and answering the one hello reply.

use serde_json::json;

/// The stub's path, written under a fresh temp directory.
pub fn hello_only() -> String {
    let dir = super::tmp("stub-core-hello-only");
    let hello = json!({
        "proto": codeeraser::corelink::PROTO, "type": "hello", "server": "ce-core",
        "version": "stub", "accept": true, "capabilities": ["hello"],
    });
    let (name, body) = if cfg!(windows) {
        (
            "core.cmd",
            format!("@echo off\r\nset /p hello=\r\necho {hello}\r\n"),
        )
    } else {
        (
            "core.sh",
            format!("#!/bin/sh\nread hello\nprintf '%s\\n' '{hello}'\n"),
        )
    };
    let path = dir.join(name);
    std::fs::write(&path, body).expect("the stub core");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    path.to_string_lossy().into_owned()
}
