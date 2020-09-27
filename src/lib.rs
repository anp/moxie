//! moxie aims to empower everyone to build reliable and efficient human
//! interfaces.
//!
//! moxie supports incremental "declarative" Rust code for interactive systems.
//! It comes with a lightweight "event loop runtime" that supports granular
//! reuse of arbitrary work, state change notifications, and async loaders.
//!
//! Most users of this crate will do so through a "moxie embedding" like
//! [moxie-dom] which is responsible for integrating moxie with a broader
//! environment. The functions in this module are applicable to any moxie
//! embedding but end users should not expect to set up their own embedding (see
//! the [`runtime`] module for information on embeddings).
//!
//! ## Revisions
//!
//! The [`runtime::Revision`] is a core concept for a moxie
//! [`runtime::Runtime`]: it's the notion of time passing. In typical
//! embeddings, every frame results in a new revision.
//!
//! ## Topologically nested functions
//!
//! The functions in this module are intended to be called repeatedly, possibly
//! on every revision. The results returned must be **stable across revisions**,
//! so we use the [topo] crate to provide stable cache keys for each invocation.
//! Each function in the root module is annotated with `#[topo::nested]` and
//! will inherit the [`topo::CallId`] within which it's called.
//!
//! ## Caching
//!
//! Nearly all UIs benefit from reusing results between frames, in moxie this is
//! supported by the [`cache`], [`cache_with`], [`once`], and [`once_with`]
//! functions. Values returned from cached closures are available in subsequent
//! [`runtime::Revision`]s at the same callsite and are dropped from the cache
//! at the end of the first revision where they were not used.
//!
//! ## State
//!
//! State variables are stored in the cache and can be mutated in between
//! revisions. They are declared with the [`cache_state`] and [`state`]
//! functions which return a [`Commit`] for reading the current value and a
//! [`Key`] for updating it. Updates to state variables wake the runtime,
//! initiating a new revision.
//!
//! ## Loading Futures
//!
//! Futures can be "loaded" by the runtime using the [`load`], [`load_with`],
//! [`load_once`], and [`load_once_with`] functions. These functions ensure the
//! future is spawned to an async executor and return its status on every
//! revision. When the future has completed, `Poll::Ready` is returned on
//! each revision. If a revision occurs without referencing the pending future,
//! the task is cancelled.
//!
//! [moxie-dom]: https://docs.rs/moxie-dom
//! [topo]: https://docs.rs/topo/

#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

pub mod runtime;

use crate::runtime::{Context, Var};
use parking_lot::Mutex;
use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    future::Future,
    ops::Deref,
    sync::Arc,
    task::Poll,
};
use topo::CallId;

/// Cache the return of the `init` function.
///
/// If the cache has a stored `(Input, Output)` for the current [`topo::CallId`]
/// and if `arg` is equal to the stored `Input`, marks the value as alive in the
/// cache and returns the result of calling `with` on the stored `Output`.
///
/// Otherwise, calls `arg.to_owned()` to get an `Input` and calls `init` to get
/// an `Output`. It calls `with` on the `Output` to get a `Ret` value, stores
/// the `(Input, Output)` in the cache afresh, and returns the `Ret`.
///
/// It is technically possible to nest calls to `cache_with` and other functions
/// in this module, but the values they store won't be correctly retained across
/// `Revision`s until we track dependency information. As a result, it's not
/// recommended.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn cache_with<Arg, Input, Output, Ret>(
    arg: &Arg,
    init: impl FnOnce(&Input) -> Output,
    with: impl FnOnce(&Output) -> Ret,
) -> Ret
where
    Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
    Input: Borrow<Arg> + 'static,
    Output: 'static,
    Ret: 'static,
{
    rt.cache.cache_with(&CallId::current(), arg, init, with)
}

/// Memoizes `expr` once at the callsite. Runs `with` on every iteration.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn once_with<Output, Ret>(
    expr: impl FnOnce() -> Output,
    with: impl FnOnce(&Output) -> Ret,
) -> Ret
where
    Output: 'static,
    Ret: 'static,
{
    rt.cache.cache_with(&CallId::current(), &(), |&()| expr(), with)
}

/// Memoizes `init` at this callsite, cloning a cached `Output` if it exists and
/// `Input` is the same as when the stored value was created.
///
/// `init` takes a reference to `Input` so that the cache can
/// compare future calls' arguments against the one used to produce the stored
/// value.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn cache<Arg, Input, Output>(arg: &Arg, init: impl FnOnce(&Input) -> Output) -> Output
where
    Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
    Input: Borrow<Arg> + 'static,
    Output: Clone + 'static,
{
    rt.cache.cache(&CallId::current(), arg, init)
}

/// Runs the provided expression once per [`topo::CallId`]. The provided value
/// will always be cloned on subsequent calls unless dropped from storage and
/// reinitialized in a later `Revision`.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn once<Output>(expr: impl FnOnce() -> Output) -> Output
where
    Output: Clone + 'static,
{
    rt.cache.cache(&CallId::current(), &(), |()| expr())
}

/// Root a state variable at this callsite, returning a [`Key`] to the state
/// variable.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn state<Output>(init: impl FnOnce() -> Output) -> (Commit<Output>, Key<Output>)
where
    Output: 'static,
{
    rt.cache_state(&CallId::current(), &(), |_| init())
}

/// Root a state variable at this callsite, returning a [`Key`] to the state
/// variable. Re-initializes the state variable if the capture `arg` changes.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn cache_state<Arg, Input, Output>(
    arg: &Arg,
    init: impl FnOnce(&Input) -> Output,
) -> (Commit<Output>, Key<Output>)
where
    Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
    Input: Borrow<Arg> + 'static,
    Output: 'static,
{
    rt.cache_state(&CallId::current(), arg, init)
}

/// Load a value from the future returned by `init` whenever `capture` changes,
/// returning the result of calling `with` with the loaded value. Cancels the
/// running future after any revision during which this call was not made.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn load_with<Arg, Input, Fut, Output, Ret>(
    arg: &Arg,
    init: impl FnOnce(&Input) -> Fut,
    with: impl FnOnce(&Output) -> Ret,
) -> Poll<Ret>
where
    Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
    Input: Borrow<Arg> + 'static,
    Fut: Future<Output = Output> + 'static,
    Output: 'static,
    Ret: 'static,
{
    rt.load_with(&CallId::current(), arg, init, with)
}

/// Calls [`load_with`] but never re-initializes the loading future.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn load_once_with<Fut, Output, Ret>(
    init: impl FnOnce() -> Fut,
    with: impl FnOnce(&Output) -> Ret,
) -> Poll<Ret>
where
    Fut: Future<Output = Output> + 'static,
    Output: 'static,
    Ret: 'static,
{
    rt.load_with(&CallId::current(), &(), |()| init(), with)
}

/// Calls [`load_with`], never re-initializes the loading future, and clones the
/// returned value on each revision once the future has completed and returned.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn load_once<Fut, Output>(init: impl FnOnce() -> Fut) -> Poll<Output>
where
    Fut: Future<Output = Output> + 'static,
    Output: Clone + 'static,
{
    rt.load_with(&CallId::current(), &(), |()| init(), Clone::clone)
}

/// Load a value from a future, cloning it on subsequent revisions after it is
/// first returned. Re-initializes the loading future if the capture argument
/// changes from previous revisions.
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn load<Arg, Input, Fut, Output>(
    capture: &Arg,
    init: impl FnOnce(&Input) -> Fut,
) -> Poll<Output>
where
    Arg: PartialEq<Input> + ToOwned<Owned = Input> + ?Sized,
    Input: Borrow<Arg> + 'static,
    Fut: Future<Output = Output> + 'static,
    Output: Clone + 'static,
{
    rt.load_with(&CallId::current(), capture, init, Clone::clone)
}

/// A read-only pointer to the value of a state variable *at a particular
/// revision*.
///
/// Reads through a commit are not guaranteed to be the latest value visible to
/// the runtime. Commits should be shared and used within the context of a
/// single [`crate::runtime::Revision`], being re-loaded from the state variable
/// each time.
#[derive(Eq, PartialEq)]
pub struct Commit<State> {
    id: CallId,
    inner: Arc<State>,
}

impl<State> Clone for Commit<State> {
    fn clone(&self) -> Self {
        Self { id: self.id, inner: Arc::clone(&self.inner) }
    }
}

impl<State> Debug for Commit<State>
where
    State: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl<State> Deref for Commit<State> {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<State> Display for Commit<State>
where
    State: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("{}", self.inner))
    }
}

/// A `Key` offers access to a state variable. The key allows reads of the state
/// variable through a snapshot taken when the `Key` was created. Writes are
/// supported with [Key::update] and [Key::set].
///
/// They are created with the `cache_state` and `state` functions.
pub struct Key<State> {
    id: CallId,
    commit_at_root: Commit<State>,
    var: Arc<Mutex<Var<State>>>,
}

impl<State> Key<State> {
    /// Returns the `topo::CallId` at which the state variable is bound.
    pub fn id(&self) -> CallId {
        self.id
    }

    /// Runs `updater` with a reference to the state variable's latest value,
    /// and enqueues a commit to the variable if `updater` returns `Some`.
    /// Returns the `Revision` at which the state variable was last rooted
    /// if the variable is live, otherwise returns `None`.
    ///
    /// Enqueuing the commit invokes the state change waker registered with the
    /// [Runtime] (if any) to ensure that the code embedding the runtime
    /// schedules another call of [run_once].
    ///
    /// This should be called during event handlers or other code which executes
    /// outside of a `Revision`'s execution, otherwise unpredictable waker
    /// behavior may be obtained.
    ///
    /// [Runtime]: crate::runtime::Runtime
    /// [run_once]: crate::runtime::Runtime::run_once
    pub fn update(&self, updater: impl FnOnce(&State) -> Option<State>) {
        let mut var = self.var.lock();
        if let Some(new) = updater(var.latest()) {
            var.enqueue_commit(new);
        }
    }
}

impl<State> Key<State>
where
    State: PartialEq,
{
    /// Commits a new state value if it is unequal to the current value and the
    /// state variable is still live. Has the same properties as
    /// [update](Key::update) regarding waking the runtime.
    pub fn set(&self, new: State) {
        self.update(|prev| if prev == &new { None } else { Some(new) });
    }
}

impl<State> Clone for Key<State> {
    fn clone(&self) -> Self {
        Self { id: self.id, commit_at_root: self.commit_at_root.clone(), var: self.var.clone() }
    }
}

impl<State> Deref for Key<State> {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        self.commit_at_root.deref()
    }
}

impl<State> Debug for Key<State>
where
    State: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.commit_at_root.fmt(f)
    }
}

impl<State> Display for Key<State>
where
    State: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.commit_at_root.fmt(f)
    }
}

impl<State> PartialEq for Key<State> {
    /// Keys are considered equal if they point to the same state variable.
    /// Importantly, they will compare as equal even if they contain
    /// different snapshots of the state variable due to having been
    /// initialized in different revisions.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.var, &other.var)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{Revision, RunLoop};
    use std::{cell::Cell, collections::HashSet, rc::Rc};

    fn with_test_logs(test: impl FnOnce()) {
        tracing::subscriber::with_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_env_filter(tracing_subscriber::filter::EnvFilter::new("warn"))
                .finish(),
            || {
                tracing::debug!("logging init'd");
                test();
            },
        );
    }

    #[test]
    fn basic_cache() {
        with_test_logs(|| {
            let mut call_count = 0u32;

            let mut prev_revision = None;
            let mut comp_skipped_count = 0;
            let mut rt = RunLoop::new(|| {
                let revision = Revision::current();

                if let Some(pr) = prev_revision {
                    assert!(revision.0 > pr);
                } else {
                    comp_skipped_count += 1;
                }
                prev_revision = Some(revision.0);
                assert!(comp_skipped_count <= 1);

                assert!(revision.0 <= 5);
                let current_call_count = once(|| {
                    call_count += 1;
                    call_count
                });

                assert_eq!(current_call_count, 1);
                assert_eq!(call_count, 1);
            });

            for i in 0..5 {
                assert_eq!(rt.revision().0, i);

                rt.run_once();

                assert_eq!(rt.revision().0, i + 1);
            }
            assert_eq!(call_count, 1);
        })
    }

    #[test]
    fn id_in_loop() {
        topo::call(|| {
            let mut ids = HashSet::new();
            for _ in 0..10 {
                topo::call(|| ids.insert(CallId::current()));
            }
            assert_eq!(ids.len(), 10);

            let mut rt = RunLoop::new(|| {
                let mut ids = HashSet::new();
                for i in 0..10 {
                    cache(&i, |_| ids.insert(CallId::current()));
                }
                assert_eq!(ids.len(), 10);
            });
            rt.run_once();
        });
    }

    #[test]
    fn cache_in_a_loop() {
        with_test_logs(|| {
            let num_iters = 10;
            let mut rt = RunLoop::new(|| {
                let mut counts = vec![];
                for i in 0..num_iters {
                    topo::call(|| once(|| counts.push(i)));
                }
                counts
            });

            let first_counts = rt.run_once();
            assert_eq!(first_counts.len(), num_iters, "each mutation must be called exactly once");

            let second_counts = rt.run_once();
            assert_eq!(
                second_counts.len(),
                0,
                "each mutation was already called in the previous revision"
            );
        })
    }

    #[test]
    fn invalidation() {
        with_test_logs(|| {
            let loop_ct = Cell::new(0);
            let raw_exec = Cell::new(0);
            let cache_exec = Cell::new(0);
            let mut rt = RunLoop::new(|| {
                raw_exec.set(raw_exec.get() + 1);
                cache(&loop_ct.get(), |_| {
                    cache_exec.set(cache_exec.get() + 1);
                });
            });

            for i in 0..10 {
                loop_ct.set(i);

                assert_eq!(
                    cache_exec.get(),
                    i,
                    "cache block should execute exactly once per loop_ct value"
                );

                assert_eq!(
                    raw_exec.get(),
                    i * 2,
                    "runtime's root block should run exactly twice per loop_ct value"
                );

                rt.run_once();
                rt.run_once();
            }
        })
    }

    #[test]
    fn basic_loading_phases() {
        let mut pool = futures::executor::LocalPool::new();
        let (send, recv) = futures::channel::oneshot::channel();
        // this is uh weird, but we know up front how much we'll poll this
        let recv = Rc::new(futures::lock::Mutex::new(Some(recv)));

        let mut rt = RunLoop::new(move || -> Poll<u8> {
            let recv = recv.clone();
            load_once(|| async move {
                recv.lock()
                    .await
                    .take()
                    .expect("load_once should only allow us to take from the option once")
                    .await
                    .expect("we control the channel and won't drop it")
            })
        });
        rt.set_task_executor(pool.spawner());

        assert_eq!(rt.run_once(), Poll::Pending, "no values received when nothing sent");
        assert_eq!(rt.run_once(), Poll::Pending, "no values received, and we aren't blocking");

        send.send(5u8).unwrap();
        pool.run_until_stalled();
        assert_eq!(rt.run_once(), Poll::Ready(5), "we need to receive the value we sent");
        assert_eq!(
            rt.run_once(),
            Poll::Ready(5),
            "the value we sent must be cached because its from a oneshot channel"
        );
    }

    #[test]
    fn interest_loss_cancels_task() {
        let mut pool = futures::executor::LocalPool::new();
        let (send, recv) = futures::channel::oneshot::channel();
        let recv = Rc::new(futures::lock::Mutex::new(Some(recv)));

        let mut rt = RunLoop::new(move || -> Option<Poll<u8>> {
            if Revision::current().0 < 3 {
                let recv = recv.clone();
                Some(load_once(|| async move {
                    recv.lock()
                        .await
                        .take()
                        .expect("load_once should only allow us to take from the option once")
                        .await
                        .expect("we control the channel and won't drop it")
                }))
            } else {
                None
            }
        });
        rt.set_task_executor(pool.spawner());

        pool.run_until_stalled();
        assert_eq!(rt.run_once(), Some(Poll::Pending));
        assert!(!send.is_canceled(), "interest expressed, receiver must be live");

        pool.run_until_stalled();
        assert_eq!(rt.run_once(), Some(Poll::Pending));
        assert!(!send.is_canceled(), "interest still expressed, receiver must be live");

        pool.run_until_stalled();
        assert_eq!(rt.run_once(), None);
        assert!(!send.is_canceled(), "interest dropped, task live for another revision");

        pool.run_until_stalled();
        assert_eq!(rt.run_once(), None);
        assert!(send.is_canceled(), "interest dropped, task dropped");

        assert!(
            send.send(4u8).is_err(),
            "must be no task holding the channel and able to receive a message"
        );
    }
}
