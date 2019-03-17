#![deny(clippy::all)]
#![feature(await_macro, futures_api, async_await)]

pub mod color;
mod drop_guard;
mod events;
pub mod position;
pub mod size;
pub mod surface;
