//! Source files for the bags/1 differential's sources leg: six languages,
//! each unit named, documented, calling and carrying literals drawn by
//! bags_gen.rs. A draw that breaks a grammar is kept — both sides parse
//! the same bytes with the same tree, so an ERROR node is a case too.

use super::draw::{Draw, doc, ident};
use crate::scan::lang::Lang;

/// The languages the leg writes, by turns.
pub(super) const LANGS: [Lang; 6] = {
    use Lang::*;
    [Rust, Python, TypeScript, Go, Java, C]
};

const LITERALS: &str = "1 2.5 \"s\" 'c' true false 0x1F None nil null";

/// Each language's unit (its placeholders filled by `Parts::fill`) and
/// how one parameter is typed (`{}` = its name).
fn template(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Rust => (
            "/// {doc}\nfn {name}({params}){ret} { {callee}({lit}); x.{member}(|y| y); {lit} }\n",
            "{}: u32",
        ),
        Lang::Python => (
            "def {name}({params}):\n    \"\"\"{doc}\"\"\"\n    {callee}({lit})\n    return self.{member}()\n\nclass {other}:\n    \"\"\"{doc}\"\"\"\n    def {member}(self):\n        return {lit}\n\n",
            "{}",
        ),
        Lang::TypeScript => (
            "/** {doc} */\nfunction {name}({params}): number { {callee}({lit}); return a.{member}(); }\nconst {member} = (x: number) => {callee}(x);\n",
            "{}: number",
        ),
        Lang::Go => (
            "// {doc}\nfunc {name}({params}) int { {callee}({lit}); return r.{member}() }\n\nfunc (r *{other}) {member}() bool { return true }\n\n",
            "{} int",
        ),
        Lang::Java => (
            "class {other} {\n  /** {doc} */\n  int {name}({params}) { {callee}({lit}); return this.{member}(); }\n  void {member}() {}\n}\n",
            "int {}",
        ),
        _ => (
            "/* {doc} */\nint {name}({params}) { {callee}({lit}); return s->{member}(1); }\n",
            "int {}",
        ),
    }
}

/// One unit's drawn spellings: name, doc line, callee, member, literal,
/// another type name, and the parameter names.
struct Parts {
    name: String,
    doc: String,
    callee: String,
    member: String,
    lit: String,
    other: String,
    params: Vec<String>,
}

impl Parts {
    fn draw(d: &mut Draw) -> Parts {
        Parts {
            name: ident(d),
            doc: doc(d),
            callee: ident(d),
            member: ident(d),
            lit: d.one(LITERALS).to_string(),
            other: ident(d),
            params: (0..d.below(4)).map(|i| format!("p{i}")).collect(),
        }
    }

    /// A template with every placeholder spelled, the parameters each
    /// typed by `ty` (`{}` = the name). The doc line goes in last: drawn
    /// identifiers and literals hold no braces, so no filled text is read
    /// again as a placeholder.
    fn fill(&self, template: &str, ty: &str) -> String {
        let typed: Vec<String> = self.params.iter().map(|p| ty.replace("{}", p)).collect();
        [
            ("{params}", typed.join(", ")),
            ("{other}", self.other.clone()),
            ("{name}", self.name.clone()),
            ("{callee}", self.callee.clone()),
            ("{member}", self.member.clone()),
            ("{lit}", self.lit.clone()),
            ("{doc}", self.doc.clone()),
        ]
        .iter()
        .fold(template.to_string(), |t, (k, v)| t.replace(k, v))
    }
}

/// A file of `units` units in `lang`.
pub(super) fn source(d: &mut Draw, lang: Lang, units: usize) -> String {
    (0..units)
        .map(|_| {
            let p = Parts::draw(d);
            match lang {
                Lang::Rust => rust(d, &p),
                _ => {
                    let (unit, ty) = template(lang);
                    p.fill(unit, ty)
                }
            }
        })
        .collect()
}

/// A documented fn, sometimes in an impl or an `impl T for U`.
fn rust(d: &mut Draw, p: &Parts) -> String {
    let ret = if d.below(2) == 0 { " -> u32" } else { "" };
    let (unit, ty) = template(Lang::Rust);
    let body = p.fill(&unit.replace("{ret}", ret), ty);
    match d.below(3) {
        0 => format!("impl {} {{\n{body}}}\n", p.other),
        1 => format!("impl {} for {} {{\n{body}}}\n", p.other, p.member),
        _ => body,
    }
}
