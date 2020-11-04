use crate::error::Error;
use codemap::CodeMap;
use codemap_diagnostic::{ColorConfig, Emitter};
use crossbeam_channel::TryRecvError;
use memofs::{Vfs, VfsEvent};
use starlark::{
    environment::{Environment, TypeValues},
    eval::{noload::NoLoadFileLoader, EvalException, FileLoader},
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tracing::{debug, info, instrument, trace};

pub struct VfsLoader {
    vfs: Vfs,
}

impl Default for VfsLoader {
    fn default() -> Self {
        Self { vfs: Vfs::new_default() }
    }
}

impl VfsLoader {
    pub fn wait_for_changes(&self) {
        let changes = self.vfs.event_receiver();
        match changes.recv().unwrap() {
            VfsEvent::Create(created) => info!(created = %created.display()),
            VfsEvent::Write(modified) => info!(modified = %modified.display()),
            VfsEvent::Remove(removed) => info!(removed = %removed.display()),
            _ => unimplemented!("unrecognized filesystem event"),
        }

        // TODO figure out how much memofs debounces, make sure its enough or we do some
        debug!("draining other fs events until quiescent");
        loop {
            match changes.try_recv() {
                Ok(event) => trace!(?event, "discarding"),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    unreachable!("other end is kept alive by ourselves")
                }
            }
        }
    }

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
