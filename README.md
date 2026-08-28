# CodeEraser-tests

The test suite of [CodeEraser](https://github.com/skymanbp/CodeEraser),
kept in its own repository and mounted in the superproject as the git
submodule at `cli/tests` (plan v2.18, 2026-08-28). The history here is
the superproject's own history of `cli/tests/` and `gui/tests/`,
extracted with `git filter-repo`; nothing was rewritten but the paths.

## Layout

| here | in the superproject | what |
|---|---|---|
| `it/` | `cli/tests/it/` | the one Rust integration binary (`cargo test --test it -- <module>::`) |
| `corelink_deadline.rs`, `daemon_conn_deadline.rs` | `cli/tests/` | the two root binaries that must run alone |
| `gui/*.js` | `cli/tests/gui/` (formerly `gui/tests/`) | the four node gates over the GUI's JS |

## Running

The suite is white-box: 44 of its 80 files call the `codeeraser` library
API directly, so it only builds inside the superproject:

```bash
git clone --recurse-submodules https://github.com/skymanbp/CodeEraser
cd CodeEraser/cli && cargo test --release --test it
node ../cli/tests/gui/lens_invariant.js     # from cli/, or from the root: node cli/tests/gui/…
```

A change to a test lands here first, then the superproject bumps its
submodule pointer — the superproject's CI checks out submodules and its
self-score keeps counting every file in this tree (user ruling
2026-08-28: the score includes the tests).

## Licence

Apache-2.0, the superproject's licence (`LICENSE`).
