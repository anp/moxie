use codemap::CodeMap;
use color_eyre::eyre::Result;
use starlark::{
    environment::{Environment, TypeValues},
    eval::{EvalException, FileLoader},
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::{error, info, instrument, warn};

mod builtins;
mod error;
mod vfs;

use error::{Error, EvalError};
use vfs::Vfs;

pub struct Workspace {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Tracks changes to files we've read.
    vfs: Vfs,

    codemap: Arc<Mutex<CodeMap>>,
    type_values: TypeValues,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "//workspace.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        let codemap = Arc::new(Mutex::new(CodeMap::new()));

        let (mut throwaway_env, mut type_values) =
            starlark::stdlib::global_environment_with_extensions();
        // TODO figure out how to do this once instead of here *and* below in `load()`?
        builtins::register(&mut throwaway_env, &mut type_values);

        Self { root: root.as_ref().to_path_buf(), vfs: Vfs::new(), codemap, type_values }
    }

    pub fn maintain(self) -> Result<()> {
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
        let _workspace_env = self
            .load(Self::ASSET_PATH, &self.type_values)
            .map_err(|e| EvalError::from_exception(e, self.codemap.clone()))?;

        warn!("TODO display discovered targets");

        info!("finished");
        Ok(())
    }
}

impl FileLoader for Workspace {
    #[instrument(skip(self, type_values))]
    fn load(&self, path: &str, type_values: &TypeValues) -> Result<Environment, EvalException> {
        // TODO smarter way to resolve assets etc
        let file = self.root.join(path.strip_prefix("//").unwrap_or(path));
        info!(file = %file.display(), "loading");

        let root_contents = self.vfs.read(&file).expect("TODO pass errors back correctly here");
        let root_contents =
            std::str::from_utf8(&*root_contents).expect("TODO pass errors back correctly here");

        let (mut env, mut throwaway_tvs) = starlark::stdlib::global_environment_with_extensions();
        // TODO figure out how to do this once instead of here *and* above in `new()`?
        builtins::register(&mut env, &mut throwaway_tvs);

        info!("evaluating");
        starlark::eval::eval(
            &self.codemap,
            &file.to_string_lossy(),
            &root_contents,
            // TODO do we ever want to restrict function definitions?
            starlark::syntax::dialect::Dialect::Bzl,
            &mut env,
            type_values,
            self,
        )?;

        Ok(env)
    }
}
