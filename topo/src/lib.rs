//! TODO write a better description.
//!
//! `use topo::*;` is necessary because we haven't worked out a nice way to pass macro names around.
pub use topo_macro::topo;

use std::{any::TypeId, cell::RefCell, hash::Hash};

/// Identifies a dynamic scope within the call topology.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    count_of_current: usize,
    callsites: im::Vector<TypeId>,
}

static_assertions::assert_impl!(pt; Point, Clone, Hash, Eq, Send, Sync);

impl Point {
    /// Returns the `Point` identifying the current dynamic scope.
    #[inline]
    pub fn current() -> Self {
        __CURRENT_POINT.with(|p| p.borrow().clone())
    }

    /// Enters the dynamic scope of the point and calls the closure provided, returning its value.
    pub fn enter<T>(self, op: impl FnOnce() -> T) -> T {
        let prev = Some(__CURRENT_POINT.with(|p| p.replace(self)));
        let _drop_when_out_of_scope_pls = PointGuardLol { prev };
        op()
    }

    /// Creates the next "link" in the chain of IDs which represents our path to the current Point.
    #[inline]
    #[doc(hidden)]
    pub fn __child(&self, callsite: TypeId) -> Self {
        let mut child = self.clone();

        if child.callsites.back() == Some(&callsite) {
            child.count_of_current += 1;
        } else {
            child.count_of_current = 0;
            child.callsites.push_back(callsite);
        }

        child
    }
}

/// Calls the provided expression within its `Point`.
///
/// ```
/// topo::call!(|| println!("{:?}", topo::Point::current()));
/// ```
#[macro_export]
macro_rules! call {
    ($inner:expr) => {
        $crate::__point!().enter($inner)
    };
}

/// Defines a new macro (named after the first metavariable) which calls a function (named in
/// the second metavariable) in a `Point` specific to this callsite and its parents.
///
/// As a quirk of the `macro_rules!` parser, we have to "bring our own" metavariables for the new
/// macro's args and their expansion for the wrapped function. This makes for an awkward invocation,
/// but it's only invoked from the proc macro attribute for generating topological macros.
///
/// This is used to work around procedural macro hygiene restrictions, allowing us to "generate" a
/// macro from a procedural macro without needing to enable a (as of writing) unstable feature.
#[doc(hidden)]
#[macro_export]
macro_rules! __make_topo_macro {
    (
        $name:ident $mangled_name:ident
        match $matcher:tt
        subst $pass:tt
    ) => {
        #[macro_export]
        macro_rules! $name { $matcher => { $crate::call!(|| $mangled_name $pass) }; }
    };
}

/// Creates and expands to a TypeId unique to the expansion site.
#[doc(hidden)]
#[macro_export]
macro_rules! __point_id {
    () => {{
        struct UwuPlsDaddyRustcGibUniqueTypeIdPlsPls; // thanks for the great name idea, cjm00!
        std::any::TypeId::of::<UwuPlsDaddyRustcGibUniqueTypeIdPlsPls>()
    }};
}

thread_local! {
    /// The `Point` representing the current dynamic scope.
    pub static __CURRENT_POINT: RefCell<Point> = RefCell::new(Point {
        count_of_current: 0,
        callsites: im::Vector::new(),
    });
}

/// Mark a `Point` in the call topology by creating a TypeId unique to the expanded location.
#[doc(hidden)]
#[macro_export]
macro_rules! __point {
    () => {
        $crate::Point::current().__child($crate::__point_id!())
    };
}

/// Resets the current Point to the one stored when the struct is dropped.
struct PointGuardLol {
    prev: Option<Point>,
}

impl Drop for PointGuardLol {
    #[inline]
    fn drop(&mut self) {
        __CURRENT_POINT.with(|p| p.replace(self.prev.take().unwrap()));
    }
}
