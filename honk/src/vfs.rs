use crate::error::Error;
use codemap::CodeMap;
use codemap_diagnostic::{ColorConfig, Emitter};
use starlark::{
    environment::{Environment, TypeValues},
    eval::{noload::NoLoadFileLoader, EvalException, FileLoader},
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tracing::{info, instrument};

pub struct VfsLoader {
    vfs: memofs::Vfs,
}

impl Default for VfsLoader {
    fn default() -> Self {
        Self { vfs: memofs::Vfs::new_default() }
    }
}

impl VfsLoader {
    pub fn load_workspace_env(&self, root: impl AsRef<Path>) -> Result<Environment, Error> {
        let root = root.as_ref();
        info!("reading workspace file");
        let root_contents = self.vfs.read(root)?;
        let root_contents = std::str::from_utf8(&*root_contents)
            .map_err(|source| Error::ScriptEncoding { source, file: root.to_path_buf() })?;

        let map = Arc::new(Mutex::new(CodeMap::new()));
        let types = TypeValues::default();
        let mut env = Environment::new("honk");

        info!("evaluating workspace file");
        match starlark::eval::eval(
            &map,
            &root.to_string_lossy(),
            &root_contents,
            // TODO figure out if this is the right dialect?
            starlark::syntax::dialect::Dialect::Bzl,
            &mut env,
            &types,
            self,
        ) {
            Ok(_output) => Ok(env),
            Err(diagnostic) => {
                // TODO reconcile this stderr reporting with other mechanisms like HTTP
                let map = map.lock().unwrap();
                let mut emitter = Emitter::stderr(ColorConfig::Auto, Some(&*map));
                emitter.emit(&[diagnostic]);
                Err(Error::Eval)
            }
        }
    }
}

impl FileLoader for VfsLoader {
    #[instrument(skip(self))]
    fn load(&self, path: &str, type_values: &TypeValues) -> Result<Environment, EvalException> {
        info!("loading file");
        // TODO load from the vfs!
        NoLoadFileLoader.load(path, type_values)
    }
}
