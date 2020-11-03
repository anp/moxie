use color_eyre::eyre::Result;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug)]
pub struct Workspace {
    /// Root directory of the workspace.
    root: PathBuf,
}

impl Workspace {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        info!(root = %root.display(), "Opening.");
        Ok(Self { root })
    }

    pub fn run(self) -> Result<()> {
        info!(root = %self.root.display(), "Running.");
        Ok(())
    }
}
