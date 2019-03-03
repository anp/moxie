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
    moxie::{Scope, Scopes, Sender},
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

moxie::runtime!(Toolbox: WrenchDrawer, moxie::TaskQueries);
