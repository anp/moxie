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
//! ```
//! # use assert_panic::assert_panic;
//! # #[illicit::from_env(num: &u8)]
//! # fn env_num_plus_one() -> u8 {
//! #    num + 1
//! # }
//! assert_panic!(
//!     { env_num_plus_one(); },
//!     String,
//!     starts with "expected a `u8` from the environment",
//! );
//! ```
//!
//! See the attribute's documentation for more details, and please consider
//! whether this is appropriate for your use case before taking it on as a
//! dependency.

#![feature(track_caller)]
#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

mod anon_rc;

use anon_rc::AnonRc;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
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
pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
where
    E: Any + 'static,
{
    let key = TypeId::of::<E>();
    let anon = CURRENT_SCOPE.with(|current| {
        current.borrow().values.iter().find(|(id, _)| id == &key).map(|(_, a)| a.clone())
    });
    if let Some(anon) = anon {
        Some(anon.downcast_deref().expect("used type for storage and lookup, should match"))
    } else {
        None
    }
}

/// Returns a reference to a value in the current environment, as
/// [`get`] does, but panics if the value has not been set.
///
/// The panic message includes the stack of current [`Layer`]s
/// and their contents.
pub fn expect<E>() -> impl Deref<Target = E> + 'static
where
    E: Any + 'static,
{
    if let Some(val) = get() {
        val
    } else {
        panic!(
            "expected a `{}` from the environment, did not find it in current env: {:#?}",
            std::any::type_name::<E>(),
            Snapshot::get(),
        )
    }
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

/// A pending addition to the local environment.
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

    // TODO(#135) remove
    #[cfg(test)]
    fn raw_location(&self) -> (&'static str, u32, u32) {
        (self.location.file(), self.location.line(), self.location.column())
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
    by_depth: BTreeMap<u32, Layer>,
}

impl Snapshot {
    /// Returns a snapshot of the current dynamic scope. Most useful for
    /// debugging the contained `Layer`s.
    pub fn get() -> Snapshot {
        let mut stacked = Snapshot { by_depth: BTreeMap::new() };
        CURRENT_SCOPE.with(|e| {
            for (_, anon) in &e.borrow().values {
                stacked
                    .by_depth
                    .entry(anon.depth())
                    .or_insert_with(|| Layer {
                        values: Default::default(),
                        depth: anon.depth(),
                        location: anon.location(), // depth -> location is 1:1
                    })
                    .add_anon(anon.clone());
            }
        });
        stacked
    }
}

impl Debug for Snapshot {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_list().entries(self.by_depth.values()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_correct_structure() {
        let empty_scope = Snapshot::get();
        let mut u8_present = (empty_scope.clone(), 0);
        let mut u8_and_string_present = u8_present.clone();

        // generate our test values
        Layer::new().offer(42u8).enter(|| {
            u8_present = (Snapshot::get(), line!() - 1);

            Layer::new().offer(String::from("owo")).enter(|| {
                u8_and_string_present = (Snapshot::get(), line!() - 1);
            });
        });

        // check the empty scopes
        let empty_scope_after = Snapshot::get();
        assert_eq!(empty_scope.by_depth.len(), 0, "environment should be empty without additions");
        assert_eq!(
            empty_scope.by_depth.len(),
            empty_scope_after.by_depth.len(),
            "snapshots must be the same before and after calls"
        );

        // our generates scopes must have
        assert_eq!(u8_present.0.by_depth.len(), 1);
        assert_eq!(u8_and_string_present.0.by_depth.len(), 2);

        let (_, first_layer) = u8_present.0.by_depth.into_iter().next().unwrap();
        assert_eq!(first_layer.depth, 1);
        assert_eq!(first_layer.raw_location(), (file!(), u8_present.1, 9));
        assert_eq!(first_layer.values.len(), 1);
        let first_u8 = first_layer
            .values
            .iter()
            .find(|(id, _)| id == &TypeId::of::<u8>())
            .map(|(_, v)| v)
            .unwrap();
        assert_eq!(
            (first_u8.raw_location(), first_u8.depth()),
            (first_layer.raw_location(), first_layer.depth),
        );

        // on to the snapshot that includes both u8 and string
        let mut u8_and_string_layers = u8_and_string_present.0.by_depth.values();
        let (first_layer, second_layer) =
            (u8_and_string_layers.next().unwrap(), u8_and_string_layers.next().unwrap());

        assert_eq!(first_layer.depth, 1);
        assert_eq!(first_layer.raw_location(), (file!(), u8_present.1, 9));
        assert_eq!(first_layer.values.len(), 1);
        let first_u8 = first_layer
            .values
            .iter()
            .find(|(id, _)| id == &TypeId::of::<u8>())
            .map(|(_, v)| v)
            .unwrap();
        assert_eq!(
            (first_u8.raw_location(), first_u8.depth()),
            (first_layer.raw_location(), first_layer.depth),
        );

        assert_eq!(second_layer.depth, 2);
        assert_eq!(second_layer.raw_location(), (file!(), u8_and_string_present.1, 13));
        assert_eq!(second_layer.values.len(), 1);
        let second_string = second_layer
            .values
            .iter()
            .find(|(id, _)| id == &TypeId::of::<String>())
            .map(|(_, v)| v)
            .unwrap();
        assert_eq!(
            (second_string.raw_location(), second_string.depth()),
            (second_layer.raw_location(), second_layer.depth),
        );
    }

    #[test]
    fn child_env_replaces_parent_env_values() {
        let mut first_called = false;
        let mut second_called = false;

        assert!(get::<u8>().is_none());
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
        assert!(get::<u8>().is_none());
    }

    #[test]
    fn child_sees_parent_env() {
        assert!(get::<u8>().is_none());
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
        assert!(get::<u8>().is_none());
    }

    #[test]
    fn removing_from_env() {
        assert!(get::<u8>().is_none());

        Layer::new().offer(2u8).enter(|| {
            assert_eq!(*expect::<u8>(), 2, "just added 2u8");

            Layer::new().enter(|| {
                assert_eq!(*expect::<u8>(), 2, "parent added 2u8");
                hide::<u8>();
                assert!(get::<u8>().is_none(), "just removed u8 from Env");
            });

            assert_eq!(*get::<u8>().unwrap(), 2, "returned to parent Env with 2u8");

            hide::<u8>();
            assert!(get::<u8>().is_none(), "just removed u8 from Env");
        })
    }
}
