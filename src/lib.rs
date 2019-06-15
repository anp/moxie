#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
pub extern crate tokio_trace;

#[doc(hidden)]
pub use tokio_trace as __trace;

#[macro_use]
pub mod memo;
pub mod state;

#[doc(inline)]
pub use {memo::*, state::*};

use {tokio_trace::field::debug, topo::topo};

pub trait Component {
    fn content(self);
}

#[topo]
pub fn show(_child: impl Component) {
    unimplemented!()
}

#[derive(Eq, PartialEq)]
pub enum LoopBehavior {
    OnWake,
    Stopped,
    #[cfg(test)] // a dirty dirty hack for tests for now, need to fix with tasks/timers
    Continue,
}

impl Default for LoopBehavior {
    fn default() -> Self {
        LoopBehavior::OnWake
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// Returns the current revision. Will always return `Revision(0)` if called outside of a
    /// runloop.
    pub fn current() -> Self {
        if let Some(r) = topo::env::get::<Revision>() {
            *r
        } else {
            Revision::default()
        }
    }
}

/// A `runloop` is the entry point of the moxie runtime environment.
///
/// On each iteration
/// of the loop:
///
/// 1. The loop's [Revision](crate::Revision) counter is incremented by 1.
/// 2. The provided `root` argument is called within a corresponding [Point](topo::Point) in the
///    call topology.
/// 3. The loop marks its task as pending until it is woken by commits to state variables.
///     * If during (2) `root` commits a `LoopBehavior::Stopped` change to the referenced
///       [LoopBehavior](crate::LoopBehavior) state [Key](crate::Key), then control flow
///       for the running future breaks out of the loop and returns, ending the runloop.
///
/// The simplest possible runloop stops itself as soon as it is entered:
///
/// ```
/// # #![feature(async_await)]
/// # #[runtime::main]
/// # async fn main() {
///
/// moxie::runloop(|ctl| ctl.set(moxie::LoopBehavior::Stopped)).await;
///
/// # }
/// ```
///
/// Most practical usages of the runloop rely on its continued execution, however.
///
/// TODO: add counting example or something actually useful maybe?
pub async fn runloop(mut root: impl FnMut(&state::Key<LoopBehavior>)) {
    let task_waker = LoopWaker(std::future::get_task_context(|c| c.waker().clone()));

    let mut current_revision = Revision(0);
    loop {
        current_revision.0 += 1;

        let mut behavior_key = None;
        topo::call!(|| {
            let (_, behavior) = state!((), |()| LoopBehavior::default());

            // CALLER'S CODE IS CALLED HERE
            root(&behavior);

            // stash the write key for ourselves for reading after exiting this call
            behavior_key = Some(behavior);
        }, Env {
            LoopWaker => task_waker.clone(),
            Revision => current_revision
        });

        // TODO break this by adding test with multiple identical runloops that clobber each other
        topo::Point::__flush();

        let next_behavior = behavior_key.as_mut().unwrap().read().unwrap();

        match *next_behavior {
            LoopBehavior::OnWake => futures::pending!(),
            LoopBehavior::Stopped => {
                info!(target: "runloop_stopping", revision = debug(&current_revision));
                break;
            }
            #[cfg(test)]
            LoopBehavior::Continue => continue,
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
