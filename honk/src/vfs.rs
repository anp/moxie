use starlark::{
    environment::{Environment, TypeValues},
    eval::{noload::NoLoadFileLoader, EvalException, FileLoader},
};
use std::ops::{Deref, DerefMut};

pub struct Vfs {
    inner: memofs::Vfs,
}

impl FileLoader for Vfs {
    fn load(&self, path: &str, type_values: &TypeValues) -> Result<Environment, EvalException> {
        // TODO load from the vfs!
        NoLoadFileLoader.load(path, type_values)
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Vfs { inner: memofs::Vfs::new_default() }
    }
}

impl Deref for Vfs {
    type Target = memofs::Vfs;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Vfs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
