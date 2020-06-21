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
//! illicit::Layer::new().with(1u8).enter(|| {
//!     assert_eq!(env_num_plus_one(), 2u8);
//! });
//! ```
//!
//! This provides convenient sugar for values stored in the current `Env` as an
//! alternative to thread-locals or a manually propagated context object.
//! However this approach incurs a significant cost in that the following code
//! will panic without the right type having been added to the environment:
//!
//! ```should_panic
//! # #[illicit::from_env(num: &u8)]
//! # fn env_num_plus_one() -> u8 {
//! #    num + 1
//! # }
//! // thread 'main' panicked at 'expected a value from the environment, found none'
//! env_num_plus_one();
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

/// Returns a snapshot of the current dynamic scope. Most useful for debugging
/// the contained `Layer`s.
pub fn snapshot() -> EnvSnapshot {
    let mut stacked = EnvSnapshot { by_depth: BTreeMap::new() };
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
            snapshot(),
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

/// Immutable environment container for the current scope. Environment values
/// can be provided by parent environments, but child functions can only mutate
/// their environment through interior mutability.
///
/// The environment is type-indexed, and each `Scope` holds 0-1 references to
/// every `[std::any::Any] + 'static` type. Access is provided through read-only
/// references.
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
    /// Construct a blank layer. Call [`Layer::with`] to add values to the new
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
    pub fn with<E>(mut self, v: E) -> Self
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

    /// The number of parent environments from which this environment descends.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    // TODO(#135) remove
    #[cfg(test)]
    fn raw_location(&self) -> (&'static str, u32, u32) {
        (self.location.file(), self.location.line(), self.location.column())
    }

    /// Call `child_fn` with this environment as the current scope.
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

/// An alternative representation of the current scope's environment, optimized
/// for debug printing.
#[derive(Clone)]
pub struct EnvSnapshot {
    by_depth: BTreeMap<u32, Layer>,
}

impl Debug for EnvSnapshot {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_list().entries(self.by_depth.values()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_correct_structure() {
        let empty_scope = snapshot();
        let mut u8_present = (empty_scope.clone(), 0);
        let mut u8_and_string_present = u8_present.clone();

        // generate our test values
        Layer::new().with(42u8).enter(|| {
            u8_present = (snapshot(), line!() - 1);

            Layer::new().with(String::from("owo")).enter(|| {
                u8_and_string_present = (snapshot(), line!() - 1);
            });
        });

        // check the empty scopes
        let empty_scope_after = snapshot();
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
        Layer::new().with(0u8).enter(|| {
            let curr_byte = *expect::<u8>();
            assert_eq!(curr_byte, 0);
            first_called = true;

            Layer::new().with(1u8).enter(|| {
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
        Layer::new().with(0u8).enter(|| {
            let curr_byte = *expect::<u8>();
            assert_eq!(curr_byte, 0);

            Layer::new().with(1u16).enter(|| {
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

        Layer::new().with(2u8).enter(|| {
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
