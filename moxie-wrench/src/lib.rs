#![feature(await_macro, futures_api, async_await)]

mod drop_guard;
mod events;
pub mod surface;

use {
    crate::surface::surface,
    futures::{executor::ThreadPool, future::AbortHandle},
    moxie::{Moxie, Scope, ScopeId},
    std::task::Waker,
};

#[salsa::query_group(WrenchDrawer)]
pub trait Components: moxie::Runtime {
    // TODO replace this salsa annotation with passing a scope directly
    #[salsa::dependencies]
    fn surface(&self, id: ScopeId, width: u32, height: u32) -> ();
}

#[salsa::database(Moxie, WrenchDrawer)]
#[derive(Default)]
pub struct Toolbox {
    runtime: salsa::Runtime<Toolbox>,
    scopes: moxie::compose::Scopes,
}

impl salsa::Database for Toolbox {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.runtime
    }
}

impl moxie::Runtime for Toolbox {
    fn scopes(&self) -> &moxie::compose::Scopes {
        &self.scopes
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
