use miette::Result;

use crate::diag::Diag;
use crate::lexis::{tok, Lexer, Token};

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    cur_line: Vec<Lexeme>,
    diag: Diag<'source>
}

impl<'source> Parser<'source> {
    pub fn new(source_name: &str, source: &'source str) -> Self {
        let mut lexer = Lexer::new(source_name, source);
        lexer.skip_comments(true);
        Parser {
            lexer,
            cur_line: Vec::new(),
            diag: Diag::new(source_name, source)
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Line>> {
        let mut lines: Vec<Line> = vec![];
        loop {
            if let Some(line) = self.parse_line()? {
                lines.push(line);
            } else {
                break;
            }
        }
        Ok(lines)
    }

    fn parse_line(&mut self) -> Result<Option<Line>> {
        self.cur_line.clear();
        let token = self.lexer.lex()?;
        match token.kind() {
            tok::pub_comment => {
                if token.span().start == 0 {
                    self.lexer.expect(tok::newline)?;
                    Ok(Some(Line::PubComment(token)))
                } else {
                    self.lexer.expect_eol()?;
                    Ok(Some(Line::PubComment(token)))
                }
            },
            tok::word | tok::punct | tok::number | tok::other | tok::colon |
            tok::pipe | tok::l_square => {
                let stmt = self.parse_stmt(token)?;
                Ok(Some(stmt))
            },
            tok::arrow | tok::r_square => Err(self.diag.expected_text(token)),
            tok::newline | tok::comment | tok::space => Ok(Some(Line::Empty)),
            tok::eof => Ok(None),
        }
    }

    fn parse_stmt(&mut self, first_token: Token) -> Result<Line> {
        match first_token.kind() {
            tok::l_square => return self.parse_complex_stmt(),
            _ => self.cur_line.push(Lexeme::Normal(first_token))
        }

        loop {
            let token = self.lexer.lex()?;
            match token.kind() {
                _ if token.is_text() => self.cur_line.push(Lexeme::Normal(token.clone())),
                _ if token.is_comment() | token.is_eol() => return self.parse_plain_stmt(token),
                tok::arrow => return self.parse_translation_stmt(),
                tok::l_square => return self.parse_complex_stmt(),
                _ => return Err(self.diag.expected_text(token))
            }
        }
    }
    
    fn parse_plain_stmt(&mut self, last_token: Token) -> Result<Line> {
        let text = aid::lexemes_to_text(&self.cur_line);
        self.cur_line.clear();
        match last_token.kind() {
            tok::pub_comment => Ok(Line::PlainStmt { text, comment: Some(last_token) }),
            tok::comment | tok::newline | tok::eof => Ok(Line::PlainStmt { text, comment: None }),
            _ => unreachable!()
        }
    }

    fn parse_translation_stmt(&mut self) -> Result<Line> {
        let original: Text = aid::lexemes_to_text(&self.cur_line);
        self.cur_line.clear();
        loop {
            let token = self.lexer.lex()?;
            if token.is_text() {
                self.cur_line.push(Lexeme::Normal(token));
            } else if token.kind() == tok::pub_comment {
                let translation: Text = aid::lexemes_to_text(&self.cur_line);
                self.cur_line.clear();
                self.lexer.expect_eol()?;
                return Ok(Line::TranslationStmt { original, translation, comment: Some(token) })
            } else if token.is_eol() {
                let translation: Text = aid::lexemes_to_text(&self.cur_line);
                self.cur_line.clear();
                return Ok(Line::TranslationStmt { original, translation, comment: None })
            } else {
                return Err(self.diag.expected_text(token));
            }
        }
    }

    fn parse_complex_stmt(&mut self) -> Result<Line> {
        let ortho = self.parse_orthogram()?;
        self.cur_line.push(Lexeme::Orthogram(ortho));
        loop {
            let token = self.lexer.lex()?;
            if token.is_text() {
                self.cur_line.push(Lexeme::Normal(token));
            } else if token.kind() == tok::pub_comment {
                self.lexer.expect_eol()?;
                let text: Vec<Lexeme> = aid::strip(self.cur_line.drain(0..self.cur_line.len()).collect());
                return Ok(Line::ComplexStmt { text, comment: Some(token) });
            } else if token.is_eol() {
                let text: Vec<Lexeme> = aid::strip(self.cur_line.drain(0..self.cur_line.len()).collect());
                return Ok(Line::ComplexStmt { text, comment: None });
            } else if token.kind() == tok::l_square {
                let ortho = self.parse_orthogram()?;
                self.cur_line.push(Lexeme::Orthogram(ortho));
            } else {
                return Err(self.diag.expected_text(token));
            }
        }
    }

    fn parse_orthogram(&mut self) -> Result<Orthogram> {
        let mut answer = Vec::new();
        loop {
            let token = self.lexer.lex()?;
            if token.is_strict_text() {
                answer.push(token);
            } else if token.kind() == tok::colon {
                let mut comment = Vec::new();
                let mut token = self.lexer.lex()?;
                loop {
                    if token.is_text() {
                        comment.push(token);
                    } else {
                        break;
                    }
                    token = self.lexer.lex()?;
                }
                if token.kind() != tok::r_square {
                    return Err(self.diag.unexpected_token(token, "`]`"));
                }
                return Ok(Orthogram::Gap { answer, comment: Some(comment) });
            } else if token.kind() == tok::r_square {
                return Ok(Orthogram::Gap { answer, comment: None });
            } else if token.kind() == tok::pipe {
                return self.parse_choice_orthogram(answer);
            } else {
                return Err(self.diag.expected_text(token));
            }
        }
    }

    fn parse_choice_orthogram(&mut self, right_answer: Text) -> Result<Orthogram> {
        let mut wrong_answers = Vec::new();
        loop {
            let mut wrong_answer = Vec::new();
            loop {
                let token = self.lexer.lex()?;
                if token.is_strict_text() {
                    wrong_answer.push(token);
                } else if token.kind() == tok::pipe {
                    wrong_answers.push(wrong_answer);
                    break;
                } else if token.kind() == tok::r_square {
                    wrong_answers.push(wrong_answer);
                    return Ok(Orthogram::Choice { right_answer, wrong_answers });
                } else {
                    return Err(self.diag.unexpected_token(token, "`|` or `]`"));
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Line {
    PubComment(Token),
    PlainStmt {
        text: Text,
        comment: Option<Token>,
    },
    TranslationStmt {
        original: Text,
        translation: Text,
        comment: Option<Token>,
    },
    ComplexStmt {
        text: Vec<Lexeme>,
        comment: Option<Token>,
    },
    Empty
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    Normal(Token),
    Orthogram(Orthogram),
}

impl Lexeme {
    pub fn unwrap_norm(&self) -> &Token {
        if let Self::Normal(tok) = self {
            return tok;
        }
        panic!("you're trying to unwrap_norm the Lexeme::Orthogram instance")
    }

    pub fn unwrap_orthogram(&self) -> &Orthogram {
        if let Self::Orthogram(ortho) = self {
            return ortho;
        }
        panic!("you're trying to unwrap_ortho the Lexeme::Normal instance")
    }
}

pub type Text = Vec<Token>;

#[derive(Debug, PartialEq, Clone)]
pub enum Orthogram {
    Gap {
        answer: Text,
        comment: Option<Text>,
    },
    Choice {
        right_answer: Text,
        wrong_answers: Vec<Text>,
    }
}

pub(super) mod aid {
    use super::*;

    pub fn lexemes_to_text(lexemes: &Vec<Lexeme>) -> Text {
        let mut text = Vec::new();
        let lexemes = aid::strip(lexemes.clone());
        for lexeme in lexemes {
            if let Lexeme::Normal(token) = lexeme {
                text.push(token.clone());
            } else {
                panic!("lexemes_to_text function encountered an Orthogram instance");
            }
        }
        text
    }
    
    pub fn strip(lexemes: Vec<Lexeme>) -> Vec<Lexeme> {
        let mut beginning_spaces = 0;
        for lexeme in &lexemes {
            if let Lexeme::Normal(token) = lexeme {
                if token.kind() == tok::space {
                    beginning_spaces += 1;
                } else {
                    break;
                }
            }
        }
        let mut ending_spaces = lexemes.len();
        for lexeme in lexemes.iter().rev() {
            if let Lexeme::Normal(token) = lexeme {
                if token.kind() == tok::space {
                    ending_spaces -= 1;
                } else {
                    break;
                }
            }
        }
        lexemes[beginning_spaces..ending_spaces].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_comment() {
        let mut parser = Parser::new("test", "# hello\n#!");
        assert_eq!(parser.parse_line().unwrap(), Some(Line::Empty));
        if let Line::PubComment(token) = parser.parse_line().unwrap().unwrap() {
            assert_eq!(token.kind(), tok::pub_comment);
        } else {
            panic!("expected a public comment");
        }
        assert_eq!(parser.parse_line().unwrap(), None);
    }

    #[test]
    fn parse_empty_str() {
        let mut parser = Parser::new("test", "");
        assert_eq!(parser.parse_line().unwrap(), None);
    }

    #[test]
    fn parse_skipped_elements() {
        let mut parser = Parser::new("test", "\n\r\n\r####");
        assert_eq!(parser.parse_line().unwrap(), Some(Line::Empty));
        assert_eq!(parser.parse_line().unwrap(), Some(Line::Empty));
        assert_eq!(parser.parse_line().unwrap(), Some(Line::Empty));
        assert_eq!(parser.parse_line().unwrap(), None);
    }

    #[test]
    #[ignore]
    fn parse_arrow_in_the_beginning() {
        let mut parser = Parser::new("test", "\t-> ");
        assert!(parser.parse_line().is_err());
    }

    #[test]
    fn parse_translation() {
        let mut parser = Parser::new("test", "hello -> world\n");
        let line = parser.parse_line().unwrap().unwrap();
        if let Line::TranslationStmt { original, translation, comment } = line {
            assert_eq!(original.len(), 1);
            assert_eq!(translation.len(), 1);
            assert_eq!(comment, None);
        }
    }

    #[test]
    fn parse_gap_orthogram() {
        let mut parser = Parser::new("test", "[a] hello [b] world [c:comment]");
        let _ = parser.parse_line().unwrap().unwrap();
    }

    #[test]
    fn parse_choice_orthogram() {
        let mut parser = Parser::new("test", "[a|b] hello [b|c|d] world [c:comment]");
        let _ = parser.parse_line().unwrap().unwrap();
    }

    #[test]
    fn parse() {
        let mut parser = Parser::new("test", "#\n#!\n[a|b] hello [c:comment]\nhello->world");
        let lines = parser.parse().unwrap();
        assert_eq!(lines.len(), 3);
    }
    
    #[test]
    fn fix_001() {
        let mut parser = Parser::new("test", "[Б|б]онч-[Б|б]руевіч");
        let _ = parser.parse().unwrap();
    }
}
