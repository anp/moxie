use starlark::{
    environment::FrozenModule,
    environment::Module,
    eval::{Evaluator, FileLoader},
    syntax::{AstModule, Dialect},
};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, warn};

pub mod builtins;
pub mod error;
pub mod vfs;

use error::Error;
use vfs::Vfs;

pub(crate) type Result<T> = color_eyre::eyre::Result<T, Error>;

pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Tracks changes to files we've read.
    vfs: Vfs,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "WORKSPACE.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf(), vfs: Vfs::new() }
    }

    pub fn maintain(mut self) -> crate::Result<()> {
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
    fn converge(&mut self) -> crate::Result<()> {
        debug!("constructing workspace env");
        let _workspace_env = self.load(Self::ASSET_PATH).map_err(Error::StarlarkError)?;

        warn!("TODO display discovered targets");

        info!("finished");
        Ok(())
    }
}

impl FileLoader for Workspace {
    #[instrument(skip(self))]
    fn load(&mut self, path: &str) -> anyhow::Result<FrozenModule> {
        // TODO smarter way to resolve assets etc
        // TODO handle relative paths somehow?
        let path = path.strip_prefix("//").unwrap_or(path);
        let file = self.root.join(path);
        debug!(file = %file.display(), "loading");

        let root_contents = self.vfs.read(&file).expect("TODO pass errors back correctly here");
        let root_contents =
            std::str::from_utf8(&*root_contents).expect("TODO pass errors back correctly here");

        let ast: AstModule = AstModule::parse(path, root_contents.to_string(), &Dialect::Standard)?;

        let globals = starlark::stdlib::standard_environment()
            // TODO figure out how to add set() back
            .with(starlark::stdlib::add_struct)
            .with(crate::builtins::register)
            .build();
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module, &globals);
        eval.set_loader(self);
        let _res = eval.eval_module(ast)?;

        Ok(module.freeze())
    }
}
