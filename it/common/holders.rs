//! Who is executing a file we want to write. `tables_package` copies the
//! core and later opens the copy for writing; on ubuntu CI that open
//! has failed twice with ETXTBSY ("Text file busy"), green on rerun,
//! and an audit of `ce` found no path that leaves a core child alive.
//! The standing ruling is to measure the holder, never to retry or
//! sleep: on a failed open this module reads `/proc` (no external
//! tools) and the panic names every process whose exe, open fds or
//! mappings point at the file. The listing is taken twice back to back,
//! so a holder in the middle of exiting shows as present-then-gone.

use std::fs::File;
use std::path::Path;

/// Open `path` for writing; on error panic with the error and the
/// holder report. No retry: the first failure is the measurement.
pub fn open_for_write(path: &Path) -> File {
    File::options().write(true).open(path).unwrap_or_else(|e| {
        let first = report(path);
        let second = report(path);
        panic!(
            "open {}: {e:?}\n-- holders at the failed open --\n{first}\
                 -- holders again, after the first listing finished --\n{second}",
            path.display()
        )
    })
}

#[cfg(not(target_os = "linux"))]
fn report(_path: &Path) -> String {
    "holders: not measured on this OS\n".to_string()
}

#[cfg(target_os = "linux")]
fn report(path: &Path) -> String {
    linux::report(path)
}

#[cfg(target_os = "linux")]
mod linux {
    use std::fs;
    use std::io::ErrorKind;
    use std::path::Path;

    /// One pass over `/proc`: what was looked at, what could not be
    /// read, and what points at the file.
    #[derive(Default)]
    struct Pass {
        scanned: usize,
        unreadable: usize,
        hits: Vec<String>,
        pids: Vec<u32>,
    }

    pub(super) fn report(path: &Path) -> String {
        let want = path.to_string_lossy().into_owned();
        let names = [want.clone(), format!("{want} (deleted)")];
        let meta = fs::metadata(path)
            .map(|m| format!("len {} mtime {:?}", m.len(), m.modified().ok()))
            .unwrap_or_else(|e| format!("metadata: {e}"));
        let mut out = format!("test pid {} file {want} ({meta})\n", std::process::id());
        let entries = match fs::read_dir("/proc") {
            Ok(e) => e,
            Err(e) => return format!("{out}/proc unreadable: {e}\n"),
        };
        let mut pass = Pass::default();
        for pid in entries
            .flatten()
            .filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok())
        {
            pass.scanned += 1;
            probe(pid, &names, &mut pass);
        }
        out += &format!(
            "scanned {} /proc entries, unreadable: {}, holders: {}\n",
            pass.scanned,
            pass.unreadable,
            pass.pids.len()
        );
        for line in &pass.hits {
            out += &format!("  {line}\n");
        }
        for pid in &pass.pids {
            out += &describe(*pid);
        }
        out
    }

    /// Does `pid` execute, hold open or map the file? A permission
    /// error on another user's process is counted, not reported.
    fn probe(pid: u32, names: &[String; 2], pass: &mut Pass) {
        let base = format!("/proc/{pid}");
        let mut denied = false;
        let mut found = Vec::new();
        match fs::read_link(format!("{base}/exe")) {
            Ok(exe) if names.iter().any(|n| exe.to_string_lossy() == n.as_str()) => {
                found.push(format!("pid {pid}: exe {}", exe.display()));
            }
            Err(e) => denied |= e.kind() == ErrorKind::PermissionDenied,
            Ok(_) => {}
        }
        match fs::read_dir(format!("{base}/fd")) {
            Ok(fds) => found.extend(fds.flatten().filter_map(|fd| {
                let target = fs::read_link(fd.path()).ok()?;
                let hit = names.iter().any(|n| target.to_string_lossy() == n.as_str());
                hit.then(|| {
                    format!(
                        "pid {pid}: fd {} -> {}",
                        fd.file_name().to_string_lossy(),
                        target.display()
                    )
                })
            })),
            Err(e) => denied |= e.kind() == ErrorKind::PermissionDenied,
        }
        match fs::read_to_string(format!("{base}/maps")) {
            Ok(maps) => found.extend(mapped(pid, &maps, names)),
            Err(e) => denied |= e.kind() == ErrorKind::PermissionDenied,
        }
        pass.unreadable += usize::from(denied);
        if !found.is_empty() {
            pass.pids.push(pid);
            pass.hits.extend(found);
        }
    }

    /// The `maps` lines whose path column is the file: their count and
    /// the first of them.
    fn mapped(pid: u32, maps: &str, names: &[String; 2]) -> Option<String> {
        let lines: Vec<&str> = maps
            .lines()
            .filter(|l| {
                let path = l.splitn(6, ' ').nth(5).unwrap_or("").trim_start();
                names.iter().any(|n| path == n.as_str())
            })
            .collect();
        let first = lines.first()?;
        Some(format!(
            "pid {pid}: {} maps lines, first: {first}",
            lines.len()
        ))
    }

    /// Name / State / PPid / Tgid, the command line and the parent
    /// chain up to pid 1, so an orphaned core and a `ce` mid-exec read
    /// differently.
    fn describe(pid: u32) -> String {
        let fields = ["Name", "State", "PPid", "Tgid"]
            .map(|k| format!("{k} {}", status(pid, k).unwrap_or_else(|| "?".into())))
            .join(", ");
        let cmdline = fs::read(format!("/proc/{pid}/cmdline"))
            .map(|b| String::from_utf8_lossy(&b).replace('\0', " "))
            .unwrap_or_else(|e| format!("cmdline: {e}"));
        let cmdline: String = cmdline.trim_end().chars().take(200).collect();
        let mut chain = vec![format!("{pid}")];
        let mut at = pid;
        while at > 1 && chain.len() < 64 {
            let Some(parent) = status(at, "PPid").and_then(|p| p.parse::<u32>().ok()) else {
                chain.push("?".into());
                break;
            };
            let name = status(parent, "Name").unwrap_or_else(|| "?".into());
            chain.push(format!("{parent} ({name})"));
            at = parent;
        }
        format!(
            "  pid {pid}: {fields}\n    cmdline: {cmdline}\n    chain: {}\n",
            chain.join(" -> ")
        )
    }

    /// One field of `/proc/<pid>/status`.
    fn status(pid: u32, key: &str) -> Option<String> {
        let text = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
        text.lines().find_map(|l| {
            let (k, v) = l.split_once(':')?;
            (k == key).then(|| v.trim().to_string())
        })
    }
}
