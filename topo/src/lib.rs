#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! `topo` provides low-level tools for incrementally computing callgraphs.
//!
//! Each scope in this hierarchy has a unique and deterministic [crate::CallId]
//! describing that environment and the path taken to arrive at its stack frame.
//! These identifiers are derived from the path taken through the callgraph to
//! the current location, and are stable across repeated invocations of the same
//! execution paths.
//!
//! By running the same topologically-nested functions in a loop, we can observe
//! changes to the structure over time. The [moxie](https://docs.rs/moxie) crate uses these identifiers and
//! environments to create persistent trees for rendering human interfaces.
//!
//! # Making functions nested within the call topology
//!
//! Define a topologically-nested function with the `topo::nested` attribute:
//!
//! ```
//! #[topo::nested]
//! fn basic_topo() -> topo::CallId {
//!     topo::CallId::current()
//! }
//!
//! #[topo::nested]
//! fn tier_two() -> topo::CallId {
//!     basic_topo()
//! }
//!
//! // each of these functions will be run in separately identified
//! // contexts as the source locations for their calls are different
//! let first = basic_topo();
//! let second = basic_topo();
//! assert_ne!(first, second);
//!
//! let third = tier_two();
//! let fourth = tier_two();
//! assert_ne!(third, fourth);
//! assert_ne!(first, third);
//! assert_ne!(first, fourth);
//! assert_ne!(second, fourth);
//! ```

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

use std::{borrow::Borrow, cell::RefCell, hash::Hash};

mod cache;
mod id;
mod token;
pub use cache::{Cache, LocalCache, SharedCache, SharedLocalCache};
pub use id::CallId;
use id::Callsite;
pub use token::{OpaqueToken, Token};

/// Calls the provided expression with an [`CallId`] specific to the callsite.
///
/// ```
/// let prev = topo::CallId::current();
/// topo::call(|| assert_ne!(prev, topo::CallId::current()));
/// ```
#[track_caller]
pub fn call<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    let callsite = Callsite::here();
    Scope::with_current(|p| p.enter_child(callsite, &callsite.current_count(), op))
}

/// Calls the provided function as a child of [`CallId::current`], using `slot`
/// as an input for the [`CallId`] which will be current for the duration of
/// `op`.
///
/// # Example
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
    use std::collections::HashSet;

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
}
