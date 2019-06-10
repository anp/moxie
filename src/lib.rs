#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
pub extern crate tokio_trace;

use topo::topo;

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

pub async fn runloop(mut root: impl FnMut(&dyn Stop)) {
    let (send_shutdown, recv_shutdown) = std::sync::mpsc::channel();
    let task_waker = std::future::get_task_context(|c| c.waker().clone());

    loop {
        topo::call!(|| {
            println!("running root");
            root(&send_shutdown);
        }, env: {
            LoopWaker => task_waker
        });

        println!("root finished, checking shutdown channel");
        if let Ok(()) = recv_shutdown.try_recv() {
            break;
        }
        println!("marking pending");
        topo::Point::__flush();
        //        futures::pending!();
    }
}

struct LoopWaker(std::task::Waker);

impl LoopWaker {
    fn wake(&self) {
        self.0.wake_by_ref();
    }
}

pub trait Stop {
    fn stop(&self);
}

impl Stop for std::sync::mpsc::Sender<()> {
    // TODO make this an async function when possible?
    // by awaiting the future you'd be able to ensure the runloop was fully canceled
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
