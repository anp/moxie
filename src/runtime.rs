//! Integration points for the moxie runtime. Generally not called directly by
//! applications.
//!
//! To integrate moxie with an existing system, a `Runtime` must be created with
//! a user-provided root closure to execute each `Revision`. If the embedding
//! system will change its scheduling based on mutations of moxie state
//! variables (e.g. scheduling a new frame render), then it should also call
//! `Runtime::set_state_change_waker` to be notified when state values have been
//! updated.
//!
//! Once these have been created, the embedding system should call
//! `Runtime::run_once` whenever appropriate for the context. This may be on a
//! regular cadence (i.e. once per frame interval) or only when state mutations
//! have occurred (as is the default in [moxie-dom]).
//!
//! [moxie-dom]: https://docs.rs/moxie-dom

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
        if let Ok(r) = illicit::get::<Context>() { r.revision() } else { Revision::default() }
    }
}

impl std::fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("r{}", self.0))
    }
}

/// The primary integration point between `moxie` and an embedding environment.
/// Owns the cache and state for a moxie application. Each instance is
/// independent.
///
/// ## Minimal Example
///
/// This example has no side effects in its root closure, and doesn't have any
/// state variables which might require mutation wakeups.
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
