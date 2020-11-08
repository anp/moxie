use crate::builtins::command::{Command, Output};
use codemap::CodeMap;
use codemap_diagnostic::{ColorConfig, Emitter};
use starlark::values::error::ValueError;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    path::PathBuf,
    str::Utf8Error,
    string::FromUtf8Error,
    sync::{Arc, Mutex},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("evaluation error: {source}")]
    Eval {
        #[from]
        source: EvalError,
    },

    #[error("i/o error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("failed to run command: {0:#?}")]
    CommandFailed(Output),

    #[error("`{command:?}` returned non utf-8: {source}")]
    StdoutEncoding { source: FromUtf8Error, command: Command },

    #[allow(unused)]
    #[error("non utf-8 *.honk script encountered at {}", file.display())]
    ScriptEncoding { source: Utf8Error, file: PathBuf },
}

impl From<Error> for ValueError {
    fn from(e: Error) -> Self {
        todo!("uh do this conversion properly: {:?}", e);
    }
}

#[derive(thiserror::Error)]
pub struct EvalError {
    pub map: Arc<Mutex<CodeMap>>,
    pub diagnostic: codemap_diagnostic::Diagnostic,
}

impl EvalError {
    pub fn from_exception(except: starlark::eval::EvalException, map: Arc<Mutex<CodeMap>>) -> Self {
        Self { diagnostic: except.into(), map }
    }

    fn emit(&self) {
        let map = self.map.lock().unwrap();
        let mut emitter = Emitter::stderr(ColorConfig::Auto, Some(&*map));
        emitter.emit(&[self.diagnostic.clone()]);
    }
}

impl Debug for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.diagnostic.fmt(f)
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.emit();
        f.debug_struct("EvalError").finish()
    }
}
