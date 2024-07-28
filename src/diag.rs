use miette::{
    MietteDiagnostic,
    LabeledSpan,
    Severity,
    Report,
    NamedSource
};

use super::source::SourceRange;
use super::lexis::Token;

pub struct Diag<'source> {
    source: &'source str,
    source_name: String
}

impl<'source> Diag<'source> {
    pub fn new(source_name: &str, source: &'source str) -> Self {
        Self {
            source,
            source_name: source_name.into()
        }
    }

    pub fn invalid_token(&self, bad_char: char, bad_char_snap: SourceRange) -> Report {
        let bad_char_code = bad_char as u32;
        let msg = format!("can't parse the next token: unexpected character U+{bad_char_code:04X} was encountered");
        let report: Report = MietteDiagnostic::new(msg)
            .with_label(LabeledSpan::new_with_span(Some("the unexpected character".to_owned()), bad_char_snap))
            .with_severity(Severity::Error).into();
        report.with_source_code(NamedSource::new(self.source_name.clone(), self.source.to_owned()))
    }

    pub fn unexpected_eof(&self, eof_snap: SourceRange) -> Report {
        let report: Report = MietteDiagnostic::new("unexpected end of file")
            .with_label(LabeledSpan::new_with_span(Some("eof".to_owned()), eof_snap))
            .with_severity(Severity::Error).into();
        report.with_source_code(NamedSource::new(self.source_name.clone(), self.source.to_owned()))
    }

    pub fn unexpected_token(&self, token: Token, expected: &str) -> Report {
        let msg = format!("unexpected token `{}` encountered instead of {}", token.spelling(), expected);
        let report: Report = MietteDiagnostic::new(msg)
            .with_label(LabeledSpan::new_with_span(Some("the unexpected token".to_owned()), token.span()))
            .with_severity(Severity::Error).into();
        report.with_source_code(NamedSource::new(self.source_name.clone(), self.source.to_owned()))
    }
    
    pub fn expected_eol(&self, token: Token) -> Report {
        let msg = format!("unexpected token `{}` instead of the end of the current line", token.spelling());
        let report: Report = MietteDiagnostic::new(msg)
            .with_label(LabeledSpan::new_with_span(Some("the unexpected token".to_owned()), token.span()))
            .with_severity(Severity::Error).into();
        report.with_source_code(NamedSource::new(self.source_name.clone(), self.source.to_owned()))
    }
    
    pub fn expected_text(&self, token: Token) -> Report {
        let msg = format!("unexpected token `{}` instead of usual text", token.spelling());
        let report: Report = MietteDiagnostic::new(msg)
            .with_label(LabeledSpan::new_with_span(Some("the unexpected token".to_owned()), token.span()))
            .with_severity(Severity::Error).into();
        report.with_source_code(NamedSource::new(self.source_name.clone(), self.source.to_owned()))
    }
}
