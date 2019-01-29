#![feature(await_macro, futures_api, async_await)]

#[macro_use]
extern crate rental;

use futures::executor::ThreadPool;

pub mod canny_map;
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
        state::{Handle, Moniker, RenderDatabase},
    };
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .default_format_timestamp(true)
        .default_format_level(true)
        .default_format_module_path(true)
        .filter(Some("webrender"), log::LevelFilter::Warn)
        .init();

    ThreadPool::new().unwrap().run(
        async {
            let db = state::Db::new();

            db.with(|compose| {
                compose.Surface(Moniker::root());
            });

            // let (mut surface, mut events) = surface::Surface::new();

            // while let Some(event) = await!(events.next()) {
            //     match surface.update(event) {
            //         ControlFlow::Continue => continue,
            //         ControlFlow::Break => break,
            //     }
            // }
        },
    );
}
