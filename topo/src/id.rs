use super::Point;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    panic::Location,
};

/// Identifies an activation record in the current call topology.
///
/// The `Id` for the execution of a stack frame is the combined product of:
///
/// * a callsite: lexical source location at which the topologically-nested
///   function was invoked
/// * parent `Id`: the identifier which was active when entering the current
///   topo-nested function
/// * a "slot": runtime value indicating the call's "logical index" within the
///   parent call
///
/// By default, the slot used is a count of the number of times that particular
/// callsite has been executed within the parent `Id`'s enclosing scope. This
/// means that when creating an `Id` in a loop the identifier will be unique for
/// each "index" of the loop iteration and will be stable if the same loop is
/// invoked again. Changing the value used for the slot allows us to have stable
/// `Id`s across multiple executions when iterating over elements of a
/// collection that itself has unstable iteration order.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Id(u64);

impl Id {
    /// Returns the root `Id`.
    pub(crate) fn root() -> Self {
        Id(0)
    }

    /// Returns the `Id` for the current scope in the call topology.
    pub fn current() -> Self {
        Point::with_current(|current| current.id)
    }

    // TODO ensure we don't have to worry about slot hash collisions
    pub(crate) fn child<S>(&self, callsite: &Callsite, slot: S) -> Self
    where
        S: Eq + Hash,
    {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        callsite.hash(&mut hasher);
        slot.hash(&mut hasher);
        Id(hasher.finish())
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x?}", self.0))
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
