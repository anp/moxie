//! moxie is a library (and perhaps a programming model) for constructing and incrementally updating
//! persistent trees (think DOM) with memoized function calls.

#![deny(clippy::all)]
#![feature(async_await, gen_future)]

#[macro_use]
mod memo;
mod state;

#[doc(inline)]
pub use {memo::*, state::*};

use {
    std::ops::Deref,
    topo::__trace::{field::debug, *},
};

/// Controls the iteration behavior of a runloop. The default of `OnWake` will leave the runloop
/// in a pending state until a state variable receives a commit, at which point the runloop's task
/// will be woken and its executor will poll it again.
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

/// A Revision represents moxie's notion of time, a counter which is incremented each time its
/// runloop iterates. [`Commit`]s to state variables are annotated with the Revision during which
/// they were made.
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

/// A `runloop` is the entry point of the moxie runtime environment. The async function returns a
/// `Future<Output=()>` which calls the root closure once per call to [Future::poll]. Typically
/// the root closure will cause some memoized side effect to the outer environment in order to
/// render a view of data available to the user. A runloop's root closure will also distribute other
/// asynchronous tasks to an executor to handle I/O events and to update the state persisted by the
/// runloop during its iteration.
///
/// While the runloop will iterate very frequently (potentially more than once for any given output
/// frame), we use [topological memoization](crate::memo) to minimize the code we run each time.
///
/// [Future::poll]: std::future::Future::poll
///
/// # Loop Body
///
/// On each iteration of the loop:
///
/// 1. The loop's [`Revision`] counter is incremented by 1.
/// 2. The provided `root` function is called within its [`topo::Point`] in the call topology.
/// 3. By default, the loop marks its task as pending until it is woken by commits to state
///    variables.
///     * If during (2) `root` commits a `LoopBehavior::Stopped` change to the referenced
///       state [`Key`]`<`[`LoopBehavior`]`>`, then control flow for the running future breaks out
///       of the loop and returns out of the runloop.
///
/// # Examples
///
/// ## Minimal
///
/// The simplest possible runloop stops itself as soon as it is entered. Most practical usages of
/// the runloop rely on its continued execution, however.
///
/// ```
/// # #![feature(async_await)]
/// # #[runtime::main]
/// # async fn main() {
/// moxie::runloop(|ctl| {
///     ctl.set(moxie::LoopBehavior::Stopped);
/// }).await;
/// # }
/// ```
pub async fn runloop(mut root: impl FnMut(&state::Key<LoopBehavior>)) {
    let task_waker = RunLoopWaker(std::future::get_task_context(|c| c.waker().clone()));

    let mut current_revision = Revision(0);
    let mut next_behavior = None;
    loop {
        current_revision.0 += 1;

        topo::call!(|| {
            let (_, behavior) = state!((), |()| LoopBehavior::default());

            // CALLER'S CODE IS CALLED HERE
            root(&behavior);

            // stash the write key for ourselves for reading after exiting this call
            next_behavior = behavior.flushed().read();
        }, Env {
            RunLoopWaker => task_waker.clone(),
            Revision => current_revision
        });

        // TODO break this by adding test with multiple identical runloops that clobber each other
        topo::Point::__flush();

        match next_behavior.as_ref().unwrap().deref() {
            LoopBehavior::OnWake => {
                trace!(target: "runloop_pending", revision = debug(&current_revision));
                futures::pending!();
            }
            LoopBehavior::Stopped => {
                info!(target: "runloop_stopping", revision = debug(&current_revision));
                break;
            }
            #[cfg(test)]
            LoopBehavior::Continue => continue,
        }
    }
}

/// Responsible for waking the runloop task. Because the topo environment is namespaced by type,
/// we create a newtype here so that other crates don't accidentally cause strage behavior by
/// overriding our access to it.
#[derive(Clone)]
struct RunLoopWaker(std::task::Waker);

impl RunLoopWaker {
    fn wake(&self) {
        self.0.wake_by_ref();
    }
}

// #[topo]
// pub fn task(_fut: impl Future<Output = ()> + Send + UnwindSafe + 'static) {
//     unimplemented!()
// }
