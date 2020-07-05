use super::{
    token::{OpaqueToken, Token},
    Point,
};
use std::{borrow::Borrow, hash::Hash, panic::Location};

/// Identifies the scope of a nested function call in a way that can be
/// deterministically reproduced across multiple executions.
///
/// The [`CallId::current`] for a function call is the combined product of:
///
/// * a callsite: the [`std::panic::Location`] where the function was called
/// * a parent: the [`CallId::current`] which was active when calling the
///   function
/// * a [slot](#slots): a value indicating the call's "index" within the parent
///   call
///
/// When a nested call returns or unwinds, it reverts [`CallId::current`] to
/// the parent `CallId`.
///
/// # Example
///
/// ```
/// use topo::{call, root, CallId};
///
/// let returns_two_ids = || {
///     let first = call(|| CallId::current());
///     let second = call(|| CallId::current());
///     assert_ne!(first, second, "these are always distinct calls");
///     (first, second)
/// };
///
/// // running the closure as a nested call(...) gives different siblings
/// assert_ne!(call(returns_two_ids), call(returns_two_ids));
///
/// // a call to root(...) gives each of these closure calls an identical parent CallId
/// assert_eq!(root(returns_two_ids), root(returns_two_ids));
/// ```
///
/// # Creation
///
/// Every `CallId` is created by calling one of:
///
/// * a function marked [`nested`]
/// * a function passed to [`call`]
/// * a function and slot passed to [`call_in_slot`]
///
/// # Slots
///
/// Slots are used to differentiate between repeated calls at the same callsite
/// and define the "index" of a child call within its parent. By default (and in
/// [`call`]) the slot is populated by the number of times the current
/// callsite has been called in this parent. Users can provide their own slot
/// with [`call_in_slot`] or using `#[topo::nested(slot = "...")]`:
///
/// See [`call_in_slot`] and [`nested`] for examples.
///
/// # Roots
///
/// The topmost parent or "root" of a callgraph can be defined in two ways:
///
/// 1. a [`call`] or [`call_in_slot`] invocation with no parent implicitly
/// creates its own root
/// 2. an explicit call to [`root`] creates a new subgraph regardless of the
/// current parent
///
/// See [`root`] for examples.
///
/// # `CallId` and multiple threads
///
/// The [`illicit`] environment used for tracking the current `CallId` is
/// thread-local, but by default [`Token`] values used to track slots are
/// interned in the global cache. This means that two different threads calling
/// an identical chain of nested functions can observe identical `CallId`s:
///
/// ```
/// # use topo::{call, root, CallId};
/// #
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #
/// # let returns_two_ids = || {
/// #     let first = call(|| CallId::current());
/// #     let second = call(|| CallId::current());
/// #     assert_ne!(first, second, "these are always distinct calls");
/// #     (first, second)
/// # };
/// #
/// use std::{
///     sync::mpsc::{channel, Sender},
///     thread,
/// };
///
/// let (send_ids, recv_ids) = channel();
///
/// let spawn_worker = |sender: Sender<(CallId, CallId)>| {
///     thread::spawn(move || sender.send(root(returns_two_ids)).unwrap())
/// };
/// let first_thread = spawn_worker(send_ids.clone());
/// let second_thread = spawn_worker(send_ids);
///
/// first_thread.join().unwrap();
/// second_thread.join().unwrap();
///
/// // the two worker threads "did the same work"
/// assert_eq!(recv_ids.recv()?, recv_ids.recv()?);
/// #
/// # Ok(()) }
/// ```
///
/// [`nested`]: `crate::nested`
/// [`call`]: `crate::call`
/// [`call_in_slot`]: `crate::call_in_slot`
/// [`root`]: `crate::root`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CallId {
    callsite: Callsite,
    parent: Token<CallId>,
    slot: OpaqueToken,
}

impl CallId {
    /// Returns the root `CallId`.
    pub(crate) fn root() -> Self {
        Self {
            callsite: Callsite::here(),
            parent: Token::fake(),
            slot: Token::<String>::fake().into(),
        }
    }

    /// Returns the current `CallId`.
    pub fn current() -> Self {
        Point::with_current(|current| current.id)
    }

    pub(crate) fn child<Q, S>(&self, callsite: Callsite, slot: &Q) -> Self
    where
        Q: Eq + Hash + ToOwned<Owned = S> + ?Sized,
        S: Borrow<Q> + Eq + Hash + Send + 'static,
    {
        Self { callsite, parent: Token::make(self), slot: Token::make(slot).into() }
    }
}

/// A value unique to the source location where it is created.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Callsite {
    location: usize,
}

impl Callsite {
    /// Constructs a callsite whose value is unique to the source location at
    /// which it is called.
    #[track_caller]
    pub fn here() -> Self {
        Location::caller().into()
    }

    /// Returns the number of times this callsite has been seen as a child of
    /// the current Point.
    pub fn current_count(self) -> u32 {
        Point::with_current(|current| {
            if let Some(c) = current.callsite_counts.borrow().iter().find(|(site, _)| site == &self)
            {
                c.1
            } else {
                0
            }
        })
    }
}

impl From<&'static Location<'static>> for Callsite {
    fn from(location: &'static Location<'static>) -> Self {
        Self {
            // the pointer value for a given location is enough to differentiate it from all others
            location: location as *const _ as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{call, root};
    use std::{sync::mpsc::channel, thread};

    #[test]
    fn threads_and_ids() {
        let returns_two_ids = || {
            let first = call(|| CallId::current());
            let second = call(|| CallId::current());
            assert_ne!(first, second, "these are always distinct calls");
            (first, second)
        };

        let (send_ids, recv_ids) = channel();
        let send_ids2 = send_ids.clone();
        let first_thread = thread::spawn(move || send_ids2.send(root(returns_two_ids)).unwrap());
        let second_thread = thread::spawn(move || send_ids.send(root(returns_two_ids)).unwrap());

        first_thread.join().unwrap();
        second_thread.join().unwrap();

        let (first, second) = (recv_ids.recv().unwrap(), recv_ids.recv().unwrap());
        assert_eq!(first, second);
    }
}
