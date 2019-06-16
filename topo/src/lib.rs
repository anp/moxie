//! Topological functions execute within a context unique to the path in the runtime call
//! graph of other topological functions preceding the current activation record.
//!
//! TODO discuss creation of tree from "abstract stack frames" represented by topological
//! invocations
//!
//! TODO discuss propagating environment values down the topological call tree
//!

#[doc(hidden)]
pub extern crate tokio_trace as __trace;

pub use topo_macro::topo;

use {
    im_rc::{vector, HashMap, Vector},
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        cell::RefCell,
        hash::{Hash, Hasher},
        ops::Deref,
        sync::Arc,
    },
};

/// Retrieve a reference to a value in the current Point's environment if it has been added to
/// the environment by a parent topological invocation.
pub fn from_env<E>() -> Option<impl Deref<Target = E> + 'static>
where
    E: Any + Send + Sync + 'static,
{
    Point::__with_current(|current| {
        current
            .env
            .inner
            .get(&TypeId::of::<E>())
            .map(|guard| OwningRef::new(guard.to_owned()).map(|anon| anon.downcast_ref().unwrap()))
    })
}

/// Calls the provided expression within a `Point` bound to the callsite.
///
/// ```
/// let prev = topo::PointId::current();
/// topo::call!(|| assert_ne!(prev, topo::PointId::current()));
/// ```
///
/// Adding an `Env { ... }` directive to the macro input will take ownership of provided values
/// and make them available to the code run in the [`Point`] created by the invocation.
///
/// ```
/// #[derive(Debug, Eq, PartialEq)]
/// struct Submarine(usize);
///
/// assert!(topo::from_env::<Submarine>().is_none());
///
/// topo::call!(|| {
///     assert_eq!(&Submarine(1), &*topo::from_env::<Submarine>().unwrap());
///
///     topo::call!(|| {
///         assert_eq!(&Submarine(2), &*topo::from_env::<Submarine>().unwrap());
///     }, Env {
///         Submarine => Submarine(2)
///     });
///
///     assert_eq!(&Submarine(1), &*topo::from_env::<Submarine>().unwrap());
/// }, Env {
///     Submarine => Submarine(1)
/// });
///
/// assert!(topo::from_env::<Submarine>().is_none());
/// ```
#[macro_export]
macro_rules! call {
    ($inner:expr $(, Env {
        $($env_item_ty:ty => $env_item:expr),+
    })?) => {{
        let mut new_env = $crate::Env::default();
        $( $( new_env.__set::<$env_item_ty>($env_item); )+ )?
        $crate::Point::__enter_child($crate::__point_id!(), new_env, $inner)
    }};
}

/// Identifies a [`Point`] in the call topology. This is analogous to the hash cons of the IDs of
/// topological function invocations leading to the identified activation record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PointId(u64);

impl PointId {
    pub fn current() -> Self {
        Point::__with_current(|p| p.id())
    }
}

/// The activation record of a dynamic scope within the call topology.
#[derive(Debug)]
pub struct Point {
    path: Vector<Callsite>,
    prev_sibling: Option<Callsite>,
    env: Env,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.prev_sibling == other.prev_sibling
    }
}
impl Eq for Point {}

impl Point {
    pub fn id(&self) -> PointId {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.path.hash(&mut hasher);
        self.prev_sibling.hash(&mut hasher);
        PointId(hasher.finish())
    }

    /// Returns the `Point` identifying the current dynamic scope.
    #[inline]
    #[doc(hidden)]
    pub fn __with_current<Out>(op: impl FnOnce(&Point) -> Out) -> Out {
        __CURRENT_POINT.with(|p| op(&*p.borrow()))
    }

    #[doc(hidden)]
    pub fn __reset() {
        __CURRENT_POINT.with(|p| {
            p.borrow_mut().prev_sibling = None;
        });
    }

    /// Creates the next "link" in the chain of IDs which represents our path to the current Point.
    #[inline]
    #[doc(hidden)]
    pub fn __enter_child<T>(callsite_ty: TypeId, add_env: Env, op: impl FnOnce() -> T) -> T {
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
                env: p.env.__child(add_env),
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

#[doc(hidden)]
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

#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub struct Env {
    inner: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Env {
    #[doc(hidden)]
    pub fn __child(&self, additional: Env) -> Env {
        let mut new = self.clone();
        for (k, v) in additional.inner {
            new.inner.insert(k, v);
        }
        new
    }

    #[doc(hidden)]
    pub fn __set<E>(&mut self, e: E)
    where
        E: Any + Send + Sync + 'static,
    {
        self.inner.insert(TypeId::of::<E>(), Arc::new(e));
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
        doc ($($docs:tt)*)
    ) => {
        $($docs)*
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
        path: vector![ Callsite {  count: 1, ty: __point_id!(), } ],
        prev_sibling: None,
        env: Env::default(),
    });
}

#[allow(unused)]
fn assert_send_and_sync<T>()
where
    T: Send + Sync,
{
}

#[allow(unused)]
fn asserts() {
    assert_send_and_sync::<PointId>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        panic::{catch_unwind, AssertUnwindSafe},
    };

    fn clone(first: &Point) -> Point {
        Point {
            path: first.path.clone(),
            prev_sibling: first.prev_sibling.clone(),
            env: first.env.clone(),
        }
    }

    #[test]
    fn one_panicking_child_in_a_loop() {
        let root = Point::__with_current(clone);
        Point::__with_current(|c| assert_eq!(&root, c));

        let second_id = __point_id!();
        let prev = Point::__with_current(|c| AssertUnwindSafe(RefCell::new(clone(c))));

        Point::__with_current(|p| assert_eq!(&root, p));

        for _ in 0..100 {
            let called = AssertUnwindSafe(std::cell::Cell::new(false));
            let res = catch_unwind(|| {
                Point::__enter_child(second_id, Env::default(), || {
                    Point::__with_current(|current| {
                        assert_eq!(second_id, current.path.back().unwrap().ty);

                        assert_ne!(
                            &*prev.borrow(),
                            current,
                            "entered the same Point twice in this loop"
                        );
                        prev.replace(clone(current));
                        called.set(true);
                        panic!("checking unwind safety?");
                    });
                });
            });

            // make sure we've returned to an expected baseline
            Point::__with_current(|curr| {
                assert_eq!(root.path, curr.path);
                assert!(called.get());
                assert!(res.is_err());
            });
        }
    }

    #[test]
    fn call_env() {
        let (mut first_called, mut second_called) = (false, false);
        let (first_byte, second_byte) = (0u8, 1u8);

        call!(|| {
            let curr_byte: u8 = *from_env::<u8>().unwrap();
            assert_eq!(curr_byte, first_byte);
            first_called = true;

            call!(|| {
                let curr_byte: u8 = *from_env::<u8>().unwrap();
                assert_eq!(curr_byte, second_byte);
                second_called = true;
            }, Env {
                u8 => second_byte
            });

            assert!(second_called);
            assert_eq!(curr_byte, first_byte);
        }, Env {
            u8 => first_byte
        });
        assert!(first_called);
        assert!(from_env::<u8>().is_none());
    }
}
