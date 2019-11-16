//! Type-indexed scoped singletons, propagated through an implicit backing context.

#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs, intra_doc_link_resolution_failure)]

use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
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
#[derive(Clone, Debug, Default)]
pub struct Scope {
    inner: Rc<Env>,
}

thread_local! {
    /// The current dynamic scope.
    static CURRENT_SCOPE: RefCell<Scope> = Default::default();
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
#[derive(Debug, Default)]
pub struct Env {
    values: HashMap<TypeId, AnonRc>,
}

impl Env {
    #[doc(hidden)]
    pub fn unstable_new() -> Self {
        Self {
            values: Default::default(),
        }
    }

    /// TODO
    pub fn enter<R>(mut self, child_fn: impl FnOnce() -> R) -> R {
        let _reset_when_done_please = CURRENT_SCOPE.with(|parent| {
            let mut parent = parent.borrow_mut();

            for (id, rc) in &parent.inner.values {
                self.values.entry(*id).or_insert_with(|| rc.clone());
            }

            let parent = replace(
                &mut *parent,
                Scope {
                    inner: Rc::new(self),
                },
            );

            scopeguard::guard(parent, move |prev| {
                CURRENT_SCOPE.with(|p| p.replace(prev));
            })
        });

        // call this out here so these calls can be nested
        child_fn()
    }

    /// Removes the provided type from the current environment for the remainder of its scope.
    /// Parent environments may still possess a reference to the value.
    pub fn hide<E: 'static>() {
        unimplemented!()
        // Point::with_current_mut(|p| {
        //     let mut without_e: EnvInner = (*p.state.env.inner).clone();
        //     let excluded_ty = TypeId::of::<E>();
        //     without_e.retain(|ty, _| ty != &excluded_ty);

        //     p.state.env = Env {
        //         inner: Rc::new(without_e),
        //     };
        // });
    }

    /// Returns a reference to a value in the current environment if it has been added to the
    /// environment by parent/enclosing [`call`] invocations.
    pub fn get<E>() -> Option<impl Deref<Target = E> + 'static>
    where
        E: Any + 'static,
    {
        let key = TypeId::of::<E>();
        let anon = CURRENT_SCOPE.with(|current| current.borrow().inner.values.get(&key).cloned());

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
                "expected a `{}` from the environment, found none",
                std::any::type_name::<E>()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_env_looks_right() {
        unimplemented!()
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
            eprintln!("getting u8 from the environment");
            let curr_byte = *Env::expect::<u8>();
            // assert_eq!(curr_byte, 2, "just added 2u8");

            eprintln!("adding u16 to the environment");
            child_env!().enter(|| {
                assert_eq!(*Env::expect::<u8>(), 2, "parent added 2u8");
                eprintln!("hiding u8 from environment");
                Env::hide::<u8>();
                assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");
            });

            assert_eq!(
                *Env::get::<u8>().unwrap(),
                2,
                "returned to parent Env with 2u8"
            );

            Env::hide::<u8>();
            assert!(Env::get::<u8>().is_none(), "just removed u8 from Env");
        })
    }
}
