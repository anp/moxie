use parking_lot::Mutex;
use starlark::{
    environment::{FrozenModule, GlobalsBuilder, LibraryExtension, Module},
    eval::{Evaluator, FileLoader},
    syntax::{AstModule, Dialect},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, error, info, instrument};

pub mod builtins;
pub mod error;
pub mod graph;
pub mod revision;
pub mod vfs;

use error::Error;
use revision::{EvaluatorExt, Revision};
use vfs::Vfs;

pub(crate) type Result<T> = color_eyre::eyre::Result<T, Error>;

pub struct Workspace {
    state: WorkspaceState,
}

impl Workspace {
    /// The asset path used to resolve the root of a honk workspace.
    const ASSET_PATH: &'static str = "WORKSPACE.honk";

    pub fn new(root: impl AsRef<Path>) -> Self {
        let state = WorkspaceState {
            root: root.as_ref().to_path_buf(),
            inner: Arc::new(Mutex::new(InnerState { vfs: Vfs::new() })),
        };
        Self { state }
    }

    pub fn maintain(self) -> crate::Result<()> {
        // TODO change current directory to workspace root?
        info!("maintaining workspace");
        loop {
            if let Err(error) = self.converge() {
                error!(%error, "couldn't converge current workspace revision");
            }
            self.state.inner.lock().vfs.wait_for_changes();
        }
    }

    #[instrument(level = "info", skip(self), fields(root = %self.state.root.display()))]
    fn converge(&self) -> crate::Result<()> {
        debug!("constructing workspace env");
        let mut loader = RevisionLoader(self.state.clone(), Revision::default());
        let _workspace_env = loader.load(Self::ASSET_PATH).map_err(Error::StarlarkError)?;

        let _build = loader.1.resolve()?;
        info!("discovered targets");

        // FIXME make this an actual web viewer via http server, right?
        dump_graphviz(&_build);

        tracing::warn!("uh run some builds i guess?");

        info!("finished");
        Ok(())
    }
}

#[derive(Clone)]
pub struct WorkspaceState {
    /// Path to `workspace.honk`.
    root: PathBuf,

    inner: Arc<Mutex<InnerState>>,
}

struct InnerState {
    /// Tracks changes to files we've read.
    vfs: Vfs,
}

fn dump_graphviz(g: &graph::ActionGraph) {
    use petgraph::dot::{Config, Dot};
    let output = Dot::with_config(g, &[Config::EdgeNoLabel]);
    println!("{}", output);
}

struct RevisionLoader(WorkspaceState, Revision);

impl FileLoader for RevisionLoader {
    #[instrument(skip(self))]
    fn load(&mut self, path: &str) -> anyhow::Result<FrozenModule> {
        // TODO smarter way to resolve assets etc
        // TODO handle relative paths somehow?
        let path = path.strip_prefix("//").unwrap_or(path);
        let file = self.0.root.join(path);
        debug!(file = %file.display(), "loading");

        let root_contents = self.0.inner.lock().vfs.read(&file)?;
        let root_contents = std::str::from_utf8(&*root_contents)?;

        let ast: AstModule = AstModule::parse(path, root_contents.to_string(), &Dialect::Standard)?;

        let globals = GlobalsBuilder::extended_by(&[LibraryExtension::StructType])
            .with(crate::builtins::register)
            .build();
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module, &globals);
        eval.disable_gc(); // we're going to drop this right away

        let revision = self.1.clone();
        eval.set_revision(&revision);
        eval.set_loader(self);
        let _res = eval.eval_module(ast)?;

        Ok(module.freeze())
    }
}
