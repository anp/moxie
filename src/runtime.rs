//! [`Runtime`]s are the primary integration point between moxie and
//! embedding environments.

mod context;
mod runloop;
mod var;

use dyn_cache::local::SharedLocalCache;
use futures::{
    future::LocalFutureObj,
    task::{noop_waker, LocalSpawn, SpawnError},
};
use illicit::AsContext;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    rc::Rc,
    task::Waker,
};

pub(crate) use context::Context;
pub use runloop::RunLoop;
pub(crate) use var::Var;

/// Revisions measure moxie's notion of time passing. Each `Runtime` increments
/// its Revision on every iteration. `crate::Commit`s to state variables are
/// annotated with the Revision during which they were made.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// Returns the current revision. Will return `Revision(0)` if called
    /// outside of a Runtime's execution.
    pub fn current() -> Self {
        if let Ok(r) = illicit::get::<Context>() {
            r.revision()
        } else {
            Revision::default()
        }
    }
}

impl std::fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("r{}", self.0))
    }
}

/// A [`Runtime`] is the primary integration point between moxie and an
/// embedder. Each independent instance is responsible for an event loop and
/// tracks time using a [`Revision`] which it increments on each iteration of
/// the loop. Owns the cache and state for a given context.
///
/// # Key considerations
///
/// ## Event Loops
///
/// moxie assumes that it will be responsible for interfacing with or managing
/// an event loop with the basic structure:
///
/// 1. enter loop
/// 2. present interface
/// 3. wait for changes
/// 4. goto (2)
///
/// Step (2) is implemented in [`Runtime::run_once`], and the user code run by
/// that method is expected to register the runtime for wakeups on future
/// events. Step (3) is implemented by the embedder, waiting until the runtime's
/// change notification waker is invoked.
///
/// ## Change notifications
///
/// Each runtime should be provided with a [`std::task::Waker`] that will notify
/// the embedding environment to run the loop again. This is done by calling
/// [`Runtime::set_state_change_waker`].
///
/// For scenarios without an obvious "main thread" this can be done for you by
/// binding a root function to a  [`RunLoop`] which implements
/// [`std::future::Future`] and can be spawned as a task onto an executor. For
/// more nuanced scenarios it can be necessary to write your own waker to ensure
/// scheduling compatible with the embedding environment. By default a no-op
/// waker is provided.
///
/// The most common way of notifying a runtime of a change is to update a
/// state variable.
///
/// ## Caching
///
/// When a runtime is repeatedly invoking the same code for every [`Revision`]
/// there's likely to be a lot of repetitive work. This might be something
/// complex like a slow computation over the set of visible items, or it might
/// be something simple like using the same DOM node for a given button on every
/// [`Revision`].
///
/// While not strictly *necessary* to integrate moxie, much of the runtime's
/// potential value comes from identifying work that's repeated across frames
/// and storing it in the runtime's cache instead of recomputing every time.
///
/// Internally the runtime hosts a [dyn-cache] instance which is
/// garbage-collected at the end of each [`Revision`]. All cached values are
/// stored there and evicted at the end of revisions where they went unused.
/// This behavior also provides deterministic drop timing for values cached by
/// the runtime.
///
/// ## Tasks
///
/// Each runtime expects to be able to spawn futures as async tasks, provided
/// with [`Runtime::set_task_executor`]. By default a no-op spawner is provided.
///
/// # Minimal Example
///
/// This example has no side effects in its root function, and doesn't have any
/// state variables which might require change notifications.
///
/// ```
/// # use moxie::runtime::{Revision, Runtime};
/// let mut rt = Runtime::new();
/// assert_eq!(rt.revision().0, 0);
/// for i in 1..10 {
///     rt.run_once(|| ());
///     assert_eq!(rt.revision(), Revision(i));
/// }
/// ```
///
/// [dyn-cache]: https://docs.rs/dyn-cache
pub struct Runtime {
    revision: Revision,
    cache: SharedLocalCache,
    spawner: Spawner,
    wk: Waker,
}

impl Default for Runtime {
    fn default() -> Runtime {
        Runtime::new()
    }
}

impl Runtime {
    /// Construct a new [`Runtime`] with blank storage and no external waker or
    /// task executor.
    pub fn new() -> Self {
        Self {
            spawner: Spawner(Rc::new(JunkSpawner)),
            revision: Revision(0),
            cache: SharedLocalCache::default(),
            wk: noop_waker(),
        }
    }

    /// The current revision of the runtime, or how many times `run_once` has
    /// been invoked.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Runs the root closure once with access to the runtime context,
    /// increments the runtime's `Revision`, and drops any cached values
    /// which were not marked alive.
    pub fn run_once<Out>(&mut self, op: impl FnOnce() -> Out) -> Out {
        self.revision.0 += 1;

        let ret = self.context_handle().offer(|| topo::call(op));

        self.cache.gc();
        ret
    }

    /// Sets the [`std::task::Waker`] which will be called when state variables
    /// receive commits. By default the runtime no-ops on a state change,
    /// which is probably the desired behavior if the embedding system will
    /// call `Runtime::run_once` on a regular interval regardless.
    pub fn set_state_change_waker(&mut self, wk: Waker) {
        self.wk = wk;
    }

    /// Sets the executor that will be used to spawn normal priority tasks.
    pub fn set_task_executor(&mut self, sp: impl LocalSpawn + 'static) {
        self.spawner = Spawner(Rc::new(sp));
    }
}

#[derive(Clone)]
struct Spawner(pub Rc<dyn LocalSpawn>);

impl Debug for Spawner {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("{:p}", &self.0))
    }
}

struct JunkSpawner;
impl LocalSpawn for JunkSpawner {
    fn spawn_local_obj(&self, _: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        Err(SpawnError::shutdown())
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        Err(SpawnError::shutdown())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagating_env_to_runtime() {
        let first_byte = 0u8;

        let mut runtime = RunLoop::new(|| {
            let from_env: u8 = *illicit::expect();
            assert_eq!(from_env, first_byte);
        });

        assert!(illicit::get::<u8>().is_err());
        first_byte.offer(|| {
            topo::call(|| runtime.run_once());
        });
        assert!(illicit::get::<u8>().is_err());
    }

    #[test]
    fn tick_a_few_times() {
        let mut rt = RunLoop::new(Revision::current);
        assert_eq!(rt.run_once(), Revision(1));
        assert_eq!(rt.run_once(), Revision(2));
        assert_eq!(rt.run_once(), Revision(3));
        assert_eq!(rt.run_once(), Revision(4));
        assert_eq!(rt.run_once(), Revision(5));
    }
}
