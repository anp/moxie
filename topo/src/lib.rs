//! TODO write a better description.
pub use topo_macro::topo;

pub mod env;

use std::{
    any::TypeId,
    cell::RefCell,
    hash::{Hash, Hasher},
};

/// Calls the provided expression within a `Point` bound to the callsite.
///
/// ```
/// topo::call!(|| println!("{:?}", topo::Point::current()));
/// ```
#[macro_export]
macro_rules! call {
    ($inner:expr $(, Env {
        $($env_item_ty:ty => $env_item:expr),+
    })?) => {{
        let mut new_env = $crate::env::Env::default();
        $( $( new_env.__set::<$env_item_ty>($env_item); )+ )?
        $crate::Point::__enter_child($crate::__point_id!(), new_env, $inner)
    }};
}

/// Identifies a dynamic scope within the call topology.
#[derive(Clone, Debug)]
pub struct Point {
    path: im::Vector<Callsite>,
    prev_sibling: Option<Callsite>,
    env: env::Env,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.prev_sibling == other.prev_sibling
    }
}
impl Eq for Point {}

impl Hash for Point {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.path.hash(h);
        self.prev_sibling.hash(h);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Callsite {
    ty: TypeId,
    count: usize,
}

impl Callsite {
    fn new(ty: TypeId, prev_sibling: &Option<Callsite>) -> Self {
        let prev_count = match prev_sibling {
            Some(ref prev) if prev.ty == ty => prev.count,
            _ => 0,
        };

        Self {
            ty,
            count: prev_count + 1,
        }
    }
}

static_assertions::assert_impl!(pt; Point, Clone, Hash, Eq, Send, Sync);

impl Point {
    /// Returns the `Point` identifying the current dynamic scope.
    #[inline]
    #[doc(hidden)]
    pub fn current() -> Self {
        __CURRENT_POINT.with(|p| p.borrow().clone())
    }

    #[doc(hidden)]
    pub fn __flush() {
        __CURRENT_POINT.with(|p| {
            p.borrow_mut().prev_sibling = None;
        });
    }

    /// Creates the next "link" in the chain of IDs which represents our path to the current Point.
    #[inline]
    #[doc(hidden)]
    pub fn __enter_child<T>(callsite_ty: TypeId, add_env: env::Env, op: impl FnOnce() -> T) -> T {
        struct PointGuardLol {
            prev: Option<Point>,
        }

        impl Drop for PointGuardLol {
            #[inline]
            fn drop(&mut self) {
                __CURRENT_POINT.with(|p| p.replace(self.prev.take().unwrap()));
            }
        }

        let _drop_when_out_of_scope_pls = __CURRENT_POINT.with(|p| {
            let mut p = p.borrow_mut();
            let current = Callsite::new(callsite_ty, &p.prev_sibling);
            let mut path = p.path.clone();
            path.push_back(current);

            let child = Self {
                env: p.env.child(add_env),
                path,
                prev_sibling: None,
            };
            p.prev_sibling = Some(current);

            let prev = Some(std::mem::replace(&mut *p, child));
            PointGuardLol { prev }
        });
        op()
    }
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
        struct UwuDaddyRustcGibUniqueTypeIdPlsPls; // thanks for the great name idea, cjm00!
        std::any::TypeId::of::<UwuDaddyRustcGibUniqueTypeIdPlsPls>()
    }};
}

thread_local! {
    /// The `Point` representing the current dynamic scope.
    pub static __CURRENT_POINT: RefCell<Point> = RefCell::new(Point {
        path: im::vector![ Callsite {  count: 1, ty: __point_id!(), } ],
        prev_sibling: None,
        env: env::Env::default(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        panic::{catch_unwind, AssertUnwindSafe},
    };

    #[test]
    fn one_panicking_child_in_a_loop() {
        let root = Point::current();
        assert_eq!(root, Point::current());

        let second_id = __point_id!();
        let prev = AssertUnwindSafe(RefCell::new(Point::current()));

        assert_eq!(root, Point::current());

        for _ in 0..100 {
            let called = AssertUnwindSafe(std::cell::Cell::new(false));
            let res = catch_unwind(|| {
                Point::__enter_child(second_id, env::Env::default(), || {
                    assert_eq!(second_id, Point::current().path.back().unwrap().ty);
                    assert_ne!(
                        &*prev.borrow(),
                        &Point::current(),
                        "entered the same Point twice in this loop"
                    );
                    prev.replace(Point::current());
                    called.set(true);
                    panic!("checking unwind safety?");
                });
            });

            // make sure we've returned to an expected baseline
            let curr = Point::current();
            assert_eq!(root.path, curr.path);
            assert!(called.get());
            assert!(res.is_err());
        }
    }
}
