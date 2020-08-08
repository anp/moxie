//! An implicit environment which is indexed by type.
//!
//! # Offering values to the local environment
//!
//! Add types to the local environment by creating and `enter`ing a [`Layer`],
//! and retrieve them using [`get`] or [`expect`]:
//!
//! ```
//! #[derive(Copy, Clone, Debug, PartialEq)]
//! enum Theme {
//!     Light,
//!     Dark,
//! }
//!
//! // no theme has been set yet
//! assert!(illicit::get::<Theme>().is_err());
//!
//! illicit::Layer::new().offer(Theme::Light).enter(|| {
//!     assert_eq!(*illicit::expect::<Theme>(), Theme::Light,);
//! });
//!
//! // previous theme "expired"
//! assert!(illicit::get::<Theme>().is_err());
//! ```
//!
//! # Receiving arguments "by environment"
//!
//! Annotate functions which require access to types in the environment with
//! [`from_env`]:
//!
//! ```
//! # #[derive(Copy, Clone, Debug, PartialEq)]
//! # enum Theme {
//! #     Light,
//! #     Dark,
//! # }
//! #
//! impl Theme {
//!     /// Calls `child` with this theme set as current.
//!     fn set<R>(self, child: impl FnOnce() -> R) -> R {
//!         illicit::Layer::new().offer(self).enter(child)
//!     }
//!
//!     /// Retrieves the current theme. Panics if none has been set.
//!     #[illicit::from_env(current_theme: &Theme)]
//!     fn current() -> Self {
//!         *current_theme
//!     }
//! }
//!
//! Theme::Light.set(|| {
//!     assert_eq!(Theme::current(), Theme::Light);
//!
//!     // we can set a temporary override for part of our call tree
//!     Theme::Dark.set(|| assert_eq!(Theme::current(), Theme::Dark));
//!
//!     // but it only lasts as long as the inner `enter` call
//!     assert_eq!(Theme::current(), Theme::Light);
//! });
//! ```
//!
//! ## Caution
//!
//! This provides convenient sugar for values stored in the current environment
//! as an alternative to thread-locals or a manually propagated context object.
//! However this approach incurs a significant cost in that the following code
//! will panic:
//!
//! ```should_panic
//! # #[derive(Copy, Clone, Debug, PartialEq)]
//! # enum Theme {
//! #     Light,
//! #     Dark,
//! # }
//! #
//! # impl Theme {
//! #     #[illicit::from_env(current_theme: &Theme)]
//! #     fn current() -> Self {
//! #         *current_theme
//! #     }
//! # }
//! println!("{:?}", Theme::current());
//! ```
//!
//! See the attribute's documentation for more details, and please consider
//! whether this is appropriate for your use case before taking it on as a
//! dependency.
//!
//! # Debugging
//!
//! Use [`Snapshot::get`] to retrieve a copy of the current local environment.
//!
//! # Comparisons
//!
//! ## execution-context
//!
//! `illicit` provides capabilities very similar in ways to
//! [execution-context]. Both crates allow one to propagate implicit values to
//! "downstream" code, and they both allow that downstream code to provide its
//! own additional values to the context. Both crates prevent mutation of
//! contained types without interior mutability.
//!
//! One notable difference is how they handle multi-threading.
//! `execution-context` requires types contained in a "flow-local" to implement
//! `Send` so that contexts can be sent between threads. In contrast, while
//! `illicit` supports the reuse of contexts, it prioritizes the single-threaded
//! use case and does not require `Send`.
//!
//! The other noteworthy difference is in how they're "addressed":
//! `execution-context` defines static variables that are referenced by
//! name/symbol, whereas `illicit` allows the definition of a single local value
//! per type and does not rely on naming. This offers some nice properties but
//! it also sacrifices the static guarantee that there will always be a default
//! value.
//!
//! ## `thread_local!`
//!
//! This crate is implemented on top of a thread-local variable which stores the
//! context, and can be seen as utilities for dynamically creating and
//! destroying thread-locals for arbitrary types. The other key difference is
//! that the ability to temporarily override a thread-local is built-in.
//!
//! The main cost over writing one's own thread-local is that this does incur
//! the additional overhead of a `HashMap` access, some `TypeId` comparison, and
//! some pointer dereferences.
//!
//! [execution-context]: https://docs.rs/execution-context

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
};

/// Defines required `illicit::get` values for a function. Binds the provided
/// types as if references to them were implicit function arguments:
///
/// ```
/// #[derive(Debug, PartialEq)]
/// enum TextDirection {
///     Ltr,
///     Rtl,
/// }
///
/// impl TextDirection {
///     fn set<R>(self, op: impl FnOnce() -> R) -> R {
///         illicit::Layer::new().offer(self).enter(op)
///     }
/// }
///
/// #[illicit::from_env(direction: &TextDirection)]
/// fn align_text_to_end(t: &str, width: usize) -> String {
///     assert!(t.len() <= width, "no linebreaking included, this is unicode-sinful as it is");
///     match direction {
///         TextDirection::Ltr => format!("{0:>1$}", t, width),
///         TextDirection::Rtl => format!("{0:<1$}", t, width),
///     }
/// }
///
/// let get_aligned = || align_text_to_end("whoa", 8);
///
/// let right_aligned = TextDirection::Ltr.set(get_aligned);
/// let left_aligned = TextDirection::Rtl.set(get_aligned);
///
/// assert_eq!(right_aligned, "    whoa");
/// assert_eq!(left_aligned, "whoa    ");
/// ```
///
/// # Panics
///
/// Will cause the annotated function to panic if it is invoked without the
/// requested type in its environment:
///
/// ```should_panic
/// # #[derive(Debug, PartialEq)]
/// # enum TextDirection {
/// #     Ltr,
/// #     Rtl,
/// # }
/// #
/// # impl TextDirection {
/// #     fn set<R>(self, op: impl FnOnce() -> R) -> R {
/// #         illicit::Layer::new().offer(self).enter(op)
/// #     }
/// # }
/// #
/// # #[illicit::from_env(direction: &TextDirection)]
/// # fn align_text_to_end(t: &str, width: usize) -> String {
/// #     assert!(t.len() <= width, "no linebreaking included, this is unicode-sinful as it is");
/// #     match direction {
/// #         TextDirection::Ltr => format!("{0:>1$}", t, width),
/// #         TextDirection::Rtl => format!("{0:<1$}", t, width),
/// #     }
/// # }
/// #
/// align_text_to_end("oopsie", 8);
/// ```
///
/// This attribute adds an `Environment Expectations` section to the doc
/// comments of the annotated function to communicate this risk to users.
#[doc(inline)]
pub use illicit_macro::from_env;

thread_local! {
    /// The current dynamic scope.
    static CURRENT_SCOPE: RefCell<Layer> = RefCell::new(Layer {
            depth: 0,
            values: Default::default(),
        }
    );
}

/// Returns a reference to a value in the current environment if it is
/// present.
///
/// # Errors
///
/// Returns an error if the requested type is not available in the local
/// environment.
///
/// # Examples
///
/// ```
/// let msg = "hello!";
/// illicit::Layer::new().offer(String::from(msg)).enter(|| {
///     assert_eq!(&*illicit::get::<String>().unwrap(), msg);
/// });
/// ```
///
/// Returns [`GetFailed`] if the requested type is not in the environment:
///
/// ```
/// assert!(illicit::get::<String>().is_err());
/// ```
pub fn get<E>() -> Result<impl Deref<Target = E> + Debug + 'static, GetFailed>
where
    E: Any + Debug + 'static,
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
///
/// # Examples
///
/// ```
/// let msg = "hello!";
/// illicit::Layer::new().offer(String::from(msg)).enter(|| {
///     assert_eq!(&*illicit::expect::<String>(), msg);
/// });
/// ```
///
/// Panics if the requested type is not in the environment:
///
/// ```should_panic
/// println!("{}", &*illicit::expect::<String>());
/// ```
#[track_caller]
pub fn expect<E>() -> impl Deref<Target = E> + 'static
where
    E: Any + Debug + 'static,
{
    get().unwrap()
}

/// Removes the provided type from the current environment for the remainder
/// of its scope. Parent environments may still possess a reference to
/// the value.
///
/// # Example
///
/// ```
/// let msg = "hello!";
/// illicit::Layer::new().offer(String::from(msg)).enter(|| {
///     assert_eq!(&*illicit::expect::<String>(), msg);
///
///     illicit::hide::<String>();
///     assert!(illicit::get::<String>().is_err());
/// });
/// ```
pub fn hide<E: 'static>() {
    CURRENT_SCOPE.with(|current| {
        let mut env = current.borrow_mut();
        let mut without_e = env.values.clone();
        let excluded_ty = TypeId::of::<E>();
        without_e.retain(|(ty, _)| ty != &excluded_ty);
        *env = Layer { values: without_e, depth: env.depth };
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
///
/// # Examples
///
/// Code always sees the contents of the "bottom-most" `Layer`:
///
/// ```
/// illicit::Layer::new().offer(String::new()).offer(5u16).enter(|| {
///     assert!(illicit::expect::<String>().is_empty());
///     assert_eq!(*illicit::expect::<u16>(), 5);
///
///     illicit::Layer::new().offer(10u16).enter(|| {
///         assert!(illicit::expect::<String>().is_empty());
///         assert_eq!(*illicit::expect::<u16>(), 10);
///     });
///
///     assert!(illicit::expect::<String>().is_empty());
///     assert_eq!(*illicit::expect::<u16>(), 5);
/// });
/// ```
#[derive(Clone)]
pub struct Layer {
    depth: u32,
    values: Vec<(TypeId, AnonRc)>,
}

impl Default for Layer {
    #[track_caller]
    fn default() -> Self {
        let mut values = Vec::new();
        let mut depth = 0;

        CURRENT_SCOPE.with(|current| {
            let current = current.borrow();
            depth = current.depth + 1;
            values = current.values.clone();
        });

        Self { values, depth }
    }
}

impl From<Snapshot> for Layer {
    fn from(snapshot: Snapshot) -> Self {
        snapshot.current
    }
}

impl Layer {
    /// Construct a new layer which defaults to the contents of the current one.
    /// Call [`Layer::offer`] to add values to the new layer before calling
    /// [`Layer::enter`] to run a closure with access to the new and old
    /// values.
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the new item and returns the modified layer.
    ///
    /// The offered type must implement `Debug` to allow [`get`]'s errors
    /// to display the contents of the illicit environment. It must also satisfy
    /// the `'static` lifetime because [`get`] is unable to express any
    /// lifetime constraints at its callsite.
    #[track_caller]
    pub fn offer<E>(mut self, v: E) -> Self
    where
        E: Debug + 'static,
    {
        let anon = AnonRc::new(v, self.depth);
        let existing = self.values.iter_mut().find(|(id, _)| *id == anon.id());

        if let Some((_, existing)) = existing {
            *existing = anon;
        } else {
            self.values.push((anon.id(), anon));
        }

        self
    }

    /// Call `child_fn` with this layer as the local environment.
    pub fn enter<R>(self, child_fn: impl FnOnce() -> R) -> R {
        let _reset_when_done_please = CURRENT_SCOPE.with(|parent| {
            let mut parent = parent.borrow_mut();
            let parent = replace(&mut *parent, self);

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
        let is_alternate = f.alternate();
        let mut s = f.debug_struct("Layer");
        for (ty, anon) in self.values.iter().map(|(_, v)| (v.ty(), v)).collect::<BTreeMap<_, _>>() {
            if is_alternate {
                s.field(ty, &(anon.debug(), anon.location()));
            } else {
                s.field(ty, anon.debug());
            }
        }
        s.finish()
    }
}

// interior mutations happen only within panic-safe methods
impl std::panic::UnwindSafe for Layer {}
impl std::panic::RefUnwindSafe for Layer {}

/// Implemented by types that can offer themselves as context to a child call.
pub trait AsContext {
    /// Call `op` within the context of a [`Layer`] containing `self`.
    fn offer<R>(self, op: impl FnOnce() -> R) -> R;
}

impl<T> AsContext for T
where
    T: Debug + Sized + 'static,
{
    fn offer<R>(self, op: impl FnOnce() -> R) -> R {
        Layer::new().offer(self).enter(op)
    }
}

/// A point-in-time representation of the implicit environment.
///
/// # Examples
///
/// Collecting a `Snapshot` is useful for debugging:
///
/// ```
/// illicit::Layer::new().offer(5u16).enter(|| {
///     println!("{:#?}", illicit::Snapshot::get());
/// });
/// ```
///
/// `Snapshot`s can also be converted back into [`Layer`]s for re-use:
///
/// ```
/// let mut snapshot = None;
/// illicit::Layer::new().offer(5u16).enter(|| {
///     assert_eq!(*illicit::expect::<u16>(), 5);
///     snapshot = Some(illicit::Snapshot::get());
/// });
/// assert!(illicit::get::<u16>().is_err());
///
/// illicit::Layer::from(snapshot.unwrap()).enter(|| {
///     assert_eq!(*illicit::expect::<u16>(), 5);
/// });
/// ```
#[derive(Clone, Debug)]
pub struct Snapshot {
    current: Layer,
}

impl Snapshot {
    /// Returns a snapshot of the current context. Suitable for debug printing,
    /// or can be converted into a [`Layer`] for reuse.
    pub fn get() -> Self {
        let mut current: Layer = CURRENT_SCOPE.with(|s| (*s.borrow()).clone());

        current.values.sort_by_key(|(_, anon)| anon.depth());

        Snapshot { current }
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
            "expected a `{}` from the environment, did not find it in current env: {:?}",
            self.looked_up, &self.current_snapshot,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_display_snapshot, assert_snapshot};

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
        let e = get::<u8>().unwrap_err();
        assert_display_snapshot!(e);
    }

    #[test]
    fn layer_debug_impl() {
        let snapshot = Layer::new().offer(1u8).enter(Snapshot::get);
        // assert_debug_snapshot adds #, which prints file paths, which breaks snapshots
        assert_snapshot!(format!("{:?}", snapshot));
    }
}
