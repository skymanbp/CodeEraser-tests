//! The annotation half of the Java header lexer as it stood at 27d0d56d
//! (cli/src/graph/ladder/java_header.rs and java_types.rs), the last
//! commit before stage C of v2.33 W2-text moved the Java rungs into the
//! core: `past_annotation` and the Lexer methods it reaches, and
//! java_types.rs's `ident_char` - copied line for line; the header and
//! type readers themselves stay in cli/src (the walk reads them). The
//! frozen java_pick.rs's `unannotated` reads it (oracle/java_pick.rs,
//! its one edit: the call path).

/// The text after an annotation whose `@` is already read — its dotted
/// name and any argument list: the Java ladder drops a type annotation
/// from the name it annotates (ladder/java.rs).
pub(super) fn past_annotation(text: &str) -> &str {
    let mut lex = Lexer(text);
    lex.annotation();
    lex.0
}

/// The unread rest of the text; every method skips the trivia before
/// the token it reads.
struct Lexer<'t>(&'t str);

impl<'t> Lexer<'t> {
    /// Whitespace and comments (an unterminated block comment runs to
    /// the end).
    fn trivia(&mut self) {
        loop {
            self.0 = self.0.trim_start();
            if let Some(rest) = self.0.strip_prefix("//") {
                self.0 = rest.split_once('\n').map_or("", |(_, tail)| tail);
            } else if let Some(rest) = self.0.strip_prefix("/*") {
                self.0 = rest.split_once("*/").map_or("", |(_, tail)| tail);
            } else {
                return;
            }
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.trivia();
        match self.0.strip_prefix(c) {
            Some(rest) => {
                self.0 = rest;
                true
            }
            None => false,
        }
    }

    fn ident(&mut self) -> Option<&'t str> {
        self.trivia();
        let end = self
            .0
            .find(|c: char| !ident_char(c))
            .unwrap_or(self.0.len());
        let (word, rest) = self.0.split_at(end);
        if word.is_empty() || word.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        self.0 = rest;
        Some(word)
    }

    /// A dotted name, and whether it ended `.*`.
    fn name(&mut self) -> Option<(String, bool)> {
        let mut name = self.ident()?.to_string();
        while self.eat('.') {
            if self.eat('*') {
                return Some((name, true));
            }
            name.push('.');
            name.push_str(self.ident()?);
        }
        Some((name, false))
    }

    /// An annotation past its `@`: the (dotted) name, then an argument
    /// list if one follows.
    fn annotation(&mut self) {
        if self.name().is_some() && self.eat('(') {
            self.group();
        }
    }

    /// Past a `(` already eaten: to its matching `)`, string, text
    /// block and char literals and comments read as opaque.
    fn group(&mut self) {
        let mut depth = 1usize;
        while depth > 0 {
            self.trivia();
            let mut chars = self.0.chars();
            let Some(c) = chars.next() else {
                return;
            };
            self.0 = chars.as_str();
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                '"' | '\'' => self.literal(c),
                _ => {}
            }
        }
    }

    /// Past the opening quote of a literal: to its closing one, escapes
    /// honoured; `"""` opens a text block, closed by the next `"""`.
    fn literal(&mut self, quote: char) {
        if quote == '"'
            && let Some(rest) = self.0.strip_prefix("\"\"")
        {
            self.0 = rest.split_once("\"\"\"").map_or("", |(_, tail)| tail);
            return;
        }
        let mut chars = self.0.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                chars.next();
            } else if c == quote {
                break;
            }
        }
        self.0 = chars.as_str();
    }
}

/// A Java identifier character (JLS 3.8: letters and digits of any
/// script, `_` and `$`) — the header lexer imports it from here so the
/// edge between the two files runs one way.
pub(super) fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}
