#![deny(clippy::all)]
#![feature(await_macro, futures_api, async_await)]

pub mod color;
mod drop_guard;
mod events;
pub mod position;
pub mod size;
pub mod surface;

use {
    crate::{color::Color, size::Size, surface::surface},
    moxie::{channel::Sender, Moxie, Scope},
};

#[salsa::query_group(WrenchDrawer)]
pub trait Components: moxie::Runtime {
    fn surface(
        &self,
        scope: Scope,
        initial_size: Size,
        mouse_events: Sender<surface::CursorMoved>,
        color: Color,
    ) -> ();
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
