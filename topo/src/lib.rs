//! Topological functions execute within a context unique to the path in the runtime call
//! graph of other topological functions preceding the current activation record.
//!
//! Define a topological function with the `topo` attribute:
//!
//! ```
//! # use topo::topo;
//! #[topo]
//! fn basic_topo() -> topo::Id { topo::Id::current() }
//!
//! #[topo]
//! fn tier_two() -> topo::Id { basic_topo!() }
//!
//! // each of these functions will be run in separately identified
//! // contexts as the source locations for their calls are different
//! let first = basic_topo!();
//! let second = basic_topo!();
//! assert_ne!(first, second);
//!
//! let third = tier_two!();
//! let fourth = tier_two!();
//! assert_ne!(third, fourth);
//! assert_ne!(first, third);
//! assert_ne!(first, fourth);
//! assert_ne!(second, fourth);
//! ```
//!
//! Because topological functions must be sensitive to the location at which they're invoked and
//! bound to their parent, we transform the function definition into a macro so we can link
//! the two activation records inside macro expansion. See the docs for the attribute for more
//! detail and further discussion of the tradeoffs.
//!
//! TODO include diagram of topology
//!
//! TODO discuss creation of tree from "abstract stack frames" represented by topological
//! invocations
//!
//! TODO discuss propagating environment values down the topological call tree
//!
//! TODO show example of a rendering loop
//!

#![deny(missing_docs)]

#[doc(hidden)]
pub extern crate tokio_trace as __trace;

pub use topo_macro::topo;

use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::{hash_map::DefaultHasher, HashMap},
        hash::{Hash, Hasher},
        mem::replace,
        ops::Deref,
        rc::Rc,
    },
};

/// Returns a reference to a value in the current environment if it has been added to the
/// environment by parent/enclosing [`call`] invocations.
pub fn from_env<E>() -> Option<impl Deref<Target = E> + 'static>
where
    E: Any + 'static,
{
    Point::__with_current(|current| {
        current
            .state
            .env
            .inner
            .get(&TypeId::of::<E>())
            .map(|guard| OwningRef::new(guard.to_owned()).map(|anon| anon.downcast_ref().unwrap()))
    })
}

/// Calls the provided expression within an [`Env`] bound to the callsite.
///
/// ```
/// let prev = topo::Id::current();
/// topo::call!(|| assert_ne!(prev, topo::Id::current()));
/// ```
///
/// Adding an `Env { ... }` directive to the macro input will take ownership of provided values
/// and make them available to the code run in the `Point` created by the invocation.
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
    ($($input:tt)*) => {{
        $crate::__raw_call!(is_root: false, $($input)*)
    }}
}

/// Roots a topology at a particular callsite while calling the provided expression with the same
/// convention as [`call`].
///
/// Rooted calls reset the state at the parent [`Id`] to that of before their execution, causing
/// repeated invocations of contained topological functions to receive the same [`Id`] on each
/// execution rather than being treated as new iterations within the same overal `Point`. This is
/// particularly useful when called in a loop:
///
/// ```
/// struct LoopCount(usize);
///
/// let mut count = 0;
/// loop {
///     count += 1;
///     let mut exit = false;
///     topo::root!(|| {
///         let count = topo::from_env::<LoopCount>().unwrap().0;
///         if count == 10 {
///             exit = true;
///         }
///     }, Env {
///          LoopCount => LoopCount(count)
///     });
///
///     if exit {
///         break;
///     }
/// }
/// ```
#[macro_export]
macro_rules! root {
    ($($input:tt)*) => {{
        $crate::__raw_call!(is_root: true, $($input)*)
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! __raw_call {
    (is_root: $is_root:expr, $inner:expr $(, Env {
    $($env_item_ty:ty => $env_item:expr),+
    })?) => {{
        let mut new_env = $crate::Env::default();
        $( $( new_env.__set::<$env_item_ty>($env_item); )+ )?
        $crate::__enter_child($crate::__point_id!(), new_env, $is_root, $inner)
    }}
}

/// Creates the next "link" in the chain of IDs which represents our path to the current Point.
#[doc(hidden)]
pub fn __enter_child<T>(
    callsite_ty: TypeId,
    add_env: Env,
    is_root: bool,
    op: impl FnOnce() -> T,
) -> T {
    struct PointGuardLol {
        reset_on_drop: bool,
        prev_initial_state: Option<State>,
        prev: Option<Point>,
    }

    impl Drop for PointGuardLol {
        #[inline]
        fn drop(&mut self) {
            let mut prev = self.prev.take().unwrap();
            if self.reset_on_drop {
                prev.state = self.prev_initial_state.take().unwrap();
            }
            __CURRENT_POINT.with(|p| p.replace(prev));
        }
    }

    let _drop_when_out_of_scope_pls = __CURRENT_POINT.with(|parent| {
        let mut parent = parent.borrow_mut();

        // this must be copied *before* creating the child below, which will mutate the state
        let parent_initial_state = parent.state.clone();

        let child = parent.child(callsite_ty, add_env);

        PointGuardLol {
            reset_on_drop: is_root,
            prev_initial_state: Some(parent_initial_state),
            prev: Some(replace(&mut *parent, child)),
        }
    });
    op()
}

/// Identifies an activation record in the call topology. This is implemented approximately similar
/// to the [hash cons][cons] of preceding topological function invocations' `Id`s.
///
/// TODO explore analogies to instruction and stack pointers?
/// TODO explore more efficient implementations by piggybacking on those?
///
/// [cons]: https://en.wikipedia.org/wiki/Hash_consing
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id(u64);

impl Id {
    /// Returns the `Id` for the current scope in the call topology.
    pub fn current() -> Self {
        fn assert_send_and_sync<T>()
        where
            T: Send + Sync,
        {
        }
        assert_send_and_sync::<Id>();

        Point::__with_current(|p| p.id)
    }
}

/// The root of a sub-graph within the overall topology formed at runtime by the call-graph of
/// topological functions.
#[derive(Debug)]
struct Point {
    id: Id,
    state: State,
}

impl Point {
    /// Mark a child Point in the topology.
    fn child(&mut self, callsite_ty: TypeId, additional: Env) -> Self {
        let callsite = Callsite::new(callsite_ty, &self.state.last_child);

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.state.child_count.hash(&mut hasher);
        callsite.hash(&mut hasher);
        let id = Id(hasher.finish());

        Self {
            id,
            state: self.state.child(callsite, additional),
        }
    }

    /// Returns the `Point` identifying the current dynamic scope.
    #[doc(hidden)]
    pub fn __with_current<Out>(op: impl FnOnce(&Point) -> Out) -> Out {
        __CURRENT_POINT.with(|p| op(&*p.borrow()))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    /// The callsite most recently bound to this one as a child.
    last_child: Option<Callsite>,
    /// The number of children currently bound to this `Point`.
    child_count: u16,
    /// The current environment.
    env: Env,
}

impl State {
    fn child(&mut self, callsite: Callsite, additional: Env) -> Self {
        self.last_child = Some(callsite);
        self.child_count += 1;

        Self {
            last_child: None,
            child_count: 0,
            env: self.env.child(additional),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Callsite {
    ty: TypeId,
    count: usize,
}

impl Callsite {
    fn new(ty: TypeId, last_child: &Option<Callsite>) -> Self {
        let prev_count = match last_child {
            Some(ref prev) if prev.ty == ty => prev.count,
            _ => 0,
        };

        Self {
            ty,
            count: prev_count + 1,
        }
    }
}

///  
#[derive(Clone, Debug, Default)]
pub struct Env {
    inner: HashMap<TypeId, Rc<dyn Any>>,
}

impl Env {
    #[doc(hidden)]
    pub fn __set<E>(&mut self, e: E)
    where
        E: Any + 'static,
    {
        self.inner.insert(TypeId::of::<E>(), Rc::new(e));
    }

    fn child(&self, additional: Env) -> Env {
        let mut new = self.clone();
        for (k, v) in additional.inner {
            new.inner.insert(k, v);
        }
        new
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
    static __CURRENT_POINT: RefCell<Point> = {
        RefCell::new(Point {
            id: Id(0),
            state: Default::default(),
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        panic::{catch_unwind, AssertUnwindSafe},
    };

    fn clone(first: &Point) -> Point {
        Point {
            id: first.id,
            state: first.state.clone(),
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
            let called = AssertUnwindSafe(Cell::new(false));
            let res = catch_unwind(|| {
                __enter_child(second_id, Env::default(), false, || {
                    Point::__with_current(|current| {
                        assert_ne!(
                            prev.borrow().id,
                            current.id,
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
                assert_eq!(&root, curr);
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
