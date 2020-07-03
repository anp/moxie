#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! `topo` provides low-level tools for incrementally computing callgraphs.
//!
//! Each scope in this hierarchy has a unique and deterministic [crate::Id]
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
//! fn basic_topo() -> topo::Id {
//!     topo::Id::current()
//! }
//!
//! #[topo::nested]
//! fn tier_two() -> topo::Id {
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

#[doc(hidden)]
pub use illicit;
#[doc(inline)]
pub use topo_macro::nested;

use std::{cell::RefCell, hash::Hash};

mod cache;
mod id;
mod token;
pub use cache::{Cache, CacheHandle, LocalCache, LocalCacheHandle};
pub use id::{Callsite, Id};
pub use token::{OpaqueToken, Token};

/// Calls the provided expression with an [`Id`] specific to the callsite.
///
/// ```
/// let prev = topo::Id::current();
/// topo::call(|| assert_ne!(prev, topo::Id::current()));
/// ```
#[track_caller]
pub fn call<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    let callsite = Callsite::here();
    let slot = Token::get(&callsite.current_count());
    Point::with_current(|p| p.enter_child(callsite, slot, op))
}

/// The default "slot" for a topo call is the number of times that callsite
/// has executed. You can override that by providing an arbitrary slot in
/// this call.
#[track_caller]
pub fn call_in_slot<F, R, S>(slot: Token<S>, op: F) -> R
where
    F: FnOnce() -> R,
    S: Eq + Hash + Send + 'static,
{
    Point::with_current(|p| p.enter_child(Callsite::here(), slot, op))
}

/// Calls the provided expression as the root of a new call tree, ignoring the
/// current `Id`.
///
/// # Example
///
/// ```
/// // a call to root() here ensures the child is always treated as the same tree
/// // no matter from where the function is called
/// let independent = || topo::root(topo::Id::current);
/// assert_eq!(topo::call(independent), topo::call(independent));
///
/// // this is a normal topo call, it returns `Id`s based on the parent state
/// let dependent = || topo::call(topo::Id::current);
/// assert_ne!(topo::call(dependent), topo::call(dependent));
/// ```
pub fn root<F, R>(op: F) -> R
where
    F: FnOnce() -> R,
{
    illicit::hide::<Point>();
    call(op)
}

/// The root of a sub-graph within the overall topology formed at runtime by the
/// call-graph of topologically-nested functions.
///
/// The current `Point` contains the local [`Id`] and a count of how often each
/// of its children has been called.
#[derive(Debug)]
struct Point {
    id: Id,
    callsite: Callsite,
    /// Number of times each callsite's type has been observed during this
    /// Point.
    callsite_counts: RefCell<Vec<(Callsite, u32)>>,
}

impl Point {
    /// Mark a child Point in the topology, calling `child` within it.
    fn enter_child<C, R, S>(&self, callsite: Callsite, slot: Token<S>, child: C) -> R
    where
        C: FnOnce() -> R,
        S: Eq + Hash + Send + 'static,
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
        F: FnOnce(&Point) -> Out,
    {
        if let Ok(current) = illicit::get::<Point>() {
            op(&*current)
        } else {
            op(&Point::default())
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

impl Default for Point {
    fn default() -> Self {
        Self { id: Id::root(), callsite: Callsite::here(), callsite_counts: Default::default() }
    }
}

impl PartialEq for Point {
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
                    call(|| ids.insert(Id::current()));
                } else {
                    call(|| ids.insert(Id::current()));
                }
            }

            assert_eq!(ids.len(), 4, "each callsite must produce multiple IDs");
        });
    }

    #[test]
    fn one_child_in_a_loop() {
        call(|| {
            let root = Id::current();
            assert_eq!(root, Id::current(), "Id must be stable across calls within the same scope");

            let mut prev = root;

            for _ in 0..100 {
                let mut called = false;
                call(|| {
                    let current = Id::current();
                    assert_ne!(prev, current, "each Id in this loop must be unique");
                    prev = current;
                    called = true;
                });

                assert_eq!(
                    root,
                    Id::current(),
                    "Id must be stable across calls within the same scope"
                );

                let mut prev = root;

                for _ in 0..100 {
                    let mut called = false;
                    call(|| {
                        let current = Id::current();
                        assert_ne!(prev, current, "each Id in this loop must be unique");
                        prev = current;
                        called = true;
                    });

                    assert_eq!(
                        root,
                        Id::current(),
                        "outside the call must have the same Id as root"
                    );
                    assert!(called, "the call must be made on each loop iteration");
                }
            }
        });
    }

    #[test]
    fn reuse_same_root_two_places() {
        let dependent = || call(Id::current);
        let independent = || root(Id::current);

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
                    call_in_slot(Token::get(s), || {
                        let current = Id::current();
                        unique_ids.insert(current);
                    });
                }
                assert_eq!(slots.len(), unique_ids.len(), "must be one Id per slot");
                unique_ids
            })
        };

        let first = to_call();
        let second = to_call();
        assert_eq!(first, second, "same Ids must be produced for each slot each time");
    }
}
