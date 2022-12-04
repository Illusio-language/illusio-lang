use crate::token::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::Config;

/// Error {source, filename, message, span
#[derive(Clone)]
pub struct Error {
    pub source: String,
    pub file_name: String,
    pub message: String,
    pub span: Span,
    pub help: String,
}
impl Error {
    pub fn show(&self) {
        let mut files = SimpleFiles::new();
        let file_id = files.add(&self.file_name, &self.source);
        let help = if self.help == "" {
            vec![]
        } else {
            vec![self.help.clone()]
        };
        let diagnostic_err = Diagnostic::error()
            .with_message(&self.message)
            .with_labels(vec![Label::primary(file_id, self.span)])
            .with_notes(help);
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = Config::default();
        codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic_err)
            .unwrap();
    }
}
