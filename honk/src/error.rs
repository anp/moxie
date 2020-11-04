use std::{path::PathBuf, str::Utf8Error};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("evaluation error")]
    Eval,

    #[error("i/o error")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("non utf-8 *.honk script encountered at {}", file.display())]
    ScriptEncoding { source: Utf8Error, file: PathBuf },
}
