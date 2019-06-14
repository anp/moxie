#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
pub extern crate tokio_trace;

#[doc(hidden)]
pub use tokio_trace as __trace;

#[macro_use]
pub mod memo;
pub mod state;

pub use {memo::*, state::*};

use {
    futures_timer::Delay,
    std::time::{Duration, Instant},
    topo::topo,
};

pub trait Component {
    fn content(self);
}

#[topo]
pub fn show(_child: impl Component) {
    unimplemented!()
}

pub enum LoopBehavior {
    OnWake,
    Vsync(Duration),
    Stopped,
}

impl Default for LoopBehavior {
    fn default() -> Self {
        LoopBehavior::OnWake
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

pub async fn runloop(mut root: impl FnMut(&state::Key<LoopBehavior>)) {
    let task_waker = LoopWaker(std::future::get_task_context(|c| c.waker().clone()));

    let mut current_revision = Revision(0);
    loop {
        let start = Instant::now();
        let mut behavior_key = None;
        topo::call!(|| {
            let (_, behavior) = state!((), |()| LoopBehavior::default());
            println!("running root");
            root(&behavior);
            behavior_key = Some(behavior);
        }, env: {
            LoopWaker => task_waker.clone(),
            Revision => current_revision
        });

        topo::Point::__flush();

        let loop_duration = Instant::now() - start;
        trace!("revision {:?} took {:?}", current_revision, loop_duration);
        current_revision.0 += 1;

        let next_behavior = behavior_key.as_mut().unwrap().read().unwrap();

        match *next_behavior {
            LoopBehavior::OnWake => futures::pending!(),
            LoopBehavior::Vsync(frame_time) => {
                Delay::new(frame_time - loop_duration).await.unwrap()
            }
            LoopBehavior::Stopped => break,
        }
    }
}

#[derive(Clone)]
struct LoopWaker(std::task::Waker);

impl LoopWaker {
    fn wake(&self) {
        self.0.wake_by_ref();
    }
}

// #[topo]
// pub fn task(_fut: impl Future<Output = ()> + Send + UnwindSafe + 'static) {
//     unimplemented!()
// }
