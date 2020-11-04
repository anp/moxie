use codemap::CodeMap;
use color_eyre::eyre::Result;
use starlark::environment::{Environment, TypeValues};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tracing::{error, info, instrument, warn};

mod vfs;

use vfs::Vfs;

pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Tracks changes to files which we've read.
    vfs: Vfs,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf(), vfs: Vfs::default() }
    }

    pub fn maintain(self) -> Result<()> {
        info!("maintaining workspace");
        loop {
            if let Err(error) = self.converge() {
                error!(%error, "couldn't converge current workspace revision");
            }
            warn!("TODO wait for changes to inputs");
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.root.display()))]
    fn converge(&self) -> Result<(), HonkError> {
        info!("reading workspace file");
        let root_contents = self.vfs.read(&self.root)?;
        let root_contents = std::str::from_utf8(&*root_contents)
            .map_err(|source| HonkError::ScriptEncoding { source, file: self.root.clone() })?;

        let map = Arc::new(Mutex::new(CodeMap::new()));
        let types = TypeValues::default();
        let mut env = Environment::new("honk");

        info!("evaluating workspace file");
        starlark::eval::eval(
            &map,
            &self.root.to_string_lossy(),
            &root_contents,
            // TODO figure out if this is the right dialect?
            starlark::syntax::dialect::Dialect::Bzl,
            &mut env,
            &types,
            &self.vfs,
        )
        .map_err(|diagnostic| HonkError::Eval { diagnostic, map: map.clone() })?;

        warn!("TODO run formatters");
        warn!("TODO run build/test");

        Ok(())
    }
}

#[derive(Debug, Error)]
enum HonkError {
    #[error("evaluation error: TODO print it here")]
    Eval { diagnostic: codemap_diagnostic::Diagnostic, map: Arc<Mutex<CodeMap>> },

    #[error("i/o error")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("non utf-8 *.honk script encountered at {}", file.display())]
    ScriptEncoding { source: std::str::Utf8Error, file: PathBuf },
}
