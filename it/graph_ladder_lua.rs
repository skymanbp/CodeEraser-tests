//! Lua ladder fixtures (plan v2.30 step 4): `require` asks every search
//! directory — the tree root, `src`, `lua`, the declared
//! `[graph.search_roots] lua` and each template the tree's own files
//! assign to `package.path` — and `dofile` / `loadfile` read a path
//! beside the loading file, then under the root. The fixture reads the
//! templates its files write the way the walk does. Ambiguity rows MUST
//! refuse: a module two directories hold that resolves is the red
//! condition.

use codeeraser::scan::lang::Lang;

use crate::common::text_ladder;

/// The habitat, `==== path` over each file, then the rows
/// (common::text_ladder). main.lua writes two templates into
/// package.path; loader.lua builds one at run time, which states no
/// template; `twin` sits in two default directories and `both` in one
/// directory in both forms. Under `[graph.search_roots] lua =
/// ["vendor"]` (the `@rooted` rows) the declared directory joins the
/// defaults and the templates.
const LADDER: &str = r#"
==== main.lua
package.path = "app/?.lua;app/?/init.lua;" .. package.path
==== loader.lua
package.path = string.format("%s/?.lua;%s", "hidden", package.path)
==== app/mod.lua
return {}
==== app/ui/widget.lua
return {}
==== app/pkg/init.lua
return {}
==== src/luarocks/core/cfg.lua
return {}
==== lua/plugin/init.lua
return {}
==== twin.lua
return {}
==== src/twin.lua
return {}
==== both.lua
return {}
==== both/init.lua
return {}
==== hidden/secret.lua
return {}
==== vendor/dep.lua
return {}
==== tools/run.lua
dofile("helper.lua")
==== tools/helper.lua
return {}
==== data/conf.lua
return {}
==== @cases
require @@ main.lua @@ mod @@ ok app/mod.lua 1
require @@ main.lua @@ ui/widget @@ ok app/ui/widget.lua 1
require @@ main.lua @@ ui.widget @@ ok app/ui/widget.lua 1
require @@ main.lua @@ pkg @@ ok app/pkg/init.lua 1
require @@ main.lua @@ luarocks.core.cfg @@ ok src/luarocks/core/cfg.lua 1
require @@ main.lua @@ plugin @@ ok lua/plugin/init.lua 1
require @@ main.lua @@ both @@ ok both.lua 1
require @@ main.lua @@ twin @@ no ambiguous_root
require @@ main.lua @@ secret @@ no out_of_scope
require @@ main.lua @@ dep @@ no out_of_scope
require @@ main.lua @@ socket.url @@ no out_of_scope
require @@ main.lua @@ a..b @@ no out_of_scope
require @@ main.lua @@ string @@ ext 3
require @@ main.lua @@ ffi @@ ext 3
require @@ main.lua @@ jit.util @@ ext 3
load @@ tools/run.lua @@ helper.lua @@ ok tools/helper.lua 2
load @@ tools/run.lua @@ data/conf.lua @@ ok data/conf.lua 2
load @@ tools/run.lua @@ ../data/conf.lua @@ ok data/conf.lua 2
load @@ tools/run.lua @@ /etc/conf.lua @@ no out_of_scope
load @@ tools/run.lua @@ missing.lua @@ no out_of_scope
==== @rooted vendor
require @@ main.lua @@ dep @@ ok vendor/dep.lua 1
require @@ main.lua @@ mod @@ ok app/mod.lua 1
require @@ main.lua @@ twin @@ no ambiguous_root
"#;

#[test]
fn lua_rungs_resolve_and_refuse() {
    text_ladder(Lang::Lua, LADDER);
}
