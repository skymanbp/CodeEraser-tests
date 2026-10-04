//! The random legs' word tables, one row each: `<leg> <NAME>=<words>`,
//! the words joined by `|` (an empty word allowed: a leading, doubled or
//! trailing `|`). One text, so the tables are data and not code.

const TABLES: &str = r"
py DIRS=|a|a/b|a/b/c|src|src/a|src/pkg|lib|lib/a|pkg|pkg/sub|x
py BASES=__init__.py|__init__.py|x.py|y.py|a.py|b.py|c.py|sub.py|os.py|requests.py|pkg.py|notes.md
py ROOTS=lib|src|./lib|lib/|pkg||..|a/b|a/../lib|pkg/sub|x//
py SEGS=a|b|c|x|y|sub|os|pkg|requests|json|__future__|src|lib||a/b|..|__init__|dep
lua DIRS=|src|lua|a|a/b|lib|lib/a|x/lua|spec|a/b.x|lib/a/b.x|a/b/init
lua BASES=a.lua|b.lua|init.lua|x.lua|string.lua|bx.lua|b_x.lua|foo.lua|y.lua|b.luac|README.md
lua T_DIRS=|lib|a|../lib|a/./b|a//b|x/lua|src|lib/
lua SUFFIXES=.lua|/init.lua|x.lua|_x.lua|.x/y.lua|/a/b.lua|/.lua|/init/init.lua|.luac.lua
lua NAMES=a|b|x|init|foo|string|os|jit|util|lua|y||table
lua LOAD=a|b.lua|x.lua|..|.||lua|init.lua|a.lua
lua STDLIB=string|os|table|jit.util|coroutine|io|utf8
go MOD_DIRS=|a|a/b|mod2|tools
go MODULES=example.com/m|example.com/m/a|m|github.com/x/y|example.com/m/|x||/x|example.com/m/a/b
go OLDS=example.com/dep|example.com/m/a|github.com/x/y|dep|example.com/dep/sub|m|example.com/dep/
go NEWS=./local|../sibling|./a/../b|example.com/m/a|example.com/fork|dep2|m/sub|./|../..|./local/|github.com/x/y
go DIRS=|a|a/b|a/b/c|mod2|mod2/p|local|local/sub|sibling|b|tools|fork|a/sub|sub
go BASES=x.go|x.go|y.go|x_test.go|doc.md|z_test.go
go HEADS=example.com/m|example.com/m/a|m|github.com/x/y|x|example.com/dep|dep|example.com/fork|fmt|net/http|os|C|unsafe|example.com/dep/sub|golang.org/x|
go RESTS=a|b|c|p|sub|p/q||..|.
c DIRS=|src|src/sub|include|include/lib|lib|a|a/include|x/y|fw/A.framework/Headers|fw/A.framework/PrivateHeaders|build|inc
c BASES=x.h|y.h|a.h|B.h|b.c|m.cpp|u.hpp|main.c|z.h|cfg.h|t.cc
c NAMES=x.h|y.h|a.h|lib/x.h|A/B.h|../x.h|./y.h|sub/x.h|include/x.h|z.h|cfg.h|a//x.h|src/../x.h|B.h|
c PLACES=include|lib|src|a/include|inc|fw|.|..|../outside|src/sub|x|
c FORCED=cfg.h|x.h|include/x.h|../x.h|a/./x.h|y.h|z.h
c DRIVERS=cc|clang++|cl.exe|clang-cl
";

/// The table a key names: `<leg> <NAME>`.
pub(super) fn t(key: &str) -> &'static str {
    TABLES
        .lines()
        .find_map(|row| row.split_once('=').filter(|(k, _)| *k == key))
        .map(|(_, words)| words)
        .unwrap_or_else(|| panic!("no table {key}"))
}
