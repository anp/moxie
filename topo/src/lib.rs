#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs, intra_doc_link_resolution_failure)]

//! `topo` creates a hierarchy of scoped, nested [environments][topo::Env] whose shape matches the
//! function callgraph. These environments store singletons indexed by their type, and references to
//! environmental values are available only to an enclosed call scope. When a `#![topo::nested]`
//! function is called, its parent environment is cheaply propagated along with any additional
//! values added at appropriate callsites.
//!
//! Each environment in this hierarchy has a unique and deterministic [`topo::Id`] describing that
//! environment and the path taken to arrive at its stack frame. These identifiers are derived from
//! the path taken through the callgraph to the current location, and are stable across repeated
//! invocations of the same execution paths.
//!
//! By running the same topologically-nested functions in a loop, we can observe changes to the
//! structure over time. The [moxie](https://docs.rs/moxie) crate uses these identifiers and
//! environments to create persistent trees for rendering human interfaces.
//!
//! # Making functions nested within the call topology
//!
//! Defining a topological function results in a macro definition for binding the
//! function to each callsite where it is invoked. Define a topologically-nested function with the
//! `topo::nested` attribute:
//!
//! ```
//! #[topo::nested]
//! fn basic_topo() -> topo::Id { topo::Id::current() }
//!
//! #[topo::nested]
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
//! within their immediate parent, we transform the function definition into a macro to track the
//! source location at which it is called. Future language features may make it possible to call
//! topo-nested functions without any special syntax.
//!
//! # Requiring references from the environment
//!
//! The `from_env` macro provides an attribute for functions that require access to a singleton in
//! their environment. Here, the contrived function requires a `u8` to add one to:
//!
//! ```
//! #[topo::from_env(num: &u8)]
//! fn env_num_plus_one() -> u8 {
//!     num + 1
//! }
//!
//! topo::Env::add(1u8);
//! assert_eq!(env_num_plus_one(), 2u8);
//! ```
//!
//! This provides convenient sugar for values stored in the current `Env` as an alternative to
//! thread-locals or a manually propagated context object. However this approach incurs a
//! significant cost in that the following code will panic without the right type having been added
//! to the environment:
//!
//! ```should_panic
//! # #[topo::from_env(num: &u8)]
//! # fn env_num_plus_one() -> u8 {
//! #    num + 1
//! # }
//! // thread 'main' panicked at 'expected a value from the environment, found none'
//! env_num_plus_one();
//! ```
//!
//! See the attribute's documentation for more details, and please consider whether this is
//! appropriate for your use case before taking it on as a dependency.

#[doc(inline)]
pub use topo_macro::{from_env, nested};

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

/// Calls the provided expression with an [`Id`] specific to the callsite, optionally passing
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
            slot: $crate::current_callsite_count(callsite),
            is_root: false,
            call: $($input)*
        )
    }};
}

/// Returns the number of times this callsite has been seen as a child of the current Point.
pub fn current_callsite_count(callsite: Callsite) -> u32 {
    Point::with_current(|point| {
        if let Some(c) = point.state.callsite_counts.get(&callsite) {
            *c
        } else {
            0
        }
    })
}

/// Roots a topology at a particular callsite while calling the provided expression with the same
/// convention as [`call`].
///
/// Normally, when a topological function is repeatedly invoked at the same callsite in a loop,
/// each invocation receives a different [`Id`], as these invocations model siblings in the
/// topology. The overall goal of this crate, however, is to provide imperative codepaths with
/// stable identifiers *across* executions at the same callsite. In practice, we must have a root
/// to the subtopology we are maintaining across these impure calls, and after each execution of the
/// subtopology it must reset the state at its [`Id`] so that the next execution of the root
/// is invoked at the same point under its parent as its previous execution was.
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

/// Identifies an activation record in the current call topology.
///
/// The `Id` for the execution of a stack frame is the combined product of:
///
/// * a callsite: lexical source location at which the topologically-nested function was invoked
/// * parent `Id`: the identifier which was active when entering the current topo-nested function
/// * a "slot": runtime value indicating the call's "logical index" within the parent call
///
/// By default, the slot used is a count of the number of times that particular callsite has been
/// executed within the parent `Id`'s enclosing scope. This means that when creating an `Id` in a
/// loop the identifier will be unique for each "index" of the loop iteration and will be stable if
/// the same loop is invoked again. Changing the value used for the slot allows us to have stable
/// `Id`s across multiple executions when iterating over elements of a collection that itself has
/// unstable iteration order.

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
    callsite: Callsite,
    /// The current environment.
    state: State,
}

thread_local! {
    /// The `Point` representing the current dynamic scope.
    static CURRENT_POINT: RefCell<Point> = Default::default();
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
            let parent_initial_state = parent.state.clone();

            let child = if reset_on_drop {
                let mut root = Point::default();
                // by getting a child of the state instead of the point, we skip creating
                // a dep on the IDs of the parent, but still pass an Env through
                root.state = parent.state.child(callsite, add_env);
                root
            } else {
                parent.child(callsite, slot, add_env)
            };
            let parent = replace(&mut *parent, child);

            // returned by this function, will fire when the current borrows have ended
            scopeguard::guard(
                (parent_initial_state, parent),
                move |(prev_initial_state, mut prev)| {
                    if reset_on_drop {
                        prev.state = prev_initial_state;
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
            callsite,
            state: self.state.child(callsite, additional),
        }
    }

    /// Runs the provided closure with access to the current [`Point`].
    fn with_current<Out>(op: impl FnOnce(&Point) -> Out) -> Out {
        CURRENT_POINT.with(|p| op(&*p.borrow()))
    }

    fn with_current_mut<Out>(op: impl FnOnce(&mut Point) -> Out) -> Out {
        CURRENT_POINT.with(|p| op(&mut *p.borrow_mut()))
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            id: Id(0),
            callsite: callsite!(),
            state: Default::default(),
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    /// Number of times each callsite's type has been observed during this Point.
    callsite_counts: HashMap<Callsite, u32>,
    /// The current environment.
    env: Env,
}

impl State {
    fn child(&mut self, callsite: Callsite, additional: EnvInner) -> Self {
        *self.callsite_counts.entry(callsite).or_default() += 1;

        Self {
            callsite_counts: HashMap::new(),
            env: self.env.child(additional),
        }
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
            name: std::any::type_name::<T>(),
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

type EnvInner = HashMap<TypeId, AnonRc>;

impl Env {
    /// Sets the provided value as the singleton for its type in the current environment for the
    /// remainder of the `Env`'s scope.
    pub fn add<E: 'static>(val: E) {
        Point::with_current_mut(|p| {
            p.state.env = p.state.env.child(env! {
                E => val,
            });
        });
    }

    /// Removes the provided type from the current environment for the remainder of its scope.
    /// Parent environments may still possess a reference to the value, so it is not guaranteed to
    /// be dropped, just no longer visible to this and subsequent child scopes.
    pub fn hide<E: 'static>() {
        Point::with_current_mut(|p| {
            let mut without_e: EnvInner = (*p.state.env.inner).clone();
            let excluded_ty = TypeId::of::<E>();
            without_e.retain(|ty, _| ty != &excluded_ty);

            p.state.env = Env {
                inner: Rc::new(without_e),
            };
        });
    }

    /// Returns a reference to a value in the current environment if it has been added to the
    /// environment by parent/enclosing [`call`] invocations.
    pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
    where
        E: Any + 'static,
    {
        let key = TypeId::of::<E>();
        let anon = Point::with_current(|current| current.state.env.inner.get(&key).cloned());

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
            $matcher => { topo::call!({ $mangled_name $pass }) };
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
    fn alternating_in_a_loop() {
        let mut ids = HashSet::new();

        for i in 0..4 {
            if i % 2 == 0 {
                call!(ids.insert(Id::current()));
            } else {
                call!(ids.insert(Id::current()));
            }
        }

        assert_eq!(ids.len(), 4, "each callsite must produce multiple IDs");
    }

    #[test]
    fn one_child_in_a_loop() {
        let root = Id::current();
        assert_eq!(
            root,
            Id::current(),
            "Id must be stable across calls within the same scope"
        );

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

    #[test]
    fn adding_to_and_removing_from_env() {
        assert!(
            Env::get::<u8>().is_none(),
            "test Env must not already have a u8"
        );

        Env::add(2u8);
        assert_eq!(*Env::get::<u8>().unwrap(), 2, "just added 2u8");

        call!({
            assert_eq!(*Env::get::<u8>().unwrap(), 2, "parent added 2u8");

            Env::add(7u8);
            assert_eq!(*Env::get::<u8>().unwrap(), 7, "just added 7u8");

            Env::hide::<u8>();
            assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");

            Env::add(9u8);
            assert_eq!(*Env::get::<u8>().unwrap(), 9, "just added 9u8");
        });

        assert_eq!(
            *Env::get::<u8>().unwrap(),
            2,
            "returned to parent Env with 2u8"
        );

        Env::hide::<u8>();
        assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");
    }
}
