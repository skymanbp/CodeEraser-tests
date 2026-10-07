use super::*;
use std::cell::Cell;
use std::rc::Rc;

const A: &str =
    "/// Fetch the user row by id.\nfn fetch_user(id: u64) -> User {\n    query(id)\n}\n";
const B: &str = "fn render_page(p: &Page) {\n    draw(p);\n}\n\nfn fetch_item(id: u64) -> Item {\n    query(id)\n}\n";
const C: &str = "fn load_user(id: u64) -> User {\n    query(id)\n}\n";

/// Every row of the four tables a refresh writes, ids included.
fn dump(idx: &Index) -> Vec<String> {
    ["files", "unitsig", "bag", "df"]
        .iter()
        .flat_map(|t| {
            crate::graph::load::rows(idx.raw(), &format!("SELECT * FROM {t}"), |r| {
                let n = r.as_ref().column_count();
                Ok(format!(
                    "{t} {:?}",
                    (0..n)
                        .map(|i| r.get_ref(i).map(|v| format!("{v:?}")))
                        .collect::<Result<Vec<_>, _>>()?
                ))
            })
            .expect("rows")
        })
        .collect()
}

fn index(tag: &str) -> (std::path::PathBuf, Index) {
    let dir = crate::testutil::scratch(tag);
    let idx = Index::open(&dir.join("index.db"), Params::default()).expect("open");
    (dir, idx)
}

/// One file refreshed on its own (one ask inside refresh_file).
fn put(idx: &mut Index, (rel, src, lang): (String, Vec<u8>, Lang)) {
    idx.refresh_file(&rel, &src, lang, Params::default(), false)
        .expect("refresh");
}

/// The process core's answer, with a count of the asks it served; the
/// ask numbered `fail` (1-based) refuses instead.
fn counted(asks: &Rc<Cell<usize>>, fail: usize) -> Ask {
    let asks = Rc::clone(asks);
    Box::new(move |units| {
        asks.set(asks.get() + 1);
        if asks.get() == fail {
            return Err("bags/1: core refused bags.request: probe".into());
        }
        crate::similar::bags::ask(units, &[]).map(|(terms, _)| terms)
    })
}

fn file(rel: &str, text: &str) -> (String, Vec<u8>, Lang) {
    (rel.to_string(), text.as_bytes().to_vec(), Lang::Rust)
}

/// A file with no units: it asks nothing, and is written in its turn.
fn md() -> (String, Vec<u8>, Lang) {
    ("m.md".to_string(), b"# m\n".to_vec(), Lang::Markdown)
}

/// The items the bags/1 contract counts for a text's units.
fn items(text: &str) -> usize {
    file_rows(text, Lang::Rust)
        .iter()
        .map(|(_, r)| cost(r).0)
        .sum()
}

/// A batch writes what one refresh per file wrote, row for row and id
/// for id, in one ask; a ceiling the next file would pass sends the
/// batch so far first — the items, then the line (a ceiling of 0 bytes
/// sends every file with units alone, the no-unit file asking nothing).
#[test]
fn a_batch_writes_what_one_refresh_per_file_wrote() {
    let (d1, mut one) = index("batch-one");
    for f in [file("a.rs", A), md(), file("b.rs", B)] {
        put(&mut one, f);
    }
    let all = items(A) + items(B);
    for (caps, want) in [
        ((usize::MAX, usize::MAX), 1),
        ((all - 1, usize::MAX), 2),
        ((usize::MAX, 0), 2),
    ] {
        let (d2, mut idx) = index("batch-many");
        let asks = Rc::new(Cell::new(0));
        let mut batch = Batch::asking(Params::default(), caps, counted(&asks, 0));
        for f in [file("a.rs", A), md(), file("b.rs", B)] {
            batch.push(&mut idx, f, false).expect("push");
        }
        batch.finish(&mut idx).expect("finish");
        assert_eq!(
            dump(&idx),
            dump(&one),
            "caps {caps:?}: the same rows and ids"
        );
        assert_eq!(asks.get(), want, "caps {caps:?}: asks");
        drop(idx);
        std::fs::remove_dir_all(&d2).ok();
    }
    drop(one);
    std::fs::remove_dir_all(&d1).ok();
}

/// An ask failing mid-refresh writes none of its batch: the batch asked
/// before it stands committed whole, the files of the failed batch keep
/// their old rows (row for row, ids included), and the refusal is named.
#[test]
fn an_ask_failing_mid_refresh_writes_none_of_its_batch() {
    let (dir, mut idx) = index("batch-fail");
    let (want_dir, mut want) = index("batch-fail-want");
    for f in [file("a.rs", A), file("b.rs", B)] {
        put(&mut idx, f.clone());
        put(&mut want, f);
    }
    put(&mut want, file("a.rs", C));
    let asks = Rc::new(Cell::new(0));
    let mut batch = Batch::asking(Params::default(), (items(C), usize::MAX), counted(&asks, 2));
    batch.push(&mut idx, file("a.rs", C), false).expect("waits");
    batch
        .push(&mut idx, file("b.rs", A), false)
        .expect("the first batch asked and written");
    let err = batch.finish(&mut idx).expect_err("the second ask refused");
    assert!(err.to_string().contains("probe"), "named: {err}");
    assert_eq!(asks.get(), 2);
    assert_eq!(
        dump(&idx),
        dump(&want),
        "a.rs refreshed whole, b.rs untouched"
    );
    drop((idx, want));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&want_dir).ok();
}
