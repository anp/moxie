#![feature(await_macro, futures_api, async_await)]

#[macro_use]
extern crate rental;

use futures::executor::ThreadPool;

mod display_list;
#[macro_use]
pub mod state;
mod surface;
mod winit_future;

use crate::prelude::*;

pub(crate) mod prelude {
    pub type Size = euclid::TypedSize2D<i32, webrender::api::DevicePixel>;

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

    ThreadPool::new().unwrap().run(
        async {
            let compose = state::Composer::new();

            loop {
                compose.surface(ScopeId::root());
            }
        },
    );
}
