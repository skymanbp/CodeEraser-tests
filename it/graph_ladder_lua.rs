//! Lua ladder fixtures (plan v2.30 step 4): `require` asks every search
//! directory — the tree root, `src`, `lua`, the declared
//! `[graph.search_roots] lua`, the requiring file's own directory and
//! each template the tree's own files assign to `package.path` — and
//! `dofile` / `loadfile` read a path beside the loading file, then under
//! the root. The fixture reads the
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
/// defaults and the templates. The requiring file's own directory (plan
/// v2.30 step 5, the second `@cases` run): a plugin's modules beside its
/// main file, a spec helper beside the specs — the directory a loader
/// prepends for the code it loads. It is one more search directory,
/// not a first resort: `util` in the plugin and at the root is two
/// files, ambiguous_root. A file one directory down does not see its
/// parent's modules, and a name only a subdirectory holds is still
/// asked the full path.
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
==== util.lua
return {}
==== plugins/cover.koplugin/main.lua
return {}
==== plugins/cover.koplugin/manager.lua
return {}
==== plugins/cover.koplugin/util.lua
return {}
==== plugins/cover.koplugin/lib/xml.lua
return {}
==== plugins/cover.koplugin/menu/init.lua
return {}
==== spec/unit/helper.lua
return {}
==== spec/unit/a_spec.lua
return {}
==== spec/unit/deep/b_spec.lua
return {}
==== src/app/run.lua
return {}
==== src/app/tool.lua
return {}
==== @cases
require @@ plugins/cover.koplugin/main.lua @@ manager @@ ok plugins/cover.koplugin/manager.lua 1
require @@ plugins/cover.koplugin/main.lua @@ lib.xml @@ ok plugins/cover.koplugin/lib/xml.lua 1
require @@ plugins/cover.koplugin/main.lua @@ lib/xml @@ ok plugins/cover.koplugin/lib/xml.lua 1
require @@ plugins/cover.koplugin/main.lua @@ menu @@ ok plugins/cover.koplugin/menu/init.lua 1
require @@ plugins/cover.koplugin/main.lua @@ util @@ no ambiguous_root
require @@ plugins/cover.koplugin/lib/xml.lua @@ manager @@ no out_of_scope
require @@ spec/unit/a_spec.lua @@ helper @@ ok spec/unit/helper.lua 1
require @@ spec/unit/deep/b_spec.lua @@ helper @@ no out_of_scope
require @@ src/app/run.lua @@ tool @@ ok src/app/tool.lua 1
require @@ src/app/run.lua @@ app.tool @@ ok src/app/tool.lua 1
require @@ util.lua @@ manager @@ no out_of_scope
require @@ util.lua @@ util @@ ok util.lua 1
"#;

#[test]
fn lua_rungs_resolve_and_refuse() {
    text_ladder(Lang::Lua, LADDER);
}
