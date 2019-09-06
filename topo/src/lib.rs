#![deny(missing_docs, intra_doc_link_resolution_failure, unsafe_code)]

//! `topo` provides tools for describing trees based on their runtime callgraph. Because normal
//! synchronous control flow has a tree(ish)-shaped callgraph, this can be quite natural.
//!
//! Topologically-bound functions run within a context unique to the path in the runtime call
//! graph of other topological functions preceding the current activation record.
//!
//! By running the same topologically-bound functions in a loop, we can observe changes to the
//! structure over time.
//!
//! Defining a topological function results in a macro definition for binding the
//! function to each callsite where it is invoked.
//!
//! Define a topologically-bound function with the `topo::bound` attribute:
//!
//! ```
//! #[topo::bound]
//! fn basic_topo() -> topo::Id { topo::Id::current() }
//!
//! #[topo::bound]
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

pub use topo_macro::bound;

use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        cell::{Cell, RefCell},
        collections::{hash_map::DefaultHasher, HashMap as Map},
        hash::{Hash, Hasher},
        mem::replace,
        ops::Deref,
        rc::Rc,
    },
};

/// A value unique to the source location where it is created.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Callsite {
    ty: TypeId,
}

impl Callsite {
    #[doc(hidden)]
    pub fn new(ty: TypeId) -> Self {
        Self { ty }
    }
}

/// Returns a value unique to the point of its invocation.
#[macro_export]
macro_rules! callsite {
    () => {{
        struct UwuDaddyRustcGibUniqueTypeIdPlsPls; // thanks for the great name idea, cjm00!
        $crate::Callsite::new(std::any::TypeId::of::<UwuDaddyRustcGibUniqueTypeIdPlsPls>())
    }};
}

/// Calls the provided expression within an [`Env`] bound to the callsite, optionally passing
/// additional environment values to the child scope.
///
/// ```
/// let prev = topo::Id::current();
/// topo::call!(assert_ne!(prev, topo::Id::current()));
/// ```
///
/// Adding an `env! { ... }` directive to the macro input will take ownership of provided values
/// and make them available to the code run in the `Point` created by the invocation.
///
/// ```
/// # use topo;
/// #[derive(Debug, Eq, PartialEq)]
/// struct Submarine(usize);
///
/// assert!(topo::Env::get::<Submarine>().is_none());
///
/// topo::call!({
///     assert_eq!(&Submarine(1), &*topo::Env::get::<Submarine>().unwrap());
///
///     topo::call!({
///         assert_eq!(&Submarine(2), &*topo::Env::get::<Submarine>().unwrap());
///     }, env! {
///         Submarine => Submarine(2),
///     });
///
///     assert_eq!(&Submarine(1), &*topo::Env::get::<Submarine>().unwrap());
/// }, env! {
///     Submarine => Submarine(1),
/// });
///
/// assert!(topo::Env::get::<Submarine>().is_none());
/// ```
#[macro_export]
macro_rules! call {
    (slot: $slot:expr, $($input:tt)*) => {{
        $crate::unstable_raw_call!(
            callsite: $crate::callsite!(),
            slot: $slot,
            is_root: false,
            call: $($input)*
        )
    }};
    ($($input:tt)*) => {{
        let callsite = $crate::callsite!();
        $crate::unstable_raw_call!(
            callsite: callsite,
            slot: $crate::next_iter_count(callsite),
            is_root: false,
            call: $($input)*
        )
    }};
}

/// Roots a topology at a particular callsite while calling the provided expression with the same
/// convention as [`call`].
///
/// Normally, when a topological function is repeatedly bound to the same callsite in a loop,
/// each invocation receives a different [`Id`], as these invocations model siblings in the
/// topology. The overall goal of this crate, however, is to provide imperative codepaths with
/// stable identifiers *across* executions at the same callsite. In practice, we must have a root
/// to the subtopology we are maintaining across these impure calls, and after each execution of the
/// subtopology it must reset the state at its [`Id`] so that the next execution of the root
/// is bound to the same point at its parent as its previous execution was. This is...an opaque
/// explanation at best and TODO revise it.
///
/// In this first example, a scope containing the loop can observe each separate loop
/// iteration mutating `count` and the root closure mutating `exit`. The variables `root_ids` and
/// `child_ids` observe the identifiers of the
///
/// ```
/// # use topo::{self, *};
/// # use std::collections::{HashMap, HashSet};
/// struct LoopCount(usize);
///
/// let mut count = 0;
/// let mut exit = false;
/// let mut root_ids = HashSet::new();
/// let mut child_ids = HashMap::new();
/// while !exit {
///     count += 1;
///     topo::root!({
///         root_ids.insert(topo::Id::current());
///         assert_eq!(
///             root_ids.len(),
///             1,
///             "the Id of this scope should be repeated, not incremented"
///         );
///
///         let outer_count = topo::Env::get::<LoopCount>().unwrap().0;
///         assert!(outer_count <= 10);
///         if outer_count == 10 {
///             exit = true;
///         }
///
///         for i in 0..10 {
///             topo::call!({
///                 let current_id = topo::Id::current();
///                 if outer_count > 1 {
///                     assert_eq!(child_ids[&i], current_id);
///                 }
///                 child_ids.insert(i, current_id);
///                 assert!(
///                     child_ids.len() <= 10,
///                     "only 10 children should be observed across all loop iterations",
///                 );
///             });
///         }
///         assert_eq!(child_ids.len(), 10);
///     }, env! {
///          LoopCount => LoopCount(count),
///     });
///     assert_eq!(child_ids.len(), 10);
///     assert_eq!(root_ids.len(), 1);
/// }
/// ```
#[macro_export]
macro_rules! root {
    ($($input:tt)*) => {{
        $crate::unstable_raw_call!(
            callsite: $crate::callsite!(),
            slot: (),
            is_root: true,
            call: $($input)*
        )
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! unstable_raw_call {
    (
        callsite: $callsite:expr,
        slot: $slot:expr,
        is_root: $is_root:expr,
        call: $inner:expr
        $(, env! { $($env:tt)* })?
    ) => {{
        #[allow(unused_mut)]
        let mut _new_env = Default::default();
        $( _new_env = $crate::env! { $($env)* };  )?

        let _reset_to_parent_on_drop_pls = $crate::Point::unstable_enter_child(
                $callsite,
                &$slot,
                _new_env,
                $is_root
        );

        $inner
    }};
}

/// Identifies an activation record in the call topology. This is implemented approximately similar
/// to the [hash cons][cons] of preceding topological function invocations' `Id`s.
///
/// TODO explore analogies to instruction and stack pointers?
/// TODO explore more efficient implementations by piggybacking on those?
///
/// [cons]: https://en.wikipedia.org/wiki/Hash_consing
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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
        CURRENT_POINT.with(|p| p.borrow().id)
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x?}", self.0))
    }
}

/// The root of a sub-graph within the overall topology formed at runtime by the call-graph of
/// topological functions.
///
/// The current `Point` contains the local [`Env`] and [`Id`].
#[derive(Debug)]
pub struct Point {
    id: Id,
    /// The current environment.
    env: Env,
}

thread_local! {
    /// The `Point` representing the current dynamic scope.
    static CURRENT_POINT: RefCell<Point> = Default::default();
    static LAST_CALLSITE_AND_COUNT: Cell<(Callsite, u32)> = Cell::new((callsite!(), 0));
}

#[doc(hidden)]
pub fn next_iter_count(callsite: Callsite) -> u32 {
    LAST_CALLSITE_AND_COUNT.with(|last| {
        let previous = last.replace((callsite, 1));
        if previous.0 == callsite {
            let new_count = previous.1 + 1;
            last.set((callsite, new_count));
            new_count
        } else {
            1
        }
    })
}

impl Point {
    /// "Root" a new child [`Point`]. When the guard returned from this function is dropped, the
    /// parent point is restored as the "current" `Point`. By calling provided code while the
    /// returned guard is live on the stack, we create the tree of indices and environments that
    /// correspond to the topological call tree, exiting the child context when the rooted scope
    /// ends.
    #[doc(hidden)]
    pub fn unstable_enter_child(
        callsite: Callsite,
        slot: &impl Hash,
        add_env: EnvInner,
        reset_on_drop: bool,
    ) -> impl Drop {
        CURRENT_POINT.with(|parent| {
            let mut parent = parent.borrow_mut();

            // this must be copied *before* creating the child below, which will mutate the state
            let parent_initial_env = parent.env.clone();

            let child = if reset_on_drop {
                let mut root = Point::default();
                // by getting a child of the state instead of the point, we skip creating
                // a dep on the IDs of the parent, but still pass an Env through
                root.env = parent.env.child(add_env);
                root
            } else {
                parent.child(callsite, slot, add_env)
            };
            let parent = replace(&mut *parent, child);

            scopeguard::guard(
                (parent_initial_env, parent),
                move |(prev_initial_env, mut prev)| {
                    if reset_on_drop {
                        prev.env = prev_initial_env;
                        LAST_CALLSITE_AND_COUNT.with(|last| last.set((callsite!(), 1)));
                    }
                    CURRENT_POINT.with(|p| p.replace(prev));
                },
            )
        })
    }

    /// Mark a child Point in the topology.
    fn child(&mut self, callsite: Callsite, slot: &impl Hash, additional: EnvInner) -> Self {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        callsite.hash(&mut hasher);
        slot.hash(&mut hasher);
        let id = Id(hasher.finish());

        Self {
            id,
            env: self.env.child(additional),
        }
    }

    /// Runs the provided closure with access to the current [`Point`].
    fn with_current<Out>(op: impl FnOnce(&Point) -> Out) -> Out {
        CURRENT_POINT.with(|p| op(&*p.borrow()))
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            id: Id(0),
            env: Default::default(),
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Immutable environment container for the current (sub)topology. Environment values can be
/// provided by parent topological invocations (currently just with [`call`] and
/// [`root`]), but child functions can only mutate their environment through interior
/// mutability.
///
/// The environment is type-indexed/type-directed, and each `Env` holds 0-1 instances
/// of every [`std::any::Any`]` + 'static` type. Access is provided through read-only references.
///
/// Aside: one interesting implication of the above is the ability to define "private scoped global
/// values" which are private to functions which are nonetheless propagating the values with
/// their control flow. This can be useful for runtimes to offer themselves execution-local values
/// in functions which are invoked by external code. It can also be severely abused, like any
/// implicit state, and should be used with caution.
#[derive(Clone, Debug, Default)]
pub struct Env {
    inner: Rc<EnvInner>,
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct AnonRc {
    name: &'static str,
    id: TypeId,
    inner: Rc<dyn Any>,
}

impl AnonRc {
    #[doc(hidden)]
    pub fn unstable_new<T: 'static>(inner: T) -> Self {
        Self {
            name: "TODO",
            id: TypeId::of::<T>(),
            inner: Rc::new(inner),
        }
    }

    #[doc(hidden)]
    pub fn unstable_insert_into(self, env: &mut EnvInner) {
        env.insert(self.id, self);
    }

    #[doc(hidden)]
    // FIXME this should probably expose a fallible api somehow?
    pub fn unstable_deref<T: 'static>(self) -> impl Deref<Target = T> + 'static {
        OwningRef::new(self.inner).map(|anon| {
            anon.downcast_ref().unwrap_or_else(|| {
                panic!("asked {:?} to cast to {:?}", anon, TypeId::of::<T>(),);
            })
        })
    }
}

impl Deref for AnonRc {
    type Target = dyn Any;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

type EnvInner = Map<TypeId, AnonRc>;

impl Env {
    /// Returns a reference to a value in the current environment if it has been added to the
    /// environment by parent/enclosing [`call`] invocations.
    pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
    where
        E: Any + 'static,
    {
        let key = TypeId::of::<E>();
        let anon = Point::with_current(|current| current.env.inner.get(&key).cloned());

        if let Some(anon) = anon {
            Some(anon.unstable_deref())
        } else {
            None
        }
    }

    /// Returns a reference to a value in the current environment, as [`Env::get`] does, but panics
    /// if the value has not been set in the environment.
    // TODO typename for debugging here would be v. nice
    pub fn expect<E>() -> impl Deref<Target = E> + 'static
    where
        E: Any + 'static,
    {
        Self::get().expect("expected a value from the environment, found none")
    }

    fn child(&self, additional: EnvInner) -> Env {
        let mut new: EnvInner = (*self.inner).clone();
        new.extend(additional.into_iter());

        Env {
            inner: Rc::new(new),
        }
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
macro_rules! unstable_make_topo_macro {
    (
        $name:ident $mangled_name:ident
        match $matcher:tt
        subst $pass:tt
        doc ($($docs:tt)*)
    ) => {
        $($docs)*
        #[macro_export]
        macro_rules! $name {
            $matcher => {{
                let callsite = topo::callsite!();
                topo::unstable_raw_call!(
                    callsite: callsite,
                    slot: topo::next_iter_count(callsite),
                    is_root: false,
                    call: $mangled_name $pass)
            }};
        }
    };
}

/// Declare additional environment values to expose to a child topological function's call tree.
#[macro_export]
macro_rules! env {
    ($($env_item_ty:ty => $env_item:expr,)*) => {{
        #[allow(unused_mut)]
        let mut new_env = std::collections::HashMap::new();
        $( $crate::AnonRc::unstable_new($env_item).unstable_insert_into(&mut new_env); )*
        new_env
    }}
}

#[cfg(test)]
mod tests {
    use {
        super::{Env, Id},
        std::collections::HashSet,
    };

    #[test]
    fn one_child_in_a_loop() {
        let root = Id::current();
        assert_eq!(root, Id::current(), "Id must be stable across calls");

        let mut prev = root;

        for _ in 0..100 {
            let called;
            call!({
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

    #[test]
    fn loop_over_map_with_keys_in_slots() {
        let slots = vec!["first", "second", "third", "fourth", "fifth"];

        let to_call = || {
            root!({
                let mut unique_ids = HashSet::new();
                for s in &slots {
                    call!(slot: s, {
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
        assert_eq!(
            first, second,
            "same Ids must be produced for each slot each time"
        );
    }

    #[test]
    fn call_env() {
        let first_called;
        let second_called;

        assert!(Env::get::<u8>().is_none());
        call!(
            {
                let curr_byte = *Env::expect::<u8>();
                assert_eq!(curr_byte, 0);
                first_called = true;

                call!(
                    {
                        let curr_byte = *Env::expect::<u8>();
                        assert_eq!(curr_byte, 1);
                        second_called = true;
                    },
                    env! {
                        u8 => 1u8,
                    }
                );

                assert!(second_called);
                assert_eq!(curr_byte, 0);
            },
            env! {
                u8 => 0u8,
            }
        );
        assert!(first_called);
        assert!(Env::get::<u8>().is_none());
    }

    #[test]
    fn root_sees_parent_env() {
        assert!(Env::get::<u8>().is_none());
        call!(
            {
                let curr_byte = *Env::expect::<u8>();
                assert_eq!(curr_byte, 0);

                root!(
                    {
                        let curr_byte = *Env::expect::<u8>();
                        assert_eq!(curr_byte, 0, "must see u8 from enclosing environment");

                        let curr_uh_twobyte = *Env::expect::<u16>();
                        assert_eq!(curr_uh_twobyte, 1, "must see locally installed u16");
                    },
                    env! {
                        u16 => 1u16,
                    }
                );

                assert_eq!(curr_byte, 0, "must see 0");
            },
            env! {
                u8 => 0u8,
            }
        );
        assert!(Env::get::<u8>().is_none());
    }
}
