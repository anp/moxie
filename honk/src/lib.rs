use color_eyre::eyre::Result;
use std::path::{Path, PathBuf};
use tracing::{error, info, instrument, warn};

mod error;
mod vfs;

use error::Error;
use vfs::VfsLoader;

pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Services starlark calls to `load()` and tracks changes to files we've
    /// read.
    loader: VfsLoader,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf(), loader: Default::default() }
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
    fn converge(&self) -> Result<(), Error> {
        let mut env = self.loader.load_workspace_env(&self.root)?;

        warn!("TODO display discovered targets");

        Ok(())
    }
}
