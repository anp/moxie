use crate::{
    error::Error,
    revision::{EvaluatorExt, Revision},
    vfs::Vfs,
};
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
use tracing::{debug, instrument};

#[derive(Clone)]
pub struct WorkspaceState {
    inner: Arc<Mutex<InnerState>>,
}

impl WorkspaceState {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerState {
                vfs: Vfs::new(),
                root: root.as_ref().to_path_buf(),
                current_revision: Default::default(),
                _prev_revision: Default::default(),
            })),
        }
    }

    pub fn root(&self) -> PathBuf {
        self.inner.lock().root.clone()
    }

    pub fn load(&self, path: &str) -> crate::Result<FrozenModule> {
        self.inner.lock().load(path).map_err(Error::StarlarkError)
    }

    pub fn current_revision(&self) -> Revision {
        self.inner.lock().current_revision.clone()
    }

    pub fn start_new_revision(&self) {
        self.inner.lock().start_new_revision()
    }

    pub fn wait_for_changes(&self) {
        // TODO undo this lock lol
        self.inner.lock().vfs.wait_for_changes()
    }
}

struct InnerState {
    /// Path to `workspace.honk`.
    root: PathBuf,

    /// Tracks changes to files we've read.
    vfs: Vfs,

    current_revision: Revision,
    _prev_revision: Revision,
}

impl InnerState {
    fn start_new_revision(&mut self) {
        self._prev_revision = std::mem::replace(&mut self.current_revision, Revision::default());
    }
}

impl FileLoader for InnerState {
    #[instrument(skip(self))]
    fn load(&mut self, path: &str) -> anyhow::Result<FrozenModule> {
        // TODO smarter way to resolve assets etc
        // TODO handle relative paths somehow?
        let path = path.strip_prefix("//").unwrap_or(path);
        let file = self.root.join(path);
        debug!(file = %file.display(), "loading");

        let root_contents = self.vfs.read(&file)?;
        let root_contents = std::str::from_utf8(&*root_contents)?;

        let ast: AstModule = AstModule::parse(path, root_contents.to_string(), &Dialect::Standard)?;

        let globals = GlobalsBuilder::extended_by(&[LibraryExtension::StructType])
            .with(crate::builtins::register)
            .build();
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module, &globals);

        let revision = self.current_revision.clone();
        eval.set_revision(&revision);
        eval.set_loader(self);
        let _res = eval.eval_module(ast)?;

        Ok(module.freeze())
    }
}
