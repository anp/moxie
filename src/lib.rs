#![feature(await_macro, futures_api, async_await, integer_atomics)]

#[macro_use]
extern crate rental;

#[macro_use]
pub mod state;
mod events;
mod surface;

// TODO split into public/private preludes
pub mod prelude {
    pub use {
        crate::state::{CallsiteId, Composer, Guard, Handle, Moniker, ScopeId, Surface},
        futures::{
            prelude::*,
            stream::{Stream, StreamExt},
        },
        log::{debug, error, info, trace, warn},
        std::sync::Arc,
    };
}

use {crate::prelude::*, futures::pending};

pub fn run() {
    info!("starting threadpool");
    futures::executor::ThreadPool::new().unwrap().run(
        async {
            let compose = Composer::default();
            loop {
                trace!("composing surface");
                compose.surface(scope!());
                pending!()
            }
        },
    );
}
