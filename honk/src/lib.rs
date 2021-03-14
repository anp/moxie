use color_eyre::eyre::Result;
use starlark::{
    environment::FrozenModule,
    environment::{Globals, Module},
    eval::{Evaluator, FileLoader},
    syntax::{AstModule, Dialect},
};
use std::{
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, instrument, warn};

// mod builtins;
mod error;
mod vfs;

use error::{Error};
use vfs::Vfs;

pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Tracks changes to files we've read.
    vfs: Vfs,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "workspace.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf(), vfs: Vfs::new() }
    }

    pub fn maintain(self) -> Result<()> {
        // TODO change current directory to workspace root?
        info!("maintaining workspace");
        loop {
            if let Err(error) = self.converge() {
                error!(%error, "couldn't converge current workspace revision");
            }
            self.vfs.wait_for_changes();
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.root.display()))]
    fn converge(&self) -> Result<(), Error> {
        debug!("constructing workspace env");
        let _workspace_env = self.load(Self::ASSET_PATH).map_err(Error::StarlarkError)?;

        warn!("TODO display discovered targets");

        info!("finished");
        Ok(())
    }
}

struct Revision {
    modules: (),
}

impl FileLoader for Workspace {
    #[instrument(skip(self))]
    fn load(&self, path: &str) -> anyhow::Result<FrozenModule> {
        // TODO smarter way to resolve assets etc
        // TODO handle relative paths somehow?
        let file = self.root.join(path.strip_prefix("//").unwrap_or(path));
        debug!(file = %file.display(), "loading");

        let root_contents = self.vfs.read(&file).expect("TODO pass errors back correctly here");
        let root_contents =
            std::str::from_utf8(&*root_contents).expect("TODO pass errors back correctly here");

        let root_module = std::fs::read_to_string(Self::ASSET_PATH)?;

        let ast: AstModule =
            AstModule::parse(Self::ASSET_PATH, root_module, &Dialect::Standard)?;

        let globals: Globals = Globals::default();

        // TODO register our builtins as globals

        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module, &globals);
        eval.set_loader(self);
        let _res = eval.eval_module(ast)?;

        Ok(module.freeze())
    }
}
