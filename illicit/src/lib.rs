//! Type-indexed scoped singletons, propagated through an implicit backing context.

#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs, intra_doc_link_resolution_failure)]

use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
        fmt::{Debug, Formatter, Result as FmtResult},
        mem::replace,
        ops::Deref,
        rc::Rc,
    },
};

#[doc(inline)]
pub use illicit_macro::from_env;

/// Immutable environment container for the current scope. Environment values can be
/// provided by parent environments, but child functions can only mutate their environment through
/// interior mutability.
///
/// The environment is type-indexed, and each `Scope` holds 0-1 references to every
/// `[std::any::Any] + 'static` type. Access is provided through read-only references.
///
/// Aside: one interesting implication of the above is the ability to define "private scoped global
/// values" which are private to functions which are nonetheless propagating the values with
/// their control flow. This can be useful for runtimes to offer themselves execution-local values
/// in functions which are invoked by external code. It can also be severely abused, like any
/// implicit state, and should be used with caution.
#[derive(Debug)]
pub struct Scope {
    env: Rc<Env>,
}

impl Scope {
    /// Returns a snapshot of the current dynamic scope. Most useful for debugging the contained
    /// `Env`.
    pub fn snapshot() -> Scope {
        CURRENT_SCOPE.with(|s| Scope {
            env: s.borrow().env.clone(),
        })
    }

    /// Removes the provided type from the current environment for the remainder of its scope.
    /// Parent environments may still possess a reference to the value.
    pub fn hide<E: 'static>() {
        CURRENT_SCOPE.with(|current| {
            let mut scope = current.borrow_mut();
            let mut without_e = scope.env.values.clone();
            let excluded_ty = TypeId::of::<E>();
            without_e.retain(|ty, _| ty != &excluded_ty);
            scope.env = Rc::new(Env { values: without_e });
        })
    }
}

thread_local! {
    /// The current dynamic scope.
    static CURRENT_SCOPE: RefCell<Scope> = RefCell::new(
        Scope {
            env: Rc::new( Env { values: Default::default() }),
        });
}

/// Declare additional environment values to expose to a child topological function's call tree.
#[macro_export]
macro_rules! child_env {
    ($($env_item_ty:ty => $env_item:expr),*) => {{
        #[allow(unused_mut)]
        let mut new_env = $crate::Env::unstable_new();
        $(
            $crate::AnonRc::unstable_new::<$env_item_ty>($env_item)
                .unstable_insert_into(&mut new_env);
        )*
        new_env
    }}
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct AnonRc {
    name: &'static str,
    id: TypeId,
    inner: Rc<dyn Any>,
    debug: Rc<dyn Debug>,
}

impl AnonRc {
    #[doc(hidden)]
    pub fn unstable_new<T: Debug + 'static>(inner: T) -> Self {
        let inner = Rc::new(inner);
        Self {
            name: std::any::type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            inner,
        }
    }

    #[doc(hidden)]
    pub fn unstable_insert_into(self, env: &mut Env) {
        env.values.insert(self.id, self);
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

/// TODO
pub struct Env {
    values: HashMap<TypeId, AnonRc>,
}

impl Env {
    #[doc(hidden)]
    pub fn unstable_new() -> Self {
        let mut new = Self {
            values: Default::default(),
        };

        CURRENT_SCOPE.with(|current| {
            let current = current.borrow();
            for anon in current.env.values.values() {
                anon.clone().unstable_insert_into(&mut new);
            }
        });

        new
    }

    /// TODO
    pub fn enter<R>(self, child_fn: impl FnOnce() -> R) -> R {
        let _reset_when_done_please = CURRENT_SCOPE.with(|parent| {
            let mut parent = parent.borrow_mut();
            let parent = replace(&mut *parent, Scope { env: Rc::new(self) });

            scopeguard::guard(parent, move |prev| {
                CURRENT_SCOPE.with(|p| p.replace(prev));
            })
        });

        // call this out here so these calls can be nested
        child_fn()
    }

    /// Returns a reference to a value in the current environment if it has been added to the
    /// environment by parent/enclosing [`call`] invocations.
    pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
    where
        E: Any + 'static,
    {
        let key = TypeId::of::<E>();
        let anon = CURRENT_SCOPE.with(|current| current.borrow().env.values.get(&key).cloned());

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
        if let Some(val) = Self::get() {
            val
        } else {
            panic!(
                "expected a `{}` from the environment, did not find it in current env: {:#?}",
                std::any::type_name::<E>(),
                Scope::snapshot().env,
            )
        }
    }
}

impl Debug for Env {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut f = f.debug_struct("Env");

        for anon in self.values.values() {
            f.field(anon.name, &*anon.debug);
        }

        f.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_debug_looks_right() {
        let current_scope = format!("{:?}", Scope::snapshot().env);
        assert_eq!(
            "Env", &current_scope,
            "environment should be empty and validly formatted"
        );

        child_env!(u8 => 42).enter(|| {
            let current_scope = format!("{:?}", Scope::snapshot().env);
            assert_eq!(
                "Env { u8: 42 }", &current_scope,
                "environment should have a u8 in it, no contents printing yet"
            );
        })
    }

    #[test]
    fn child_env_replaces_parent_env_values() {
        let mut first_called = false;
        let mut second_called = false;

        assert!(Env::get::<u8>().is_none());
        child_env!(u8 => 0u8).enter(|| {
            let curr_byte = *Env::expect::<u8>();
            assert_eq!(curr_byte, 0);
            first_called = true;

            child_env!(u8 => 1u8).enter(|| {
                let curr_byte = *Env::expect::<u8>();
                assert_eq!(curr_byte, 1);
                second_called = true;
            });

            assert!(second_called);
            assert_eq!(curr_byte, 0);
        });
        assert!(first_called);
        assert!(Env::get::<u8>().is_none());
    }

    #[test]
    fn child_sees_parent_env() {
        assert!(Env::get::<u8>().is_none());
        child_env!(u8 => 0u8).enter(|| {
            let curr_byte = *Env::expect::<u8>();
            assert_eq!(curr_byte, 0);

            child_env!(u16 => 1u16).enter(|| {
                let curr_byte = *Env::expect::<u8>();
                assert_eq!(curr_byte, 0, "must see u8 from enclosing environment");

                let curr_uh_twobyte = *Env::expect::<u16>();
                assert_eq!(curr_uh_twobyte, 1, "must see locally installed u16");
            });

            assert_eq!(curr_byte, 0, "must see 0");
        });
        assert!(Env::get::<u8>().is_none());
    }

    #[test]
    fn removing_from_env() {
        assert!(Env::get::<u8>().is_none());

        child_env!(u8 => 2).enter(|| {
            assert_eq!(*Env::expect::<u8>(), 2, "just added 2u8");

            child_env!().enter(|| {
                assert_eq!(*Env::expect::<u8>(), 2, "parent added 2u8");
                Scope::hide::<u8>();
                assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");
            });

            assert_eq!(
                *Env::get::<u8>().unwrap(),
                2,
                "returned to parent Env with 2u8"
            );

            Scope::hide::<u8>();
            assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");
        })
    }
}
