//! The one ruled divergence between the core's Haskell rungs and the
//! frozen ones (oracle/hs.rs, a byte copy from before the change). Since
//! release 1.10.0 the second rung lets the importer's OWN package answer:
//! cabal's rule is that a component listing its own package in
//! build-depends depends on that package's library and sees its exposed
//! modules, and an import naming its own package under PackageImports
//! names that library too. The frozen rung still leaves the owner out. The
//! live side is the ruled one, so the random legs allow a mismatch on a
//! site carrying the mark below and on no other site, and fail when no
//! marked site mismatches any more (the note would be stale).

use super::common::{World, write_tree};
use crate::graph::cabal::{self, Cabal};
use crate::graph::ladder::{self, Outcome, Reason, Scope, Site};
use crate::scan::lang::Lang;

/// The mark, computed from the frozen cabal reader (never the core): the
/// importer's owning cabal — the deepest directory prefix, one at that
/// depth — has a name, exposes the imported module, and is a package the
/// import may draw from by the new rule: the import names it under
/// PackageImports, or names no package and the owner's build-depends
/// lists the owner's own name.
pub(super) fn own_package_answers(site: &Site, scope: &Scope) -> bool {
    let (package, module) = match site.spec.strip_prefix('"').and_then(|r| r.split_once('"')) {
        Some((p, m)) => (Some(p), m.trim()),
        None => (None, site.spec),
    };
    let Some(owner) = owner(site.from, scope) else {
        return false;
    };
    let draws = package.map_or(owner.deps.contains(&owner.name), |p| p == owner.name);
    !owner.name.is_empty() && owner.exposes(module) && draws
}

/// The importer's one owning cabal (none, or two at one depth: no mark).
fn owner(from: &str, scope: &Scope) -> Option<Cabal> {
    let holders: Vec<Cabal> = scope
        .configs
        .iter()
        .filter(|c| c.ends_with(".cabal"))
        .filter_map(|rel| cabal::parse(scope.root, rel))
        .filter(|c| c.dir.is_empty() || from.starts_with(&format!("{}/", c.dir)))
        .collect();
    let deepest = holders.iter().map(|c| c.dir.len()).max()?;
    let mut at: Vec<Cabal> = holders
        .into_iter()
        .filter(|c| c.dir.len() == deepest)
        .collect();
    (at.len() == 1).then(|| at.remove(0))
}

/// The live answer, the frozen answer and the mark for one site over the
/// tree `files` (the cabals among them written as `texts`).
fn answers(name: &str, texts: &[(&str, &str)], files: &[&str], spec: &str) -> [String; 3] {
    let root = super::common::scratch().join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("scratch dir");
    write_tree(&root, texts);
    let world = World {
        files: files.iter().map(|f| (*f).to_string()).collect(),
        configs: texts.iter().map(|(rel, _)| (*rel).to_string()).collect(),
        ..World::default()
    };
    let site = Site {
        kind: "import",
        from: "pkg/app/Main.hs",
        spec,
        line: 1,
    };
    world.with_scopes(&root, |core, oracle| {
        let live = ladder::resolve_all(&[(Lang::Haskell, &site)], core).expect("the core answers");
        let frozen = ladder::frozen::resolve(Lang::Haskell, &site, oracle);
        let mark = own_package_answers(&site, oracle);
        [
            format!("{:?}", live[0]),
            format!("{frozen:?}"),
            mark.to_string(),
        ]
    })
}

fn resolved(path: &str) -> String {
    format!(
        "{:?}",
        Outcome::Resolved {
            path: path.into(),
            rung: 2
        }
    )
}

fn refused(reason: Reason) -> String {
    format!("{:?}", Outcome::Unresolved(reason))
}

const LIBRARY: &str = "library\n  hs-source-dirs: src\n  exposed-modules: A\n";
const APP: &str = "executable app\n  hs-source-dirs: app\n  main-is: Main.hs\n";

/// The divergence on fixed trees, on every push (the lib legs run with
/// the core exported, as every resolve leg does): the own package named
/// under PackageImports answers live and nowhere frozen; the own package
/// in its own build-depends beside another package exposing the same
/// module makes two packages live (ambiguous) and one frozen; without the
/// self-dependency, no mark and both sides agree.
/// One fixed tree's three answers against `want`, `what` naming the case.
fn pinned(name: &str, texts: &[(&str, &str)], spec: &str, want: [String; 3], what: &str) {
    let files = ["pkg/src/A.hs", "pkg/app/Main.hs", "other/lib/A.hs"];
    assert_eq!(answers(name, texts, &files, spec), want, "{what}");
}

#[test]
fn the_own_package_divergence_is_pinned() {
    let other = "name: other\nlibrary\n  hs-source-dirs: lib\n  exposed-modules: A\n";
    let named = format!("name: pkg\n{LIBRARY}{APP}");
    let (live, frozen) = (resolved("pkg/src/A.hs"), refused(Reason::OutOfScope));
    let own = [("pkg/pkg.cabal", named.as_str())];
    pinned(
        "hs-own-named",
        &own,
        "\"pkg\" A",
        [live, frozen, "true".into()],
        "named",
    );
    let selfdep = format!("name: pkg\n{LIBRARY}{APP}  build-depends: base, pkg, other\n");
    let both = [
        ("pkg/pkg.cabal", selfdep.as_str()),
        ("other/other.cabal", other),
    ];
    let two = refused(Reason::AmbiguousWorkspace);
    pinned(
        "hs-own-depends",
        &both,
        "A",
        [two, resolved("other/lib/A.hs"), "true".into()],
        "self-dependency",
    );
    let plain = format!("name: pkg\n{LIBRARY}{APP}  build-depends: base, other\n");
    let neither = [
        ("pkg/pkg.cabal", plain.as_str()),
        ("other/other.cabal", other),
    ];
    let agree = resolved("other/lib/A.hs");
    pinned(
        "hs-own-plain",
        &neither,
        "A",
        [agree.clone(), agree, "false".into()],
        "no mark: both agree",
    );
}
