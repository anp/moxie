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

#[doc(hidden)]
pub use illicit;
#[doc(inline)]
pub use topo_macro::nested;

use std::{
    any::TypeId,
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

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
        Point::with_current(|p| p.id)
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x?}", self.0))
    }
}

/// The root of a sub-graph within the overall topology formed at runtime by the call-graph of
/// topologically-nested functions.
///
/// The current `Point` contains the local [`Env`] and [`Id`].
#[doc(hiddent)]
#[derive(Debug)]
pub struct Point {
    id: Id,
    callsite: Callsite,
    /// Number of times each callsite's type has been observed during this Point.
    callsite_counts: RefCell<HashMap<Callsite, u32>>,
}

impl Point {
    /// Mark a child Point in the topology.
    #[doc(hidden)]
    pub fn unstable_enter_child<R>(
        &self,
        callsite: Callsite,
        slot: &impl Hash,
        increment_counts: bool,
        child: impl FnOnce() -> R,
    ) -> R {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        callsite.hash(&mut hasher);
        slot.hash(&mut hasher);
        let id = Id(hasher.finish());
        if increment_counts {
            *self
                .callsite_counts
                .borrow_mut()
                .entry(callsite)
                .or_default() += 1;
        }

        let child_point = Self {
            id,
            callsite,
            callsite_counts: RefCell::new(HashMap::new()),
        };

        illicit::child_env!(Point => child_point).enter(child)
    }

    /// Runs the provided closure with access to the current [`Point`].
    fn with_current<Out>(op: impl FnOnce(&Point) -> Out) -> Out {
        op(&*illicit::Env::expect::<Point>())
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            id: Id(0),
            callsite: callsite!(),
            callsite_counts: Default::default(),
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

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

    /// Returns the number of times this callsite has been seen as a child of the current Point.
    pub fn current_count(&self) -> u32 {
        Point::with_current(|point| {
            if let Some(c) = point.callsite_counts.borrow().get(self) {
                *c
            } else {
                0
            }
        })
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
            slot: callsite.current_count(),
            is_root: false,
            call: $($input)*
        )
    }};
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
    ) => {{
        $crate::illicit::Env::expect::<$crate::Point>().unstable_enter_child(
            $callsite,
            &$slot,
            $is_root,
            || $inner,
        )
    }};
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

#[cfg(test)]
mod tests {
    use {super::Id, std::collections::HashSet};

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
            let mut called = false;
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
}
