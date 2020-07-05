#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! `topo` provides stable callgraph identifiers and query caching for
//! implementing higher level [Incremental Computing] abstractions like
//! those in the [moxie](https://docs.rs/moxie) crate.
//!
//! # Cache storage
//!
//! Reusing the results from prior computations is a central aspect of
//! incremental computation and can be viewed as a [caching problem].
//! See the [`cache`] module for what this crate provides.
//!
//! # Scoping queries with [`CallId`]s
//!
//! There are several ways this crate's caches can be indexed, but for
//! incrementally computing repetitive hierarchical structures (like UI trees)
//! it can be very useful to describe cache queries in terms of a function's
//! abstract "location" within the runtime callgraph.
//!
//! To extract [`CallId`]s from code, the runtime callgraph must be "annotated"
//! with invocations of [`call`], [`call_in_slot`], and user-defined
//! [`nested`]-annotated functions. Each of these enters the scope of a child
//! [`CallId`] when called, causing [`CallId::current`] calls in that inner
//! scope to return the new [`CallId`].
//!
//! [`CallId`]s are deterministic and unique based on the preceding chain of
//! parent [`CallId`]s and data slots. Every chain has a root, either defined
//! implicitly by making a [`call`] without a parent, or explicitly by calling
//! [`root`]. By controlling the creation of roots, a runtime can ensure that
//! aside from changes to the executed graph itself, subsequent calls to the
//! same function will produce the same [`CallId`]s.
//!
//! ## Example
//!
//! ```
//! use topo::{call, root, CallId};
//!
//! let returns_two_ids = || {
//!     let first = call(|| CallId::current());
//!     let second = call(|| CallId::current());
//!     assert_ne!(first, second, "these are always distinct calls");
//!     (first, second)
//! };
//!
//! // running the closure as a nested call(...) gives different siblings
//! assert_ne!(call(returns_two_ids), call(returns_two_ids));
//!
//! // a call to root(...) gives each of these closure calls an identical parent CallId
//! assert_eq!(root(returns_two_ids), root(returns_two_ids));
//! ```
//!
//! ## Caching slots
//!
//! [`call_in_slot`] allows one to specify a "slot" -- a runtime value that
//! represents the call's "location" within its parents scope. This is
//! particularly useful for creating [`CallId`]s for child calls in collections
//! whose iteration order does not map exactly to the child's logical scope.
//!
//! An example might be a list of users where usernames are stable and their
//! associated resources should be cached by username, but the list order and
//! length change in ways unrelated to usernames. The username would be used as
//! the slot to prevent destruction and recreation of username-associated
//! resources when the list order changes.
//!
//! ```
//! use topo::{call_in_slot, CallId};
//!
//! let get_name_id = |name| call_in_slot(name, || CallId::current());
//!
//! // reusing the same slot will get the same CallId
//! let bob = get_name_id("bob");
//! let bob_again = get_name_id("bob");
//! assert_eq!(bob, bob_again);
//!
//! // different names produce different slots
//! let alice_hello = get_name_id("alice");
//! assert_ne!(bob, alice_hello);
//! ```
//!
//! Internally, slots are interned in a global cache with
//! [`cache::Token::make`]. [`Token`]s map uniquely to a value in the cache
//! storage.
//!
//! [Incremental Computing]: https://en.wikipedia.org/wiki/Incremental_computing
//! [caching problem]: https://en.wikipedia.org/wiki/Cache_(computing)

/// Gives a function a unique [`CallId`] in its caller's topology by applying
/// `#[track_caller]` to the function and wrapping its body in [`call`] or
/// [`call_in_slot`] if the `slot` parameter is given.
///
/// ```
/// #[topo::nested]
/// fn widget() -> topo::CallId {
///     topo::CallId::current()
/// }
///
/// // each call to the nested function gets a unique CallId in its parent's scope
/// assert_ne!(widget(), widget());
///
/// // nesting can be overridden by giving the function its own root
/// assert_eq!(topo::root(widget), topo::root(widget));
/// ```
///
/// # Slots
///
/// By default, `#[nested]` functions use for their slot the number of times the
/// current source location has been called during the span of the current
/// `CallId`. It is the behavior offered by the [`call`] shorthand.
///
/// To override the slot of a nested function, use the `slot` parameter, which
/// is then passed directly as the first argument to [`call_in_slot`]:
///
/// ```
/// #[topo::nested(slot = "name")]
/// fn get_name_id(name: &str, _value: &str) -> topo::CallId {
///     topo::CallId::current()
/// }
///
/// // reusing the same slot will get the same CallId
/// let bob = get_name_id("bob", "hello");
/// let bob_again = get_name_id("bob", "hello");
/// assert_eq!(bob, bob_again);
///
/// // the same name in a nested call returns a *new* CallId
/// let bob_nested = topo::call(|| get_name_id("bob", "hello"));
/// assert_ne!(bob, bob_nested);
///
/// // different names produce different slots, even when other args are the same
/// let alice_hello = get_name_id("alice", "hello");
/// assert_ne!(bob, alice_hello);
///
/// // changing non-slot arguments doesn't affect the CallId produced
/// let alice_goodbye = get_name_id("alice", "goodbye");
/// assert_eq!(alice_hello, alice_goodbye);
/// ```
///
/// See [`call_in_slot`] and [`CallId`]'s documentation for more information on
/// how slots are used.
#[doc(inline)]
pub use topo_macro::nested;

use cache::{OpaqueToken, Token};
use std::{borrow::Borrow, cell::RefCell, hash::Hash, panic::Location};

pub mod cache;

/// Calls the provided function as a child of [`CallId::current`], using for a
/// slot the number of times the given source location has been called during
/// the current parent's scope.
///
/// This is a useful default for calls which are not expected to repeat at the
/// same callsite during the parent scope, i.e. those that will only be called
/// once per scope. It is also a useful default for calls that will occur in a
/// loop where the positional index is the primary way of identifying repeated
/// entries into the child scope.
///
/// See [`CallId`], [`root`], [`call_in_slot`], and [`nested`].
///
/// # Example
///
/// ```
/// use topo::{call, root, CallId};
///
/// let get_list_of_ids = || {
///     topo::call(|| {
///         let mut ids = vec![];
///         for i in 0..3 {
///             let current = call(CallId::current);
///             if i > 0 {
///                 assert_ne!(ids[i - 1], current, "each CallId is different from the last");
///             }
///             ids.push(current);
///         }
///         ids
///     })
/// };
///
/// // without a parent call, each of these behaves as its own root
/// assert_eq!(get_list_of_ids(), get_list_of_ids());
///
/// // ...and explicitly wrapping each call in a root(...) produces the same result
/// assert_eq!(root(get_list_of_ids), root(get_list_of_ids), "explicit roots match");
///
/// // but when they're siblings under a single call, they produce distinct results
/// call(|| assert_ne!(get_list_of_ids(), get_list_of_ids(), "siblings don't match"));
/// ```
#[track_caller]
pub fn call<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    struct CallCount(u32);

    let callsite = Callsite::here();
    let count = CallCount(callsite.current_count());
    Scope::with_current(|p| p.enter_child(callsite, &count, op))
}

/// Calls the provided function as a child of [`CallId::current`], using `slot`
/// as an input for the new [`CallId`].
///
/// Because this overrides [`call`]'s default slot of call count, it is
/// possible for the same [`CallId`] to be issued multiple times during a
/// single parent scope.
///
/// # Examples
///
/// ```
/// use topo::{call_in_slot, CallId};
///
/// let get_name_id = |name, value| {
///     call_in_slot(name, || {
///         println!("{}", value);
///         CallId::current()
///     })
/// };
///
/// // reusing the same slot will get the same CallId
/// let bob = get_name_id("bob", "hello");
/// let bob_again = get_name_id("bob", "hello");
/// assert_eq!(bob, bob_again);
///
/// // the same name in a nested call returns a *new* CallId
/// let bob_nested = topo::call(|| get_name_id("bob", "hello"));
/// assert_ne!(bob, bob_nested);
///
/// // different names produce different slots
/// let alice_hello = get_name_id("alice", "hello");
/// assert_ne!(bob, alice_hello);
///
/// // changing non-slot arguments doesn't affect the CallId produced
/// let alice_goodbye = get_name_id("alice", "goodbye");
/// assert_eq!(alice_hello, alice_goodbye);
/// ```
///
/// Note that while [`call`] uses `call_in_slot` internally, there is no way to
/// manually "reuse" a call count slot with this function.
///
/// ```
/// use topo::{call, call_in_slot, CallId};
///
/// let get_lists_of_ids = || {
///     topo::call(|| {
///         let (mut counted_ids, mut slotted_ids) = (vec![], vec![]);
///         for i in 0..3 {
///             // (we're cheating here because we know that call() uses a u32)
///             let slotted = call_in_slot(&(i as u32), CallId::current);
///             let counted = call(CallId::current);
///
///             if i > 0 {
///                 assert_ne!(slotted_ids[i - 1], slotted);
///                 assert_ne!(counted_ids[i - 1], counted);
///             }
///             slotted_ids.push(slotted);
///             counted_ids.push(counted);
///         }
///
///         // these should *not* be the same despite emulating the call count
///         assert_ne!(&counted_ids, &slotted_ids);
///         (counted_ids, slotted_ids)
///     })
/// };
///
/// assert_eq!(get_lists_of_ids(), get_lists_of_ids());
/// ```
#[track_caller]
pub fn call_in_slot<F, Q, R, S>(slot: &Q, op: F) -> R
where
    F: FnOnce() -> R,
    Q: Eq + Hash + ToOwned<Owned = S> + ?Sized,
    S: Borrow<Q> + Eq + Hash + Send + 'static,
{
    Scope::with_current(|p| p.enter_child(Callsite::here(), slot, op))
}

/// Calls the provided function as the root of a new call tree, ignoring the
/// current `CallId`.
///
/// # Example
///
/// ```
/// // a call to root() here ensures the child is always treated as the same tree
/// // no matter from where the function is called
/// let independent = || topo::root(topo::CallId::current);
/// assert_eq!(topo::call(independent), topo::call(independent));
///
/// // this is a normal topo call, it returns `CallId`s based on the parent state
/// let dependent = || topo::call(topo::CallId::current);
/// assert_ne!(topo::call(dependent), topo::call(dependent));
/// ```
pub fn root<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    illicit::hide::<Scope>();
    call(op)
}

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
        Scope::with_current(|current| current.id)
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
struct Callsite {
    location: usize,
}

impl Callsite {
    /// Constructs a callsite whose value is unique to the source location at
    /// which this is called.
    #[track_caller]
    pub fn here() -> Self {
        Location::caller().into()
    }

    /// Returns the number of times this callsite has been seen in the current
    /// call.
    pub fn current_count(self) -> u32 {
        Scope::with_current(|current| {
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

/// The root of a sub-graph within the overall topology.
///
/// The current `Scope` contains the local [`CallId`] and a count of how often
/// each of its children has been called.
#[derive(Debug)]
struct Scope {
    /// current id
    id: CallId,
    /// source location for this scope's root
    callsite: Callsite,
    /// # times each callsite's type has been observed during this scope.
    callsite_counts: RefCell<Vec<(Callsite, u32)>>,
}

impl Scope {
    /// Mark a child Point in the topology, calling `child` within it.
    fn enter_child<C, Q, R, S>(&self, callsite: Callsite, slot: &Q, child: C) -> R
    where
        C: FnOnce() -> R,
        Q: Eq + Hash + ToOwned<Owned = S> + ?Sized,
        S: Borrow<Q> + Eq + Hash + Send + 'static,
    {
        self.increment_count(callsite);
        let child_point = Self {
            callsite,
            callsite_counts: RefCell::new(Default::default()),
            id: self.id.child(callsite, slot),
        };
        illicit::Layer::new().offer(child_point).enter(child)
    }

    /// Runs the provided closure with access to the current [`Point`].
    fn with_current<F, Out>(op: F) -> Out
    where
        F: FnOnce(&Scope) -> Out,
    {
        if let Ok(current) = illicit::get::<Scope>() {
            op(&*current)
        } else {
            op(&Scope::default())
        }
    }

    fn increment_count(&self, callsite: Callsite) {
        let mut counts = self.callsite_counts.borrow_mut();

        if let Some((_, count)) = counts.iter_mut().find(|(site, _)| site == &callsite) {
            *count += 1;
        } else {
            counts.push((callsite, 1));
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self { id: CallId::root(), callsite: Callsite::here(), callsite_counts: Default::default() }
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{call, root};
    use std::{collections::HashSet, sync::mpsc::channel, thread};

    #[test]
    fn alternating_in_a_loop() {
        call(|| {
            let mut ids = HashSet::new();

            for i in 0..4 {
                if i % 2 == 0 {
                    call(|| ids.insert(CallId::current()));
                } else {
                    call(|| ids.insert(CallId::current()));
                }
            }

            assert_eq!(ids.len(), 4, "each callsite must produce multiple IDs");
        });
    }

    #[test]
    fn one_child_in_a_loop() {
        call(|| {
            let root = CallId::current();
            assert_eq!(
                root,
                CallId::current(),
                "CallId must be stable across calls within the same scope"
            );

            let mut prev = root;

            for _ in 0..100 {
                let mut called = false;
                call(|| {
                    let current = CallId::current();
                    assert_ne!(prev, current, "each CallId in this loop must be unique");
                    prev = current;
                    called = true;
                });

                assert_eq!(
                    root,
                    CallId::current(),
                    "CallId must be stable across calls within the same scope"
                );

                let mut prev = root;

                for _ in 0..100 {
                    let mut called = false;
                    call(|| {
                        let current = CallId::current();
                        assert_ne!(prev, current, "each CallId in this loop must be unique");
                        prev = current;
                        called = true;
                    });

                    assert_eq!(
                        root,
                        CallId::current(),
                        "outside the call must have the same CallId as root"
                    );
                    assert!(called, "the call must be made on each loop iteration");
                }
            }
        });
    }

    #[test]
    fn reuse_same_root_two_places() {
        let dependent = || call(CallId::current);
        let independent = || root(CallId::current);

        assert_ne!(call(dependent), call(dependent));
        assert_eq!(call(independent), call(independent));
    }

    #[test]
    fn loop_over_map_with_keys_in_slots() {
        let slots = vec!["first", "second", "third", "fourth", "fifth"];

        let to_call = || {
            call(|| {
                let mut unique_ids = HashSet::new();
                for s in &slots {
                    call_in_slot(s, || {
                        let current = CallId::current();
                        unique_ids.insert(current);
                    });
                }
                assert_eq!(slots.len(), unique_ids.len(), "must be one CallId per slot");
                unique_ids
            })
        };

        let first = to_call();
        let second = to_call();
        assert_eq!(first, second, "same Ids must be produced for each slot each time");
    }

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
