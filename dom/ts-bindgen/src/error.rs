use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io,
};
use swc_ecma_parser::error::Error as SwcError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindingError {
    #[error("reading input failed")]
    ReadInputFile(io::Error),

    #[error("parsing")]
    ParseInputFile { source: ParseError },

    #[error("writing output failed")]
    WriteOutFile(io::Error),
}

impl From<SwcError> for BindingError {
    fn from(e: SwcError) -> Self {
        BindingError::ParseInputFile { source: e.into() }
    }
}

#[derive(Error)]
pub struct ParseError {
    e: SwcError,
}

impl ParseError {
    fn to_stderr(&self) {
        use swc_common::errors::{ColorConfig, EmitterWriter, Handler, HandlerFlags};
        let emitter = EmitterWriter::stderr(
            ColorConfig::Auto,
            None,  // source maps
            false, // short_message
            true,  // teach
        );
        let handler = Handler::with_emitter_and_flags(Box::new(emitter), HandlerFlags {
            can_emit_warnings: true,
            treat_err_as_bug: true,
            dont_buffer_diagnostics: true,
            report_delayed_bugs: false,
            external_macro_backtrace: false, // lol
        });
        self.e.clone().into_diagnostic(&handler).emit();
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_stderr();
        write!(f, "ParseError(see stderr)")
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_stderr();
        write!(f, "Parsing error (see stderr for details)")
    }
}

impl From<SwcError> for ParseError {
    fn from(e: SwcError) -> Self {
        ParseError { e }
    }
}
