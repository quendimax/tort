use logos::{self, Logos};
use miette::Result;

use crate::diag::Diag;
use crate::source::SourceRange;


#[allow(non_camel_case_types)]
#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    #[token("[", priority = 3)]
    l_square,

    #[token("]", priority = 3)]
    r_square,

    #[token("->")]
    arrow,

    #[token(":", priority = 3)]
    colon,

    #[token("|", priority = 3)]
    pipe,

    #[regex(r"#[^\n\r]*", priority=5)]
    comment,

    #[regex(r"#![^\n\r]*", priority=7)]
    pub_comment,

    #[regex(r"[\pL\pM]+")]
    word,

    #[regex(r"[\pP--\[\]:]+")]
    punct,

    #[regex(r"[\pN]+")]
    number,

    #[regex(r"[\pS--|]+")]
    other,

    #[regex(r"[\t\pZ]+")]
    space,

    #[regex(r"\r|\n|\r\n")]
    newline,

    /// End of file
    eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            tok::arrow => "->",
            tok::colon => ":",
            tok::comment => "<COMMENT>",
            tok::eof => "<EOF>",
            tok::l_square => "[",
            tok::r_square => "]",
            tok::newline => "<LF>",
            tok::number => "<NUM>",
            tok::other | &tok::word => "<WORD>",
            tok::punct => "<PUNCT>",
            tok::pipe => "|",
            tok::pub_comment => "<PUB-COMMENT>",
            tok::space => "<WHITESPACE>"
        };
        write!(f, "{}", msg)
    }
}

pub mod tok {
    pub use super::TokenKind::*;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    kind: TokenKind,
    span: SourceRange,
    spelling: String
}

impl Token {
    fn new(tok_kind: TokenKind, span: SourceRange, slice: &str) -> Self {
        Token {
            kind: tok_kind,
            span,
            spelling: match tok_kind {
                tok::word | tok::punct | tok::number | tok::other | tok::newline |
                tok::l_square | tok::r_square | tok::arrow | tok::colon | tok::pipe => slice,
                tok::comment => slice[1..].trim(),  // skip first #
                tok::pub_comment => slice[2..].trim(),  // skip first #!
                tok::space => " ",
                tok::eof => ""
            }.into()
        }
    }

    /// Create a new token of [`tok::eof`] (end of file) kind.
    pub fn eof() -> Self {
        Token {
            kind: tok::eof,
            span: Default::default(),
            spelling: Default::default()
        }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn span(&self) -> SourceRange {
        self.span.clone()
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub fn is_text(&self) -> bool {
        match self.kind() {
            tok::word | tok::punct | tok::number | tok::other |
            tok::colon | tok::pipe | tok::space => true,
            _ => false
        }
    }

    pub fn is_strict_text(&self) -> bool {
        match self.kind() {
            tok::word | tok::punct | tok::number | tok::other | tok::space => true,
            _ => false
        }
    }

    pub fn is_comment(&self) -> bool {
        match self.kind() {
            tok::comment | tok::pub_comment => true,
            _ => false
        }
    }

    pub fn is_eol(&self) -> bool {
        match self.kind() {
            tok::eof | tok::newline => true,
            _ => false
        }
    }
}

pub struct Lexer<'source> {
    lexer: logos::Lexer<'source, TokenKind>,
    diag: Diag<'source>,
    skip_comments: bool,
}

impl<'source> Lexer<'source> {
    pub fn new(source_name: &str, source: &'source str) -> Self {
        Self {
            lexer: TokenKind::lexer(source),
            diag: Diag::new(source_name, source),
            skip_comments: true,
        }
    }

    pub fn skip_comments(&mut self, enable: bool) {
        self.skip_comments = enable;
    }

    pub fn lex(&mut self) -> Result<Token> {
        let mut error_has_happened = false;

        for res in &mut self.lexer {
            match res {
                Ok(kind) => {
                    if kind == tok::comment && self.skip_comments {
                        continue;
                    }
                    return Ok(Token::new(kind, self.lexer.span(), self.lexer.slice()));
                }
                Err(()) => {
                    error_has_happened = true;
                    break;
                }
            };
        }

        if error_has_happened {
            // trying to get utf8 sequece
            for i in 0..3 {
                let span = SourceRange {
                    start: self.lexer.span().start,
                    end: self.lexer.span().end + i
                };
                if let Some(s) = self.lexer.source().get(span.clone()) {
                    if let Some(c) = s.chars().next() {
                        return Err(self.diag.invalid_token(c, span));
                    }
                }
            }
            unreachable!()
        }

        Ok(Token::eof())
    }

    pub fn expect(&mut self, expected_kind: TokenKind) -> Result<Token> {
        let token = self.lex()?;
        if token.kind() == expected_kind {
            Ok(token)
        } else {
            Err(self.diag.unexpected_token(token, &format!("`{}`", expected_kind)))
        }
    }

    /// Consume token of kind `tok::newline` or `tok::eof`, else return error.
    pub fn expect_eol(&mut self) -> Result<()> {
        let token = self.lex()?;
        if token.kind() == tok::newline || token.kind() == tok::eof {
            Ok(())
        } else {
            Err(self.diag.expected_eol(token))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn lex_all_tokens() {
        let source = " [ ] \t | : -> | \n# comment\r\n\u{2002}#! icomment\nhello! 2+4";
        let mut lexer = TokenKind::lexer(source);
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::l_square)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::r_square)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::pipe)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::colon)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::arrow)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::pipe)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::newline)));
        assert_eq!(lexer.next(), Some(Ok(tok::comment)));
        assert_eq!(lexer.next(), Some(Ok(tok::newline)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::pub_comment)));
        assert_eq!(lexer.next(), Some(Ok(tok::newline)));
        assert_eq!(lexer.next(), Some(Ok(tok::word)));
        assert_eq!(lexer.next(), Some(Ok(tok::punct)));
        assert_eq!(lexer.next(), Some(Ok(tok::space)));
        assert_eq!(lexer.next(), Some(Ok(tok::number)));
        assert_eq!(lexer.next(), Some(Ok(tok::other)));
        assert_eq!(lexer.next(), Some(Ok(tok::number)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn skip_comments() {
        let source = "#sdf\n#!asdf\n#!\n#\n#";
        let mut lexer = Lexer::new("test", &source);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::pub_comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::pub_comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::eof);
    }

    #[test]
    fn do_not_skip_comments() {
        let source = "#sdf\n#!asdf\n#!\n#\n#";
        let mut lexer = Lexer::new("test", source);
        lexer.skip_comments(false);
        assert_eq!(lexer.lex().unwrap().kind(), tok::comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::pub_comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::pub_comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::comment);
        assert_eq!(lexer.lex().unwrap().kind(), tok::newline);
        assert_eq!(lexer.lex().unwrap().kind(), tok::comment);
        assert_eq!(lexer.lex().unwrap(), Token::eof());
    }

    #[test]
    fn parse_controls() {
        let source = "\0";
        let mut lexer = Lexer::new("test", source);
        let diag = Diag::new("test", source);
        assert_eq!(format!("{:?}", lexer.lex().unwrap_err()),
                   format!("{:?}", diag.invalid_token('\u{0000}', 0..1)));

        let source = "\u{0001}";
        let mut lexer = Lexer::new("test", source);
        let diag = Diag::new("test", source);
        assert_eq!(format!("{:?}", lexer.lex().unwrap_err()),
                   format!("{:?}", diag.invalid_token('\u{0001}', 0..1)));

        let source = "hello\u{0001}";
        let mut lexer = Lexer::new("test", source);
        let diag = Diag::new("test", source);
        assert_eq!(lexer.lex().unwrap().kind(), tok::word);
        assert_eq!(format!("{:?}", lexer.lex().unwrap_err()),
                   format!("{:?}", diag.invalid_token('\u{0001}', 5..6)));
    }
}
