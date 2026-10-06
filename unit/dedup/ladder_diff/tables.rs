//! The random legs' word tables, one row each: `<leg> <NAME>=<words>`,
//! the words joined by `|` (an empty word allowed: a leading, doubled or
//! trailing `|`). One text, so the tables are data and not code.

const TABLES: &str = r#"
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
r DIRS=|R|R/sub|pkg|pkg/R|pkg/R/sub|a|a/b|scripts|x|x/R|inst
r BASES=a.R|b.R|c.r|zz.R|utils.R|b b.R|.R|x.Rmd|notes.txt|a.R
r SOURCE=a.R|b.R|c.r|R|utils.R|..|.||sub|x|b b.R|scripts|pkg
r ROOTS=R|pkg/R|x||..|R/sub|a/../R|scripts/
r PACKAGES=pkgA|pkgB|stats|utils|pkg.dots|pkgA |
r COLLATE=a.R|b.R|c.r|sub/a.R|../a.R|b b.R|zz.R||utils.R|.R
r DESC_DIRS=|pkg|x|a|R
java DIRS=src/main/java/a|src/main/java/a/b|src/test/java/a|src/test/java/a/b|lib/src/main/java/a|lib/src/main/java/a/b|lib/src/test/java/a|a|a/b|x||src/main/java|src/it/java/a
java BASES=A.java|B.java|C.java|D.java|Util.java|package-info.java|A.java|B.java|notes.md
java PACKAGES=a|a.b|x||a.c|b|a.B|java.util|org.x
java CLASSES=A|B|C|D|Util|N|M|String|List|Map|Object|Inner|X|Ghost
java SUPERS=C|N|A.N|a.b.C|a.b.C.N|B|Object|java.util.List|Util.Inner|x.D|D|A|C.M|a.A.Inner|Map.Entry|a.b.Ghost
java MEMBERS=m|N|CONST|of|Inner|Ghost
java JDK=java.util|java.util.function|java.io|javax.swing|java.lang|jdk.jfr|java.sql|org.w3c.dom|java.util.concurrent
java ROOTS=src/main/java|lib/src/main/java|a|lib|src/test/java||x|src
java ANNOTATIONS=@Tag |@a.b.Tag |@Tag(1) |@Tag(x = ")") |@A @B(1) |@Tag /* c */ |@Tag("""s)""") |@Tag('(') |@Tag("\"(") |@Tag(// x~) |@ |@1x |@Tag(|@a.* |@Tag(("(")) |@Tag(/* ) */) |@ Tag |@a. b |@Tag('\'') |@Tag("unclosed|@Tag(/* open
hs DIRS=|src|src/Data|src/A|app|lib|lib/Data|pkg|pkg/src|pkg/src/A|pkg/src/Data|other|other/lib|other/lib/Data|test|x|x/Y|Data|A|A/B|pkg/A
hs BASES=Main.hs|A.hs|B.hs|Map.hs|Util.hs|Foo.hs|lower.hs|Data.hs|Setup.hs|notes.md|A.hs-boot
hs CABAL_DIRS=|pkg|other|app|x|src
hs CABAL_NAMES=p|q|pkgA|other
hs LIBRARY=other/lib/A.hs|other/lib/B.hs|other/lib/Data/Map.hs|other/lib/Util.hs|other/lib/lower.hs|other/lib/A/B.hs|x/lib/A.hs|pkg/src/Main.hs|pkg/src/A.hs|pkg/Setup.hs
hs PACKAGES=pkgA|pkgB|other|base|containers||pkgA-x
hs HEADERS=library|Library|library internal|executable exe|Executable  exe|test-suite t|benchmark b|common shared|common base-deps|common|flag f|source-repository head|  if os(windows)|  else|library:|LIBRARY
hs FIELDS=Hs-Source-Dirs|EXPOSED-MODULES|ghc-options|name|other_modules|x y|default-language|Main-Is
hs ROOTS=src|lib|.|app|../x|src lib|src, lib|pkg/src|..|../..|./src/|src  ,app|x/Y|other/lib|
hs MODULES=A|A.B|Data.Map|Util|Foo|Main|B|Data|Src.A|A, B|Data.Map Util|lower|A.b|Foo,|Data.Util|Y.A
hs DEPENDS=base >=4 && <5|containers|pkgA|pkgB, other|other ^>=1.0|text, pkgA >= 1|, base|pkgA-x|base, containers, pkgB||(pkgA)|mtl
hs MAINS=Main.hs|A.hs|src/Main.hs|Main.lhs||../Main.hs
hs COMMONS=shared|base-deps|shared, base-deps|missing|Shared
hs INDENTS=  |	||    
hs BOOT=Prelude|Data.Map|Data.List|Control.Monad|Data.Text|System.IO|GHC.Generics|Data.IORef|Data.Map.Strict|Control.Monad.State|Data.Aeson|Text.Printf|Data.Set|Data.ByteString
hs BOOT_PACKAGES=base|containers|text|mtl|bytestring|aeson|pkgA
hs LEGS=hs-roots 11 0|hs-packages 12 1|hs-external 13 2
hs MALFORMED=lower.Case|A..B|.A|A.|"unclosed A|"" A|"pkgA"A|"pkgA"   A.B   |a|A B|Data.map|"pkgA"|9A|Ä.B
ts DIRS=|src|src/a|src/a/b|lib|types|shared|packages/p1|packages/p1/src|packages/p1/src/a|packages/p2/src|apps/web|apps/web/src|apps/web/src/x|tools/y
ts BASES=a.ts|b.tsx|index.ts|index.tsx|x.ts|x.d.ts|m.mts|c.cts|util.ts|y.ts|z.tsx|types.d.ts|readme.md|x|lib.ts|a.ts
ts PKG_DIRS=|packages/p1|packages/p2|apps/web|tools
ts CFG_DIRS=|packages/p1|apps/web|shared|apps/web/src|tools
ts CFG_NAMES=tsconfig.json|tsconfig.json|tsconfig.base.json|base.json|tsconfig.app.json
ts NM_DIRS=|packages/p1|apps/web|src|apps/web/src/x
ts NM_NAMES=react|lodash|@scope/x|@types/node|left-pad|vitest|@scope/p1|zod
ts PKG_NAMES=@scope/p1|p1|@scope/p2|web|p1|@scope/p1|tools|
ts DEPS=react|lodash|@types/node|zod|@scope/x|p1|vitest|fs
ts EXTENDS=./tsconfig.base.json|../tsconfig.json|./base|../../tsconfig.base.json|../../../../../escape.json|@tsconfig/node18/tsconfig.json|./missing.json|./base.json|../shared/base.json|.././tsconfig.app.json|../../shared/base|./tsconfig.json
ts BASE_URLS=.|./src|src|..|../../..|./|../src|apps/web/src||lib
ts PATTERNS=@app/*|~/*|@lib|*|@dup/*|@x/*/y|@pre*fix|@scope/p1|@n/*|@a*b*c
ts TARGETS=src/*|lib/*|./src/*|types/*|src/a/*|src/a/b/*|lib/index.ts|../shared/*|*|packages/p1/src/*|src/*/index|*.ts|./
ts EXPORT_KEYS=.|./sub|./lib/*|./lib/*.js|./*|./x/*/y|./null|./a/*/b/*|./lib/*.ts|import|types|./
ts EXPORT_TARGETS=./src/index.ts|./src/a/x.ts|./src/*.ts|./src/*|./types/*.d.ts|./lib/*.js|./src/*/a.ts|../p2/src/*|./missing.ts|./src/a/*.tsx
ts CONDITIONS=import|require|types|default|node|browser
ts REL_ODD=./nope|../../../../../x|./|../|./x.js|./a.mjs|./c.cjs|./index|.|./src/../x|./.js
ts ALIAS=@app/a|@app/x|@app/a/b|~/a|~/util|@lib|@dup/x|@dup/a|utils/x|src/a/b|x|a|@x/a/y|@prefix|@pre-fix|@n/x|@abc|types/x|lib|shared/x
ts WORKSPACE=@scope/p1|p1|@scope/p1/sub|p1/lib/a|p1/lib/a.js|@scope/p2/lib/x|web|web/x/a/y|@scope/p1/null|tools/a|@scope/p1/lib/a.ts|p1/a/z/b/w|@scope/p2
ts BARE=fs|node:fs|node:sea|node:nope|fs/promises|node:test/reporters|react|react/jsx-runtime|@types/node|lodash/fp|left-pad|@scope|@scope/x|@/x|..|.|/abs|node:|zod|vitest/config|sys|module|@scope/x/deep
ts PLANT_PATHS={"compilerOptions": {"baseUrl": ".", "paths": {"@app/*": ["src/*"], "~/*": ["./src/*"], "@lib": ["lib/index.ts"]}}}|{"compilerOptions": {"paths": {"@app/*": ["src/*", "lib/*"], "@dup/*": ["src/*", "src/a/*"]}}}|{"compilerOptions": {"baseUrl": "src"}}|{"extends": "./shared/base.json", "compilerOptions": {"baseUrl": "."}}|{"extends": ["./shared/base.json"], "compilerOptions": {"baseUrl": "./src", "paths": {"@dup/*": ["*", "a/*"]}}}|{"compilerOptions": {"baseUrl": ".", "paths": {"@app/*": ["src/*", "src/a/*"]}}}
ts PLANT_BASES={"compilerOptions": {"paths": {"~/*": ["src/*"], "@lib": ["lib/*"]}}}|{"compilerOptions": {"baseUrl": "..", "paths": {"@app/*": ["src/a/*"]}}}|{"extends": "../tsconfig.json"}|{}|{"compilerOptions": {"baseUrl": "../src"}}|{"compilerOptions": {"paths": {"@lib/*": ["../lib/*"]}}}
ts PLANT_PATH_FILES=src/a.ts|src/x.ts|src/a/b.ts|lib/index.ts|src/util.ts|types/x.d.ts|shared/x.ts|src/a/index.ts|lib/a.ts|src/a/x.tsx
ts PLANT_EXPORTS={".": "./src/index.ts", "./sub": {"import": "./src/a/x.ts", "types": "./src/a.ts"}, "./lib/*": "./src/*.ts"}|"./src/index.ts"|{"import": "./src/index.ts", "require": "./src/lib.ts"}|{"./lib/*": "./src/*", "./lib/a/*": "./src/a/*.ts", "./null": null}|{".": {"node": {"import": "./src/index.ts"}}, "./*": "./src/*.ts"}|{"./lib/*": ["./src/*.ts", "./src/a/*.ts"]}
ts PLANT_PKG_FILES=packages/p1/src/index.ts|packages/p1/src/a/x.ts|packages/p1/src/a.ts|packages/p1/src/lib.ts|packages/p1/src/a/y.ts|packages/p2/src/x.ts|packages/p2/src/index.ts|packages/p1/src/a/a.ts
ts LEGS=ts-relative 21 0|ts-esm 25 4|ts-paths 22 1|ts-workspace 23 2|ts-bare 24 3
ts SPELL_PATHS=src/>@app/|src/>~/|src/a/>@dup/|lib/>@lib/|src/>|>|src/>src/|lib/>@app/
ts SPELL_MEMBERS=packages/p1/src/>@scope/p1/lib/|packages/p1/src/>@scope/p1/|packages/p2/src/>p2/|packages/p1/src/a/>@scope/p1/lib/a/|packages/p1/src/>p1/lib/|packages/p2/src/>p2/lib/
rs PKG_DIRS=|crates/a|crates/b|tools/x
rs FILES=src/lib.rs|src/main.rs|src/a.rs|src/a/mod.rs|src/a/b.rs|src/b/mod.rs|src/b/c.rs|src/bin/x.rs|src/bin/y/main.rs|src/bin/y/part.rs|tests/t.rs|tests/it/main.rs|tests/it/common.rs|build.rs|src/x/y.rs|src/inner/deep.rs|src/p/q.rs|lib/core.rs|src/tools/gen.rs|src/a/b/mod.rs|src/c.rs|src/util.rs|src/x.rs|src/inner.rs|src/tests.rs|src/other.rs|src/sub/y.rs|src/lib.rs|src/main.rs
rs LOOSE=scratch/x.rs|it/main.rs|it/sub.rs|it/sub/mod.rs|a.rs|mod.rs|crates/a/src/lib.rs|crates/b/src/main.rs
rs PKG_NAMES=a|a-b|a_b|b|core|x|std|tools|serde|a
rs LIB_PATHS=src/lib.rs|src/x.rs|lib/core.rs|src/missing.rs|../escape.rs|src/a/mod.rs
rs BIN_PATHS=src/main.rs|src/tools/gen.rs|src/bin/x.rs|src/b/c.rs|../out.rs
rs DEPS=serde|tempfile|a|a-b|b|x|cc|rand|a_b
rs NAMES=a|b|c|x|y|inner|deep|tests|util|q|p|sub|tools|other|gen|common|core
rs ITEMS=Thing|Other|X|Y|a|b|inner|f|C|util|x|gen|Deep|Root|Z|deep|Thing|Other
rs EXTERN=std|core|alloc|serde|a|a_b|b|x|tempfile|test|proc_macro|rand|tools|gen|missing|a-b
rs LOCAL=self|super|super::super|super::super::super|self::super
rs PATHS=other.rs|../x.rs|sub/y.rs|a/b.rs|inner/deep.rs|../../../../escape.rs|mod.rs|b.rs|x.rs|tests.rs|a/mod.rs|./c.rs|p/q.rs|util.rs
rs VIS=|||pub |pub |pub(crate) |pub(super) 
rs DEFS=pub struct $;|struct $;|pub fn $() {}|fn $() {}|pub(crate) struct $;|pub enum $ {}|pub const $: u8 = 0;|macro_rules! $ { () => {} }|pub trait $ {}|static $: u8 = 0;|pub type $ = u8;|pub union $ { x: u8 }|impl $ {}
rs NOISE=// use crate::fake;|/* mod nope; */|const S: &str = "mod fake; use crate::x;";|use crate::{a,|#![allow(dead_code)]|fn g() {~    let _ = 1;~}|mod|use ;|use {};
rs PLANTS=src/lib.rs|src/a.rs|src/a/deep.rs|src/b/mod.rs|src/b/c.rs|src/c.rs|src/x/y.rs|src/inner/deep.rs|src/main.rs|src/bin/x.rs|tests/it/main.rs|tests/it/common.rs
rs PLANT src/lib.rs=pub mod a;~pub mod b;~mod c;~pub use a::Thing;~pub use b::*;~pub struct Root;|mod a;~pub mod b;~#[path = "x/y.rs"]~pub mod p;~pub use crate::a::deep::Deep;~pub use self::c::Other as Z;~mod c;|pub mod a;~pub mod b;~pub extern crate core as y;~pub use a::deep;~mod inner {~    pub mod deep;~    pub use super::a::Thing;~    use self::deep::Deep;~}|pub mod a;~pub mod b;~pub mod c;~pub use crate::b::c::*;~pub use a::*;~fn f() {~    use crate::a::deep::Thing;~}
rs PLANT src/a.rs=pub mod deep;~pub use self::deep::Thing;~pub use deep::*;|pub mod deep;~pub struct Thing;~pub use crate::b::Other;|pub use crate::c::*;~pub mod deep;~use super::b::c;|pub mod deep;~pub use super::b::Other;~pub use crate::Root;
rs PLANT src/a/deep.rs=pub struct Thing;~pub struct Deep;~pub fn f() {}|pub use super::super::b::Other;~pub struct Deep;|use super::Thing;~use crate::b;~pub struct Deep;~pub use crate::c::Other as Thing;
rs PLANT src/b/mod.rs=pub mod c;~pub struct Other;~pub use self::c::Thing;|pub use crate::a::*;~pub use crate::a::deep::Deep as Other;~mod c;|pub mod c;~pub use c::*;~pub use super::a::deep;|mod c;~pub extern crate std as y;~pub use self::c::C as Other;
rs PLANT src/b/c.rs=pub struct Thing;~pub struct C;|pub use super::Other;~pub struct C;~use super::super::a::Thing;|pub struct C;~pub use crate::a::*;
rs PLANT src/c.rs=pub struct Other;~pub use crate::a::Thing;~pub use super::b::*;|pub struct Other;~pub mod deep {~    pub use crate::a::deep::*;~}|pub use crate::b::Other;~pub use crate::b::c::*;
rs PLANT src/x/y.rs=pub struct Thing;~pub use crate::a::*;|pub use super::super::a::deep::Deep;~pub struct Y;
rs PLANT src/inner/deep.rs=pub struct Deep;|pub use crate::a::Thing;~pub struct Deep;
rs PLANT src/main.rs=mod a;~use crate::a::Thing;~use a::deep::Deep;|use a::Thing;~use a::b::Other;~use a_b::c::*;~fn main() {}|mod a;~mod b;~mod c;~use crate::b::Other;~use crate::c::deep::Deep;
rs PLANT src/bin/x.rs=use a::Thing;~use a::b::Other;~mod helper;|use crate::Root;~use a::deep::Deep;
rs PLANT tests/it/main.rs=mod common;~use common::Thing;~use a::b::c::C;|mod common;~use crate::common::*;~use a::Root;
rs PLANT tests/it/common.rs=pub struct Thing;~pub use a::b::Other;|pub use super::Thing;~use crate::Thing;
rs LEGS=rs-mod 31 0|rs-crate 32 1|rs-local 33 2|rs-extern 34 3|rs-binder 35 4
md DIRS=|docs|docs/sub|a|a/b|img|site|site/zh|x y|é
md BASES=README.md|a.md|b.md|index.md|x.markdown|notes.md|a b.md|a#b.md|é.md|c.py|lib.rs|A.MD|a.Md|.md|readme.md
md ASSETS=img/logo.png|style.css|a.png|docs/x.svg|data.json|docs/sub/i.png|logo.png|a b.png|site/zh/s.css
md HEADS=Intro|Getting Started|intro|Ünïcode Title|100% done|a_b|Σίσυφος|x y|`code` span|FAQ|émoji 🎉|ΟΔΟΣ|Intro
md LINES=# $H|## $H|$H~===|$H~---|<a id="$H"></a>|<a name="$H"></a>|[$L]: $T|   [$L]: $T|[$L]:$T|[$L]: <$T>|[$L]: $T "title"|```~[$L]: $T~```|<!-- [$L]: $T -->|see [x][$L] here|[$L][] and [$L]|text [t]($T)|    [$L]: $T|plain text|> [$L]: $T|- [$L]: $T|[x][$L]|``~# $H~``
md LABELS=Foo|foo|FOO| foo |Foo  Bar|foo bar|ΣΑΣ|σας|ΑΣ.|Σ|İ|ǅ|Straße|x°y|x y|a^b|ΟΔΟΣ ΣΤΗ|A'Σ|label|Label|LABEL|ẞ|Ꮳ
md TARGETS=a.md|./a.md|../b.md|docs/|docs|a.md#intro|b.md#Intro|#frag|#|ftp:x|https:y|mailto:x|//host/x|/abs.md|a%20b.md|a b.md|%2e%2e/a.md|img/logo.png|c.py|..|.||x:y|C:/x|a%23b.md|a#b.md|%C3%A9.md|é.md|%e9.md|%+f|docs/sub/|README.md#x%20y|a.md#|zz.md|index.md#getting-started|a.markdown#intro|x.markdown|a1+.x:y|1a:b|../../..|logo.png|style.css|%ED%A0%80.md|%C0%AF.md|%F4%90%80%80.md|%2Fa.md
md PREFIXES=||||../|docs/|./|../../|img/|site/zh/|%2e/|a/b/
md FRAGS=x-y|getting-started|code-span|intro|intro-1|getting-started|x%20y|x y|Intro|faq|%C3%BCn%C3%AFcode-title|ünïcode-title|code-span|nope||%ZZ|σίσυφος|100-done|a_b|%CE%BF%CE%B4%CE%BF%CF%82|%+f|émoji-|x%2|ab%
md LEGS=md-link 41 0|md-image 42 1|md-reflink 43 2|md-refdef 44 3|md-url 45 4
"#;

/// The table a key names: `<leg> <NAME>`.
pub(super) fn t(key: &str) -> &'static str {
    TABLES
        .lines()
        .find_map(|row| row.split_once('=').filter(|(k, _)| *k == key))
        .map(|(_, words)| words)
        .unwrap_or_else(|| panic!("no table {key}"))
}
