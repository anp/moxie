#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
pub extern crate tokio_trace;

use {
    futures::task::LocalSpawn,
    std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe},
    topo::topo,
};

#[doc(hidden)]
pub use tokio_trace as __trace;

pub mod memo;

pub trait Component {
    fn content(self);
}

#[topo]
pub fn show(_child: impl Component) {
    unimplemented!()
}

pub async fn runloop(
    root: impl Fn(&dyn Stop) + Clone + UnwindSafe + 'static,
    _spawner: impl LocalSpawn + Send + 'static,
) {
    // make sure we can be woken back up and exited
    // std::future::get_task_context(|cx| WAKER.set(cx.waker().clone()));
    // SPAWNER.set(Box::new(spawner));

    let (send_shutdown, recv_shutdown) = std::sync::mpsc::channel();
    loop {
        let send_shutdown = AssertUnwindSafe(send_shutdown.clone());
        let root = root.clone();

        let tick_result = catch_unwind(move || root(&*send_shutdown));

        if let Err(e) = tick_result {
            error!("error composing: {:?}", e);
            break;
        }

        if let Ok(()) = recv_shutdown.try_recv() {
            break;
        }

        futures::pending!();
    }
}

pub trait Stop {
    fn stop(&self);
}

impl Stop for std::sync::mpsc::Sender<()> {
    fn stop(&self) {
        let _ = self.send(());
    }
}

// #[topo]
// pub fn state<S: 'static + Any + UnwindSafe>(_init: impl FnOnce() -> S) -> Guard<S> {
//     unimplemented!()
// }

// #[topo]
// pub fn task(_fut: impl Future<Output = ()> + Send + UnwindSafe + 'static) {
//     unimplemented!()
// }

// pub struct Guard<State> {
//     // TODO
//     _ty: std::marker::PhantomData<State>,
// }

// impl<State> Guard<State> {
//     pub fn key(&self) -> Key<State> {
//         unimplemented!()
//     }
// }

// impl<State> Deref for Guard<State> {
//     type Target = State;
//     fn deref(&self) -> &Self::Target {
//         unimplemented!()
//     }
// }

// impl<State> DerefMut for Guard<State> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unimplemented!()
//     }
// }

// pub struct Key<State> {
//     _ty: std::marker::PhantomData<State>,
// }
