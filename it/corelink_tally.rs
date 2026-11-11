//! The per-thread tally of request lines (corelink::sent_on_this_thread):
//! a line counts on the thread that ASKED for it, once per request line
//! and once per hello. Since the request bytes leave on a scoped writer
//! thread (corelink/session.rs `watched`), "the thread that wrote the
//! bytes" and "the thread that asked" are two threads; the tally is the
//! asker's, so a leg that holds "this path asks no core" reads its own
//! thread and nobody else's work moves it.

use codeeraser::corelink::{Link, sent_on_this_thread};
use serde_json::json;

use crate::common::core_bin;

/// A fresh session's hello and one request count exactly two on the
/// asking thread, though the writer thread put the bytes on the pipe; a
/// second thread that asks nothing reads 0 the whole time.
#[test]
fn the_asking_thread_counts_its_lines_and_no_other_thread_does() {
    let core = core_bin();
    let (asked, idle) = std::thread::scope(|scope| {
        let asker = scope.spawn(|| {
            let before = sent_on_this_thread();
            let (mut link, _) = Link::own(&core).expect("own a core session");
            let after_hello = sent_on_this_thread();
            let _ = link.request("tables", json!({}));
            (before, after_hello, sent_on_this_thread())
        });
        let asked = asker.join().expect("asker thread");
        let idle = scope
            .spawn(sent_on_this_thread)
            .join()
            .expect("idle thread");
        (asked, idle)
    });
    assert_eq!(
        asked,
        (0, 1, 2),
        "hello then one request, counted on the asking thread"
    );
    assert_eq!(idle, 0, "a thread that asked nothing reads 0");
}
