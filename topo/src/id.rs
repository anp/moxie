use super::{
    token::{OpaqueToken, Token},
    Point,
};
use std::{hash::Hash, panic::Location};

/// Identifies an activation record in the current call topology.
///
/// The `CallId` for the execution of a stack frame is the combined product of:
///
/// * a callsite: lexical source location at which the topologically-nested
///   function was invoked
/// * parent `CallId`: the identifier which was active when entering the current
///   topo-nested function
/// * a "slot": runtime value indicating the call's "logical index" within the
///   parent call
///
/// By default, the slot used is a count of the number of times that particular
/// callsite has been executed within the parent `CallId`'s enclosing scope.
/// This means that when creating an `CallId` in a loop the identifier will be
/// unique for each "index" of the loop iteration and will be stable if the same
/// loop is invoked again. Changing the value used for the slot allows us to
/// have stable `CallId`s across multiple executions when iterating over
/// elements of a collection that itself has unstable iteration order.
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
            slot: Token::<String>::fake().opaque(),
        }
    }

    /// Returns the `CallId` for the current scope in the call topology.
    pub fn current() -> Self {
        Point::with_current(|current| current.id)
    }

    pub(crate) fn child<S>(&self, callsite: Callsite, slot: Token<S>) -> Self
    where
        S: Eq + Hash + Send + 'static,
    {
        Self { callsite, parent: Token::get(self), slot: slot.opaque() }
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
        Self {
            // the pointer value for a given location is enough to differentiate it from all others
            location: Location::caller() as *const _ as usize,
        }
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
