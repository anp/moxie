use crate::builtins::command::Output;
use starlark::values::ValueError;
use std::{path::PathBuf, str::Utf8Error, string::FromUtf8Error};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("starlark error:\n{0}")]
    StarlarkError(#[source] anyhow::Error),

    #[error("failed to run command: {0:#?}")]
    CommandFailed(Output),

    #[error("`{command}` returned non utf-8: {source}")]
    StdoutEncoding { source: FromUtf8Error, command: String },

    #[error("error handling JSON: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },

    #[allow(unused)]
    #[error("non utf-8 *.honk script encountered at {}", file.display())]
    ScriptEncoding { source: Utf8Error, file: PathBuf },
}

impl From<Error> for ValueError {
    fn from(e: Error) -> Self {
        todo!("uh do this conversion properly: {:?}", e);
    }
}
