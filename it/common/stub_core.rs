//! Stub cores: a script that shakes hands as a core and answers what
//! the leg arms it with. Since plan v2.32 step 2 no measurement runs
//! without the definition package (`tables/1`), so a stub that stands
//! for "a core without family X" must still answer the package — a copy
//! of the real core's, read once per test process from CE_CORE_BIN.
//! Two forms, a Windows `.cmd` and a unix `.sh`: each reads the hello
//! line, answers the hello reply, then, when armed with a package,
//! reads the next request line and answers it with the package reply
//! (the first request a link sends carries id 1). Both replies are
//! copied out of files beside the script, never echoed: the package
//! holds `&`, `|`, `<` and `%`, which cmd would read as syntax.

use serde_json::{Value, json};
use std::sync::OnceLock;

/// The real core's `tables.result` reply, asked once.
pub fn real_tables() -> &'static Value {
    static REPLY: OnceLock<Value> = OnceLock::new();
    REPLY.get_or_init(|| {
        let core = super::gates::core_bin();
        let (mut link, _) = codeeraser::corelink::Link::open(&core).expect("the real core");
        link.request("tables", json!({})).expect("tables/1")
    })
}

/// A stub that offers nothing but `hello` and the real package: the
/// advisory families' legs, where the core has no such family, which
/// must come back named absent, never read as an empty answer.
pub fn hello_only() -> String {
    stub("stub-core-hello-only", "7.7.0", Some(real_tables()))
}

/// A stub core answering `proto` with `capabilities` = hello (and
/// `tables/1` when armed with `tables`), its hello naming the package's
/// digest.
pub fn stub(name: &str, proto: &str, tables: Option<&Value>) -> String {
    let dir = super::tmp(name);
    let mut caps = vec!["hello"];
    let mut hello = json!({
        "proto": proto, "type": "hello", "server": "ce-core",
        "version": "stub", "accept": true,
    });
    if let Some(reply) = tables {
        caps.push("tables/1");
        hello["tablesDigest"] = reply["digest"].clone();
        std::fs::write(dir.join("tables.json"), format!("{reply}\n")).expect("the package reply");
    }
    hello["capabilities"] = json!(caps);
    std::fs::write(dir.join("hello.json"), format!("{hello}\n")).expect("the hello reply");
    let path = dir.join(if cfg!(windows) { "core.cmd" } else { "core.sh" });
    std::fs::write(&path, script(&dir, tables.is_some())).expect("the stub core");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    path.to_string_lossy().into_owned()
}

/// The script: read a line, answer from a file, once or twice.
fn script(dir: &std::path::Path, tables: bool) -> String {
    let hello = dir.join("hello.json");
    let reply = dir.join("tables.json");
    if cfg!(windows) {
        let mut s = format!(
            "@echo off\r\nset /p line=\r\ntype \"{}\"\r\n",
            hello.display()
        );
        if tables {
            s += &format!("set /p line=\r\ntype \"{}\"\r\n", reply.display());
        }
        s
    } else {
        // builtins only, like cmd's `type`: a leg may run with PATH cut
        let copy = |f: &std::path::Path| {
            format!(
                "while IFS= read -r l; do printf '%s\n' \"$l\"; done < '{}'\n",
                f.display()
            )
        };
        let mut s = format!("#!/bin/sh\nread line\n{}", copy(&hello));
        if tables {
            s += &format!("read line\n{}", copy(&reply));
        }
        s
    }
}
