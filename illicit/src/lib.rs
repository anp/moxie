//! Type-indexed dynamically-scoped singletons, propagated through an implicit
//! context.
//!
//! # Requiring references from the environment
//!
//! The `from_env` macro provides an attribute for functions that require access
//! to a singleton in their environment. Here, the contrived function requires a
//! `u8` to add one to:
//!
//! ```
//! #[illicit::from_env(num: &u8)]
//! fn env_num_plus_one() -> u8 {
//!     num + 1
//! }
//!
//! illicit::Layer::new().offer(1u8).enter(|| {
//!     assert_eq!(env_num_plus_one(), 2u8);
//!
//!     illicit::Layer::new().offer(7u8).enter(|| {
//!         assert_eq!(env_num_plus_one(), 8u8);
//!     });
//! });
//! ```
//!
//! Here, both calls see a `u8` in their environment.
//!
//! # Caution
//!
//! This provides convenient sugar for values stored in the current environment
//! as an alternative to thread-locals or a manually propagated context object.
//! However this approach incurs a significant cost in that the following code
//! will panic without the right type having been added to the environment:
//!
//! ```should_panic
//! # #[illicit::from_env(num: &u8)]
//! # fn env_num_plus_one() -> u8 {
//! #    num + 1
//! # }
//! env_num_plus_one();
//! ```
//!
//! See the attribute's documentation for more details, and please consider
//! whether this is appropriate for your use case before taking it on as a
//! dependency.

#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

mod anon_rc;

use anon_rc::AnonRc;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    mem::replace,
    ops::Deref,
    panic::Location,
    rc::Rc,
};

#[doc(inline)]
pub use illicit_macro::from_env;

thread_local! {
    /// The current dynamic scope.
    static CURRENT_SCOPE: RefCell<Rc<Layer>> = RefCell::new(Rc::new(
        Layer {
            depth: 0,
            location: Location::caller(),
            values: Default::default(),
        }
    ));
}

/// Returns a reference to a value in the current environment if it is
/// present.
///
/// # Errors
///
/// Returns an error if the requested type is not available in the local
/// environment.
pub fn get<E>() -> Result<impl Deref<Target = E> + 'static, GetFailed>
where
    E: Any + 'static,
{
    let key = TypeId::of::<E>();
    let anon = CURRENT_SCOPE.with(|current| {
        current.borrow().values.iter().find(|(id, _)| id == &key).map(|(_, a)| a.clone())
    });
    if let Some(anon) = anon {
        Ok(anon.downcast_deref().expect("used type for storage and lookup, should match"))
    } else {
        Err(GetFailed::here::<E>())
    }
}

/// Returns a reference to a value in the current environment, as
/// [`get`] does, but panics if the value has not been set.
///
/// The panic message includes the stack of current [`Layer`]s
/// and their contents.
#[track_caller]
pub fn expect<E>() -> impl Deref<Target = E> + 'static
where
    E: Any + 'static,
{
    get().unwrap()
}

/// Removes the provided type from the current environment for the remainder
/// of its scope. Parent environments may still possess a reference to
/// the value.
pub fn hide<E: 'static>() {
    CURRENT_SCOPE.with(|current| {
        let mut env = current.borrow_mut();
        let mut without_e = env.values.clone();
        let excluded_ty = TypeId::of::<E>();
        without_e.retain(|(ty, _)| ty != &excluded_ty);
        *env = Rc::new(Layer { values: without_e, depth: env.depth, location: env.location });
    })
}

/// A container for the local environment, usually used to represent a pending
/// addition to it.
///
/// The environment is type-indexed, and access is provided through read-only
/// references. Call [`Layer::offer`] to include new values in the environment
/// for called functions and [`get`] or [`expect`] to retrieve references to the
/// offered values.
///
/// Aside: one interesting implication of the above is the ability to define
/// "private scoped global values" which are private to functions which are
/// nonetheless propagating the values with their control flow. This can be
/// useful for runtimes to offer themselves execution-local values in functions
/// which are invoked by external code. It can also be severely abused, like any
/// implicit state, and should be used with caution.
#[derive(Clone)]
pub struct Layer {
    depth: u32,
    location: &'static Location<'static>,
    values: Vec<(TypeId, AnonRc)>,
}

impl Default for Layer {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Layer {
    /// Construct a blank layer. Call [`Layer::offer`] to add values to the new
    /// layer before calling [`Layer::enter`] to run a closure with access
    /// to the new values.
    #[track_caller]
    pub fn new() -> Self {
        let mut values = Vec::new();
        let mut depth = 0;

        CURRENT_SCOPE.with(|current| {
            let current = current.borrow();
            depth = current.depth + 1;
            values = current.values.clone();
        });

        Self { values, depth, location: std::panic::Location::caller() }
    }

    /// Adds the new item and returns the modified layer.
    ///
    /// The offered type must implement `Debug` to allow [`Snapshot`]
    /// to display the contents of the illicit environment. It must also satisfy
    /// the `'static` lifetime because [`get`] is unable to express any
    /// lifetime constraints at its callsite.
    pub fn offer<E>(mut self, v: E) -> Self
    where
        E: Debug + 'static,
    {
        self.add_anon(AnonRc::new(v, self.location, self.depth));
        self
    }

    fn add_anon(&mut self, anon: AnonRc) {
        if let Some(existing) =
            self.values.iter_mut().find(|(id, _)| *id == anon.id()).map(|(_, e)| e)
        {
            *existing = anon;
        } else {
            self.values.push((anon.id(), anon));
        }
    }

    /// Call `child_fn` with this layer added to the environment.
    pub fn enter<R>(self, child_fn: impl FnOnce() -> R) -> R {
        let _reset_when_done_please = CURRENT_SCOPE.with(|parent| {
            let mut parent = parent.borrow_mut();
            let parent = replace(&mut *parent, Rc::new(self));

            scopeguard::guard(parent, move |prev| {
                CURRENT_SCOPE.with(|p| p.replace(prev));
            })
        });

        // call this out here so these calls can be nested
        child_fn()
    }
}

impl Debug for Layer {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let env_w_loc = format!("Env @ {}", self.location);

        let mut f = f.debug_struct(&env_w_loc);
        for (ty, anon) in self.values.iter().map(|(_, v)| (v.ty(), v)).collect::<BTreeMap<_, _>>() {
            f.field(ty, anon.debug());
        }
        f.finish()
    }
}

/// A point-in-time representation of the implicit environment, intended for
/// debug printing.
#[derive(Clone)]
pub struct Snapshot {
    current: Layer,
}

impl Snapshot {
    /// Returns a snapshot of the current context.
    pub fn get() -> Self {
        let mut current: Layer = CURRENT_SCOPE.with(|s| (**s.borrow()).clone());

        current.values.sort_by_key(|(_, anon)| anon.depth());

        Snapshot { current }
    }

    /// Call `child_fn` with this as its context.
    pub fn enter<R>(self, child_fn: impl FnOnce() -> R) -> R {
        self.current.enter(child_fn)
    }
}

impl Debug for Snapshot {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_list()
            .entries(self.current.values.iter().map(|(_, a)| (a.location(), a.debug())))
            .finish()
    }
}

/// A failure to find a particular type in the local context.
#[derive(Debug)]
pub struct GetFailed {
    looked_up: &'static str,
    current_snapshot: Snapshot,
}

impl GetFailed {
    fn here<E: 'static>() -> Self {
        Self { looked_up: std::any::type_name::<E>(), current_snapshot: Snapshot::get() }
    }
}

impl Display for GetFailed {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!(
            "expected a `{}` from the environment, did not find it in current env: {:#?}",
            self.looked_up, &self.current_snapshot,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_env_replaces_parent_env_values() {
        let mut first_called = false;
        let mut second_called = false;

        assert!(get::<u8>().is_err());
        Layer::new().offer(0u8).enter(|| {
            let curr_byte = *expect::<u8>();
            assert_eq!(curr_byte, 0);
            first_called = true;

            Layer::new().offer(1u8).enter(|| {
                let curr_byte = *expect::<u8>();
                assert_eq!(curr_byte, 1);
                second_called = true;
            });

            assert!(second_called);
            assert_eq!(curr_byte, 0);
        });
        assert!(first_called);
        assert!(get::<u8>().is_err());
    }

    #[test]
    fn child_sees_parent_env() {
        assert!(get::<u8>().is_err());
        Layer::new().offer(0u8).enter(|| {
            let curr_byte = *expect::<u8>();
            assert_eq!(curr_byte, 0);

            Layer::new().offer(1u16).enter(|| {
                let curr_byte = *expect::<u8>();
                assert_eq!(curr_byte, 0, "must see u8 from enclosing environment");

                let curr_uh_twobyte = *expect::<u16>();
                assert_eq!(curr_uh_twobyte, 1, "must see locally installed u16");
            });

            assert_eq!(curr_byte, 0, "must see 0");
        });
        assert!(get::<u8>().is_err());
    }

    #[test]
    fn removing_from_env() {
        assert!(get::<u8>().is_err());

        Layer::new().offer(2u8).enter(|| {
            assert_eq!(*expect::<u8>(), 2, "just added 2u8");

            Layer::new().enter(|| {
                assert_eq!(*expect::<u8>(), 2, "parent added 2u8");
                hide::<u8>();
                assert!(get::<u8>().is_err(), "just removed u8 from Env");
            });

            assert_eq!(*get::<u8>().unwrap(), 2, "returned to parent Env with 2u8");

            hide::<u8>();
            assert!(get::<u8>().is_err(), "just removed u8 from Env");
        })
    }

    #[test]
    fn failure_error() {
        let message = if let Err(e) = get::<u8>() {
            e.to_string()
        } else {
            panic!("got a u8 from the environment when we shouldn't have");
        };
        assert_eq!(
            &message,
            "expected a `u8` from the environment, did not find it in current env: []"
        );
    }
}
