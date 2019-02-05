#![feature(await_macro, futures_api, async_await, integer_atomics)]

#[macro_use]
extern crate rental;

mod display_list;
#[macro_use]
pub mod state;
mod runtime;
mod surface;
mod winit_future;

pub(crate) mod prelude {
    pub type Size = euclid::TypedSize2D<i32, webrender::api::DevicePixel>;

    pub use futures::{
        future::{Future, FutureExt},
        stream::{Stream, StreamExt},
        task::LocalWaker,
    };
    pub use log::{debug, error, info, warn};

    pub use crate::{
        display_list::DisplayList,
        state::{CallsiteId, ComposeDb, Composer, Guard, Handle, Moniker, ScopeId},
    };
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .default_format_timestamp(true)
        .default_format_level(true)
        .default_format_module_path(true)
        .filter(Some("webrender"), log::LevelFilter::Warn)
        .filter(Some("salsa"), log::LevelFilter::Warn)
        .init();

    futures::executor::ThreadPool::new()
        .unwrap()
        .run(runtime::RuntimeyWimey::default().gogo());
}
