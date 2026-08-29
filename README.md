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
| `unit/` | `cli/tests/unit/` | the crate's unit tests, one file per `#[cfg(test)]` module of `cli/src` (mirrored path; a `mod.rs` host names its directory), mounted back by `#[cfg(test)] #[path = "../tests/unit/…"] mod tests;` in the source file (plan v2.18 step #13); the crate's tarball ships exactly these files and nothing else of this tree, and `it/unit_mounts.rs` holds src mounts, this directory and the tarball to one set |

## Running

The suite is white-box: every file under `unit/` is a module of the
`codeeraser` crate and 44 of the 80 integration files call its library
API directly, so it only builds inside the superproject:

```bash
git clone --recurse-submodules https://github.com/skymanbp/CodeEraser
cd CodeEraser/cli && cargo test --release --test it
node ../cli/tests/gui/lens_invariant.js     # from cli/, or from the root: node cli/tests/gui/…
```

A change to a test lands here first, then the superproject bumps its
submodule pointer. In the superproject this tree is a **reader** — its
references feed the graph and the advisory's mention universe — and
never a measured part (user ruling 2026-08-28, plan v2.18 step #12):
what measures it is its own `ce.toml` and `ce-baseline.json` here, the
same six gates the superproject's CI runs on itself, rooted in this
directory (`ce <cmd> cli/tests`).

## Licence

Apache-2.0, the superproject's licence (`LICENSE`).
