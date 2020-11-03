use color_eyre::eyre::Result;
use std::path::{Path, PathBuf};
use tracing::{info, instrument, warn};

#[derive(Debug)]
pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    pub fn maintain(self) -> Result<()> {
        info!("maintaining workspace");
        loop {
            self.converge()?;
            warn!("TODO wait for changes to inputs");
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.root.display()))]
    fn converge(&self) -> Result<()> {
        info!("reading workspace file");

        warn!("TODO get workspace file contents");
        warn!("TODO evaluate workspace file");
        warn!("TODO run formatters");
        warn!("TODO run build/test");

        Ok(())
    }
}
