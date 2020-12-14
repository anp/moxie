//! moxie aims to empower everyone to build reliable and efficient human
//! interfaces.
//!
//! moxie supports incremental "declarative" Rust code for interactive systems.
//! It comes with a lightweight event loop runtime that supports granular
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
pub mod testing;

use crate::runtime::{Context, Var};
use parking_lot::Mutex;
use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    future::Future,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
    task::Poll,
};
use topo::CallId;

/// Applied to impl blocks, this macro defines a new "updater" wrapper type that
/// holds a [`crate::Key`] and forwards all receiver-mutating methods. Useful
/// for defining interactions for a stateful component with less boilerplate.
///
/// Requires the name of the updater struct to generate in the arguments to the
/// attribute.
pub use moxie_macros::updater;

/// Cache the return of the `init` function.
///
/// If the cache has a stored `(Input, Output)` for the current [`topo::CallId`]
/// and if `arg` is equal to the stored `Input`, marks the value as alive in the
/// cache and returns the result of calling `with` on the stored `Output`.
///
/// Otherwise, calls `arg.to_owned()` to get an `Input` and calls `init` to get
/// an `Output`. It calls `with` on the `Output` to get a `Ret` value, stores
/// the `(Input, Output)` in the cache, and returns `Ret`.
///
/// # Example
///
/// ```
/// use moxie::{cache_with, runtime::RunLoop, testing::CountsClones};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// let epoch = AtomicU64::new(0);
/// let num_created = AtomicU64::new(0);
///
/// // this runtime holds a single state variable
/// // which is reinitialized whenever we change `epoch` above
/// let mut rt = RunLoop::new(|| {
///     let cached = cache_with(
///         &epoch.load(Ordering::Relaxed),
///         |_| {
///             num_created.fetch_add(1, Ordering::Relaxed);
///             CountsClones::default()
///         },
///         // this makes it equivalent to calling moxie::once(...)
///         CountsClones::clone,
///     );
///
///     (num_created.load(Ordering::Relaxed), cached.clone_count())
/// });
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 1, "the first value is always cached");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
///
/// epoch.store(1, Ordering::Relaxed); // invalidates the cache
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 2, "reinitialized once after epoch changed");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
/// ```
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

/// Caches `init` once in the current [`topo::CallId`]. Runs `with` on every
/// [`runtime::Revision`].
///
/// # Example
///
/// ```
/// use moxie::{once_with, runtime::RunLoop, testing::CountsClones};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// let num_created = AtomicU64::new(0);
/// let mut rt = RunLoop::new(|| {
///     let cached = once_with(
///         || {
///             num_created.fetch_add(1, Ordering::Relaxed);
///             CountsClones::default()
///         },
///         // this makes it equivalent to calling moxie::once(...)
///         CountsClones::clone,
///     );
///     (num_created.load(Ordering::Relaxed), cached.clone_count())
/// });
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 1, "the first value is always cached");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
/// ```
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn once_with<Output, Ret>(
    init: impl FnOnce() -> Output,
    with: impl FnOnce(&Output) -> Ret,
) -> Ret
where
    Output: 'static,
    Ret: 'static,
{
    rt.cache.cache_with(&CallId::current(), &(), |&()| init(), with)
}

/// Memoizes `init` at this callsite, cloning a cached `Output` if it exists and
/// `Input` is the same as when the stored value was created.
///
/// `init` takes a reference to `Input` so that the cache can
/// compare future calls' arguments against the one used to produce the stored
/// value.
///
/// # Example
///
/// ```
/// use moxie::{cache, runtime::RunLoop, testing::CountsClones};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// let epoch = AtomicU64::new(0);
/// let num_created = AtomicU64::new(0);
///
/// // this runtime holds a single state variable
/// // which is reinitialized whenever we change `epoch` above
/// let mut rt = RunLoop::new(|| {
///     let cached = cache(&epoch.load(Ordering::Relaxed), |_| {
///         num_created.fetch_add(1, Ordering::Relaxed);
///         CountsClones::default()
///     });
///
///     (num_created.load(Ordering::Relaxed), cached.clone_count())
/// });
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 1, "the first value is always cached");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
///
/// epoch.store(1, Ordering::Relaxed);
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 2, "reinitialized once after epoch changed");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
/// ```
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

/// Runs `init` once per [`topo::CallId`]. The provided value
/// will always be cloned on subsequent calls unless first dropped from storage
/// before being re-initialized.
///
/// # Example
///
/// ```
/// use moxie::{once, runtime::RunLoop, testing::CountsClones};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// let num_created = AtomicU64::new(0);
/// let mut rt = RunLoop::new(|| {
///     let cached = once(|| {
///         num_created.fetch_add(1, Ordering::Relaxed);
///         CountsClones::default()
///     });
///     (num_created.load(Ordering::Relaxed), cached.clone_count())
/// });
///
/// for i in 1..1_000 {
///     let (num_created, num_clones) = rt.force_next();
///     assert_eq!(num_created, 1, "the first value is always cached");
///     assert_eq!(num_clones, i, "cloned once per revision");
/// }
/// ```
#[topo::nested]
#[illicit::from_env(rt: &Context)]
pub fn once<Output>(init: impl FnOnce() -> Output) -> Output
where
    Output: Clone + 'static,
{
    rt.cache.cache(&CallId::current(), &(), |()| init())
}

/// Root a state variable at this callsite, returning a [`Key`] to the state
/// variable.
///
/// # Example
///
/// ```
/// use futures::task::waker;
/// use moxie::{runtime::RunLoop, state, testing::BoolWaker};
///
/// // this runtime holds a single state variable
/// let mut rt = RunLoop::new(|| state(|| 0u64));
///
/// let track_wakes = BoolWaker::new();
///
/// let (first_commit, first_key) = rt.force_next_with(waker(track_wakes.clone()));
/// assert_eq!(*first_commit, 0, "no updates yet");
/// assert!(!track_wakes.is_woken(), "no updates yet");
///
/// first_key.set(0); // this is a no-op
/// assert_eq!(**first_key.commit_at_root(), 0, "no updates yet");
/// assert!(!track_wakes.is_woken(), "no updates yet");
///
/// first_key.set(1);
/// assert_eq!(**first_key.commit_at_root(), 0, "update only enqueued, not yet committed");
/// assert!(track_wakes.is_woken());
///
/// let (second_commit, second_key) = rt.force_next(); // this commits the pending update
/// assert_eq!(**second_key.commit_at_root(), 1);
/// assert_eq!(*second_commit, 1);
/// assert_eq!(*first_commit, 0, "previous value still held by previous pointer");
/// assert!(!track_wakes.is_woken(), "wakes only come from updating state vars");
/// assert_eq!(first_key, second_key, "same state variable");
/// ```
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
///
/// # Example
///
/// ```
/// use moxie::{cache_state, runtime::RunLoop, testing::BoolWaker};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// let epoch = AtomicU64::new(0);
///
/// // this runtime holds a single state variable
/// // which is reinitialized whenever we change `epoch` above
/// let mut rt = RunLoop::new(|| cache_state(&epoch.load(Ordering::Relaxed), |e| *e));
///
/// let track_wakes = BoolWaker::new();
///
/// let (first_commit, first_key) = rt.force_next_with(futures::task::waker(track_wakes.clone()));
/// assert_eq!(*first_commit, 0, "no updates yet");
/// assert!(!track_wakes.is_woken(), "no updates yet");
///
/// first_key.set(0); // this is a no-op
/// assert_eq!(**first_key.commit_at_root(), 0, "no updates yet");
/// assert!(!track_wakes.is_woken(), "no updates yet");
///
/// first_key.set(1);
/// assert_eq!(**first_key.commit_at_root(), 0, "update only enqueued, not yet committed");
/// assert!(track_wakes.is_woken());
///
/// let (second_commit, second_key) = rt.force_next(); // this commits the pending update
/// assert_eq!(**second_key.commit_at_root(), 1);
/// assert_eq!(*second_commit, 1);
/// assert_eq!(*first_commit, 0, "previous value still held by previous pointer");
/// assert!(!track_wakes.is_woken(), "wakes only come from updating state vars");
/// assert_eq!(first_key, second_key, "same state variable");
///
/// // start the whole thing over again
/// epoch.store(2, Ordering::Relaxed);
///
/// let (third_commit, third_key) = rt.force_next();
/// assert_ne!(third_key, second_key, "different state variable");
///
/// // the rest is repeated from above with slight modifications
/// assert_eq!(*third_commit, 2);
/// assert!(!track_wakes.is_woken());
///
/// third_key.set(2);
/// assert_eq!(**third_key.commit_at_root(), 2);
/// assert!(!track_wakes.is_woken());
///
/// third_key.set(3);
/// assert_eq!(**third_key.commit_at_root(), 2);
/// assert!(track_wakes.is_woken());
///
/// let (fourth_commit, fourth_key) = rt.force_next();
/// assert_eq!(**fourth_key.commit_at_root(), 3);
/// assert_eq!(*fourth_commit, 3);
/// assert_eq!(*third_commit, 2);
/// assert!(!track_wakes.is_woken());
/// assert_eq!(third_key, fourth_key);
/// ```
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
///
/// # Example
///
/// ```
/// use futures::{channel::oneshot, executor::LocalPool};
/// use moxie::{load_with, runtime::RunLoop};
/// use std::{
///     sync::{
///         atomic::{AtomicU64, Ordering},
///         mpsc::channel,
///     },
///     task::Poll,
/// };
///
/// let epoch = AtomicU64::new(0);
/// let (send_futs, recv_futs) = channel();
///
/// let mut rt = RunLoop::new(|| {
///     // loads a new future when epoch changes
///     load_with(
///         &epoch.load(Ordering::Relaxed),
///         |_| {
///             let (sender, receiver) = oneshot::channel();
///             send_futs.send(sender).unwrap();
///             receiver
///         },
///         // makes this equivalent to load(...)
///         |res| res.clone(),
///     )
/// });
///
/// let mut exec = LocalPool::new();
/// rt.set_task_executor(exec.spawner());
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run_until_stalled();
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// // resolve the future
/// let sender = recv_futs.recv().unwrap();
/// assert!(recv_futs.try_recv().is_err(), "only one channel is created per epoch");
///
/// sender.send(()).unwrap();
///
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
///
/// // force the future to be reinitialized
/// epoch.store(1, Ordering::Relaxed);
///
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// // resolve the future
/// let sender = recv_futs.recv().unwrap();
/// assert!(recv_futs.try_recv().is_err(), "only one channel is created per epoch");
///
/// sender.send(()).unwrap();
///
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
/// ```
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
///
/// # Example
///
/// ```
/// use futures::{channel::oneshot, executor::LocalPool};
/// use moxie::{load_once_with, runtime::RunLoop};
/// use std::task::Poll;
///
/// let (sender, receiver) = oneshot::channel();
/// let mut receiver = Some(receiver);
/// let mut rt = RunLoop::new(|| load_once_with(|| receiver.take().unwrap(), |res| res.clone()));
///
/// let mut exec = LocalPool::new();
/// rt.set_task_executor(exec.spawner());
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run_until_stalled();
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// sender.send(()).unwrap();
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
/// ```
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
///
/// # Example
///
/// ```
/// use futures::{channel::oneshot, executor::LocalPool};
/// use moxie::{load_once, runtime::RunLoop};
/// use std::task::Poll;
///
/// let (sender, receiver) = oneshot::channel();
/// let mut receiver = Some(receiver);
/// let mut rt = RunLoop::new(|| load_once(|| receiver.take().unwrap()));
///
/// let mut exec = LocalPool::new();
/// rt.set_task_executor(exec.spawner());
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run_until_stalled();
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// sender.send(()).unwrap();
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
/// ```
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
///
/// # Example
///
/// ```
/// use futures::{channel::oneshot, executor::LocalPool};
/// use moxie::{load, runtime::RunLoop};
/// use std::{
///     sync::{
///         atomic::{AtomicU64, Ordering},
///         mpsc::channel,
///     },
///     task::Poll,
/// };
///
/// let epoch = AtomicU64::new(0);
/// let (send_futs, recv_futs) = channel();
///
/// let mut rt = RunLoop::new(|| {
///     // loads a new future when epoch changes
///     load(&epoch.load(Ordering::Relaxed), |e| {
///         let (sender, receiver) = oneshot::channel();
///         send_futs.send((*e, sender)).unwrap();
///         receiver
///     })
/// });
///
/// let mut exec = LocalPool::new();
/// rt.set_task_executor(exec.spawner());
///
/// assert_eq!(rt.force_next(), Poll::Pending);
/// exec.run_until_stalled();
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// // resolve the future
/// let (created_in_epoch, sender) = recv_futs.recv().unwrap();
/// assert!(recv_futs.try_recv().is_err(), "only one channel is created per epoch");
/// assert_eq!(created_in_epoch, 0);
///
/// sender.send(()).unwrap();
///
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
///
/// // force the future to be reinitialized
/// epoch.store(1, Ordering::Relaxed);
///
/// assert_eq!(rt.force_next(), Poll::Pending);
///
/// // resolve the future
/// let (created_in_epoch, sender) = recv_futs.recv().unwrap();
/// assert!(recv_futs.try_recv().is_err(), "only one channel is created per epoch");
/// assert_eq!(created_in_epoch, 1);
///
/// sender.send(()).unwrap();
///
/// exec.run();
/// assert_eq!(rt.force_next(), Poll::Ready(Ok(())));
/// ```
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
///
/// See [`state`] and [`cache_state`] for examples.
#[derive(Eq, Hash, PartialEq)]
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
/// supported with [`Key::update`] and [`Key::set`].
///
/// They are created with the [`cache_state`] and [`state`] functions.
///
/// See [`state`] and [`cache_state`] for examples.
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

    /// Returns the `Commit` of the current `Revision`
    pub fn commit_at_root(&self) -> &Commit<State> {
        &self.commit_at_root
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
    ///
    /// # Example
    ///
    /// ```
    /// use futures::task::waker;
    /// use moxie::{runtime::RunLoop, state, testing::BoolWaker};
    ///
    /// // this runtime holds a single state variable
    /// let mut rt = RunLoop::new(|| state(|| 0u64));
    ///
    /// let track_wakes = BoolWaker::new();
    ///
    /// let (first_commit, first_key) = rt.force_next_with(waker(track_wakes.clone()));
    /// assert_eq!(*first_commit, 0, "no updates yet");
    /// assert!(!track_wakes.is_woken(), "no updates yet");
    ///
    /// first_key.update(|_| None); // this is a no-op
    /// assert_eq!(**first_key.commit_at_root(), 0, "no updates yet");
    /// assert!(!track_wakes.is_woken(), "no updates yet");
    ///
    /// first_key.update(|prev| Some(prev + 1));
    /// assert_eq!(**first_key.commit_at_root(), 0, "update only enqueued, not yet committed");
    /// assert!(track_wakes.is_woken());
    ///
    /// let (second_commit, second_key) = rt.force_next(); // this commits the pending update
    /// assert_eq!(**second_key.commit_at_root(), 1);
    /// assert_eq!(*second_commit, 1);
    /// assert_eq!(*first_commit, 0, "previous value still held by previous pointer");
    /// assert!(!track_wakes.is_woken(), "wakes only come from updating state vars");
    /// assert_eq!(first_key, second_key, "same state variable");
    /// ```
    pub fn update(&self, updater: impl FnOnce(&State) -> Option<State>) {
        let mut var = self.var.lock();
        if let Some(new) = updater(var.latest()) {
            var.enqueue_commit(new);
        }
    }

    /// Set a new value for the state variable, immediately taking effect.
    fn force(&self, new: State) {
        self.var.lock().enqueue_commit(new);
    }
}

impl<State> Key<State>
where
    State: PartialEq,
{
    /// Commits a new state value if it is unequal to the current value and the
    /// state variable is still live. Has the same properties as
    /// [update](Key::update) regarding waking the runtime.
    ///
    /// See [`state`] and [`cache_state`] for examples.
    pub fn set(&self, new: State) {
        self.update(|prev| if prev == &new { None } else { Some(new) });
    }
}

impl<State> Key<State>
where
    State: Clone + PartialEq,
{
    /// Mutates a copy of the current state, committing the update if it results
    /// in a change. Has the same properties as [update](Key::update)
    /// See [`state`] and [`cache_state`] for examples.
    pub fn mutate(&self, op: impl FnOnce(&mut State)) {
        self.update(|prev| {
            let mut new = prev.clone();
            op(&mut new);
            if prev == &new { None } else { Some(new) }
        });
    }
}

impl<State> Clone for Key<State> {
    fn clone(&self) -> Self {
        Self { id: self.id, commit_at_root: self.commit_at_root.clone(), var: self.var.clone() }
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

impl<State> Eq for Key<State> {}

impl<State> Hash for Key<State> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
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

                rt.force_next();

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
            rt.force_next();
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

            let first_counts = rt.force_next();
            assert_eq!(first_counts.len(), num_iters, "each mutation must be called exactly once");

            let second_counts = rt.force_next();
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

                rt.force_next();
                rt.force_next();
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

        assert_eq!(rt.force_next(), Poll::Pending, "no values received when nothing sent");
        assert_eq!(rt.force_next(), Poll::Pending, "no values received, and we aren't blocking");

        send.send(5u8).unwrap();
        pool.run_until_stalled();
        assert_eq!(rt.force_next(), Poll::Ready(5), "we need to receive the value we sent");
        assert_eq!(
            rt.force_next(),
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
        assert_eq!(rt.force_next(), Some(Poll::Pending));
        assert!(!send.is_canceled(), "interest expressed, receiver must be live");

        pool.run_until_stalled();
        assert_eq!(rt.force_next(), Some(Poll::Pending));
        assert!(!send.is_canceled(), "interest still expressed, receiver must be live");

        pool.run_until_stalled();
        assert_eq!(rt.force_next(), None);
        assert!(!send.is_canceled(), "interest dropped, task live for another revision");

        pool.run_until_stalled();
        assert_eq!(rt.force_next(), None);
        assert!(send.is_canceled(), "interest dropped, task dropped");

        assert!(
            send.send(4u8).is_err(),
            "must be no task holding the channel and able to receive a message"
        );
    }
}
