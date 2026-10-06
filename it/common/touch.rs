//! Move a file's mtime without opening its data for writing. Ubuntu CI
//! refused a write-open of a just-exited core three times (ETXTBSY): the
//! kernel lifts an executable's write-deny only when `mm_users` reaches
//! zero, and a concurrent /proc reader holds it past the child's exit.
//! Timestamps are not refused by that deny, so the test never writes.

use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

/// Set `path`'s modification time to `when`.
pub fn touch_modified(path: &Path, when: SystemTime) {
    let file = attributes_handle(path).expect("open for its timestamps");
    file.set_modified(when).expect("set mtime");
}

#[cfg(unix)]
fn attributes_handle(path: &Path) -> std::io::Result<File> {
    // futimens checks ownership, not the write count: read-only suffices.
    File::open(path)
}

#[cfg(windows)]
fn attributes_handle(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_WRITE_ATTRIBUTES: u32 = 0x100;
    std::fs::OpenOptions::new()
        .access_mode(FILE_WRITE_ATTRIBUTES)
        .open(path)
}
