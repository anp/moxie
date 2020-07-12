#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! Caches for storing the results of repeated function calls. The cache types
//! available use minimal dynamic dispatch to allow storing arbitrarily many
//! types of query results in a single parent store.
//!
//! There are two main flavors of cache available for use in this crate:
//!
//! | Mutable type   | Requires `Send`? |
//! |----------------|------------------|
//! | [`SendCache`]  | yes              |
//! | [`LocalCache`] | no               |
//!
//! These "inner" caches require mutable access to call their functions like
//! [`LocalCache::get_if_arg_eq_prev_input`] or [`SendCache::store`].
//!
//! They each come available in a shared variant:
//!
//! | Shared type          | Synchronized? |
//! |----------------------|---------------|
//! | [`SharedSendCache`]  | Mutex         |
//! | [`SharedLocalCache`] | RefCell       |
//!
//! These variants are wrapped with reference counting and synchronization and
//! are used by calling [`SharedSendCache::cache_with`] or
//! [`SharedLocalCache::cache`].
//!
//! # Query types
//!
//! Each [`Query`] type maps to a typed "namespace" within the unityped cache
//! storage, each query having a distinct type for its scope, input, and
//! output.
//!
//! ## Scopes
//!
//! The scope of a query is its identifier within cache storage.
//! Scopes must implement `Eq` and `Hash` so that results can be
//! efficiently and uniquely indexed within a namespace.
//!
//! Each scope identifies 0-1 `(Input, Output)` pairs in each namespace. The
//! same type of scope can be used in multiple [`Query`]s without collision if
//! the types of inputs, outputs, or both differ.
//!
//! ## Inputs
//!
//! The input to a query determines when it is (re-)run. If a given query has
//! been run before, then the previous input is compared to the current input
//! before potentially running the query. If the input hasn't changed, the query
//! can be skipped and its previously-stored output is returned.
//!
//! ## Outputs
//!
//! The only constraint on query outputs is that they are owned (`Output:
//! 'static`). This imposes the inconvenient requirement that all access to
//! stored values occurs during the scope of a closure (similar to thread-locals
//! in the standard library).
//!
//! The most common way to work around this requirement is to choose output
//! types that cheaply implement [`std::clone::Clone`].
//!
//! # Allocations
//!
//! In order to store distinct query results in the same container, allocations
//! and indirection are required.
//!
//! ## Borrowed query parameters
//!
//! All of the cache functions accept a reference to a type `Key:
//! ToOwned<Owned=Scope>` so that the scope is only cloned on the first
//! insertion to its storage and all subsequent lookups can be with a borrowed
//! type.
//!
//! Like the query scope, functions to get cache values accept a borrowed
//! version of the input and only clone it when the input has changed.
//!
//! ## Causes
//!
//! There are three situations where these caches allocate:
//!
//! 1. caching new types which haven't been seen by that cache instance yet
//! 2. storing the results of a new query
//! 3. updating the results of a stored query
//!
//! There are several types of allocations performed by the caches in this
//! crate:
//!
//! | Allocation                         | Causes   |
//! |------------------------------------|----------|
//! | box a new, empty namespace         | (1)      |
//! | resize a cache's map of namespaces | (1)      |
//! | call `.to_owned()` on a scope/key  | (2)      |
//! | resize a namespace's storage       | (2)      |
//! | call `.to_owned()` on an input/arg | (2), (3) |
//!
//! Outside of these, only user-defined functions should perform any allocation.
//!
//! # Garbage Collection
//!
//! Every value in the cache has a [`Liveness`] which is set to
//! [`Liveness::Live`] when the value is first stored and again when it is read.
//!
//! The inner caches implement [`Gc`] and it is available through
//! [`SharedLocalCache::gc`] & [`SharedSendCache::gc`]. When called, the `gc()`
//! method retains only those values which are still [`Liveness::Live`] and then
//! marks them all [`Liveness::Dead`] again.
//!
//! This behavior resembles a simple mark-and-sweep garbage collector where the
//! "mark phase" is the use of the cache in between [`Gc::gc`] calls. Any values
//! which weren't used in the mark phase are dropped in the next "sweep phase"
//! when [`Gc::gc`] is called.
//!
//! ## Nested Queries
//!
//! While it is possible to nest use of the shared caches within the init
//! closures passed to them, the caches do not yet track the required dependency
//! relationship to correctly retain intermediate cached results across GCs.
//! While this works well enough for some scenarios it needs to be resolved in
//! the general case before this way of using this crate is recommended.

use downcast_rs::{impl_downcast, Downcast};
use hash_hasher::HashBuildHasher;
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::{
    any::TypeId,
    borrow::Borrow,
    cell::RefCell,
    cmp::Eq,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

mod namespace;

pub use namespace::Hashed;
use namespace::Namespace;

/// A type which can contain values of varying liveness, including itself.
pub trait Gc: Downcast + Debug {
    /// Remove dead entries, returning the container's own status afterwards.
    fn gc(&mut self) -> Liveness;
}

impl_downcast!(Gc);

/// Describes the outcome of garbage collection for a cached value.
#[derive(Debug, PartialEq)]
pub enum Liveness {
    /// The value would be retained in a GC right now.
    Live,
    /// The value would be dropped in a GC right now.
    Dead,
}

macro_rules! doc_comment {
    ($($contents:expr)+ => $($item:tt)+) => {
        doc_comment! {@ concat!($($contents),+), $($item)+ }
    };
    (@ $contents:expr, $($item:tt)+) => {
        #[doc = $contents]
        $($item)+
    };
}

macro_rules! define_cache {
    ($cache:ident $(: $bound:ident)?, $($rest:tt)*) => {
paste::item! {
    define_cache! {@
        $cache $(: $bound)?,
        [<$cache:snake _tests>],
        [<Shared $cache>],
        $($rest)*
    }
}
    };
    (@
        $cache:ident $(: $bound:ident)?,
        $test_mod:ident,
        $shared:ident,
        $refct:ident,
        $lock:ident :: $acquire:ident
    ) => {

doc_comment! {"
Holds arbitrary query results which are namespaced by arbitrary scope types. Usually used
through [`Shared" stringify!($cache) "::cache_with`] and [`Gc::gc`].

# Query types

> Note: the types referenced in this documentation are only visible on individual methods, as
> `" stringify!($cache) "` is not itself a generic type.

Storage is sharded by the type of the query. The type of a query has three parts:
 
The query scope is the value which indexes the storage for a particular query type, it has the
bound `Scope: 'static + Eq + Hash" $(" + " stringify!($bound))? "`.

Each `Scope` corresponds to at most a single `Input: 'static" $(" + " stringify!($bound))? "`
and a single `Output: 'static" $(" + " stringify!($bound))? "` value at any given time.

# Reading stored values

See [`" stringify!($cache) "::get_if_arg_eq_prev_input`] which accepts borrowed forms of `Scope`
and `Input`: `Key` and `Arg` respectively. `Arg` must satisfy `PartialEq<Input>` to determine
whether to return a stored output.

# Garbage Collection

Each time [`Gc::gc`] is called it removes any values which haven't been
referenced since the prior call.

After each GC, all values still in the cache are marked garbage. They are marked live again when
inserted with [`" stringify!($cache) "::store`] or read with
[`" stringify!($cache) "::get_if_arg_eq_prev_input`].
"=>
#[derive(Debug, Default)]
pub struct $cache {
    /// We use a [`hash_hasher::HashedMap`] here because we know that `Query` is made up only of
    /// `TypeIds` which come pre-hashed courtesy of rustc.
    inner: HashMap<TypeId, Box<dyn Gc $(+ $bound)?>, HashBuildHasher>,
}}

impl $cache {
    /// Return a reference to a query's stored output if a result is stored and `arg` equals the
    /// previously-stored `Input`. If a reference is returned, the stored input/output
    /// is marked live and will not be GC'd the next call.
    ///
    /// If no reference is found, the hashes of the query type and the provided key are returned
    /// to be reused when storing a value.
    pub fn get_if_arg_eq_prev_input<'k, Key, Scope, Arg, Input, Output>(
        &mut self,
        key: &'k Key,
        arg: &Arg,
    ) -> Result<&Output, (Query<Scope, Input, Output>, Hashed<&'k Key>)>
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let query = Query::new(self.inner.hasher());
        self.get_namespace_mut(&query).get_if_input_eq(key, arg).map_err(|h| (query, h))
    }

    /// Stores the input/output of a query which will not be GC'd at the next call.
    /// Call `get_if_arg_eq_prev_input` to get a `Hashed` instance.
    pub fn store<Key, Scope, Input, Output>(
        &mut self,
        (query, key): (Query<Scope, Input, Output>, Hashed<&Key>),
        input: Input,
        output: Output,
    ) where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.get_namespace_mut(&query).store(key, input, output);
    }

    fn get_namespace_mut<Scope, Input, Output>(
        &mut self,
        query: &Query<Scope, Input, Output>,
    ) -> &mut Namespace<Scope, Input, Output>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let gc: &mut (dyn Gc $(+ $bound)?) = &mut **self
            .inner
            .raw_entry_mut()
            .from_hash(query.hash(), |t| t == &query.ty())
            .or_insert_with(|| {
                (query.ty(), query.make_namespace())
            }).1;
        gc.as_any_mut().downcast_mut().unwrap()
    }
}

impl Gc for $cache {
    fn gc(&mut self) -> Liveness {
        self.inner.values_mut()
            .fold(Liveness::Dead, |l, namespace| {
                if namespace.gc() == Liveness::Live {
                    Liveness::Live
                } else {
                    l
                }
            })
    }
}

impl std::panic::UnwindSafe for $cache {}
impl std::panic::RefUnwindSafe for $cache {}

doc_comment! {"
Provides shared, synchronized access to a [`" stringify!($cache) "`] and a function-memoization
API in [`" stringify!($shared) "::cache_with`].

# Example

```
let storage = dyn_cache::" stringify!($shared) r#"::default();
let call_count = std::cell::Cell::new(0);
let increment_count = |&to_add: &i32| {
    let new_count = call_count.get() + to_add;
    call_count.set(new_count);
    new_count
};

assert_eq!(call_count.get(), 0, "not called yet");

let with_one = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "called only once");
assert_eq!(call_count.get(), with_one);

let with_one_again = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "still called only once, previous value cached");
assert_eq!(call_count.get(), with_one_again);

let with_two = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 3, "called again with a new, larger increment");
assert_eq!(call_count.get(), with_two);

let with_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 4, "called again with the same increment, different scope");
assert_eq!(call_count.get(), with_other_query);

let with_two_again = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 4, "cell still has last mutation's value");
assert_eq!(with_two_again, with_two, "cache should still have previous value");

storage.gc(); // won't drop any values, but sets all of the cached values to be dropped
call_count.set(0);

// re-run 'b', marking it live
let reran_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(reran_other_query , 4, "returns the cached value");
assert_eq!(call_count.get(), 0, "without running increment_count");

storage.gc(); // query 'a' will be dropped

// re-run 'b', observing cached value
let reran_other_query = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
assert_eq!(reran_other_query , 4, "still returns the cached value");
assert_eq!(call_count.get(), 0, "still without running increment_count");

// run 'a' again, observe no cached value
let with_one_again = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
assert_eq!(call_count.get(), 1, "called without caching");
assert_eq!(call_count.get(), with_one_again);
```
"#=>
#[derive(Clone)]
pub struct $shared {
    inner: $refct<$lock<$cache>>,
}}

impl Default for $shared {
    fn default() -> Self {
        Self {
            inner: $refct::new($lock::new($cache::default()))
        }
    }
}

impl $shared {
doc_comment!{r"
Caches the result of `init(arg)` once per `key`, re-running it when `arg` changes. Always
runs `with` on the stored `Output` before returning the result.

See [`" stringify!($shared) "::cache`] for an ergonomic wrapper that requires `Output: Clone`.
"=>
    pub fn cache_with<Key, Scope, Arg, Input, Output, Ret>(
        &self,
        key: &Key,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
        with: impl FnOnce(&Output) -> Ret,
    ) -> Ret
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ToOwned<Owned=Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
        Ret: 'static $(+ $bound)?,
    {
        let hashed = match { self.inner.$acquire().get_if_arg_eq_prev_input(key, arg) } {
            Ok(stored) => return with(stored),
            Err(h) => h,
        };

        let arg = arg.to_owned();
        let to_store = init(&arg);
        let to_return = with(&to_store);
        self.inner.$acquire().store(hashed, arg, to_store);
        to_return
    }}

doc_comment!{r"
Caches the result of `init(arg)` once per `key`, re-running it when `arg` changes. Clones
the cached output before returning the result.

See [`" stringify!($shared) "::cache_with`] for a lower-level version which does not require
`Output: Clone`.
"=>
    pub fn cache<Key, Scope, Arg, Input, Output>(
        &self,
        key: &Key,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
    ) -> Output
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ToOwned<Owned=Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static + Clone $(+ $bound)?,
    {
        self.cache_with(key, arg, init, Clone::clone)
    }}

doc_comment!{r"
Caches the result of `init(arg)` once per `key`, re-running it when `arg` changes.

Does not return any reference to the cached value. See [`" stringify!($shared) "::cache`] 
for similar functionality that returns a copy of `Output` or
[`" stringify!($shared) "::cache_with`] which allows specifying other pre-return functions.
"=>
    pub fn hold<Key, Scope, Arg, Input, Output>(
        &self,
        key: &Key,
        arg: &Arg,
        init: impl FnOnce(&Input) -> Output,
    )
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ToOwned<Owned=Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        self.cache_with(key, arg, init, |_| {})
    }}

doc_comment!{"
Forwards to [`Gc::gc`].
"=>
    pub fn gc(&self) -> Liveness {
        self.inner.$acquire().gc()
    }}
}

impl Debug for $shared {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple(stringify!($shared))
            .field(&*self.inner.$acquire())
            .finish()
    }
}

impl From<$cache> for $shared {
    fn from(inner: $cache) -> Self {
        Self { inner: $refct::new($lock::new(inner)) }
    }
}

impl std::panic::UnwindSafe for $shared {}
impl std::panic::RefUnwindSafe for $shared {}

#[cfg(test)]
mod $test_mod {
    use super::*;
    use std::sync::Arc;
    use parking_lot::Mutex;

    #[test]
    fn single_query_with_gc() {
        let storage = $shared::default();
        let call_count = std::cell::Cell::new(0);
        let increment_count = |&to_add: &i32| {
            let new_count = call_count.get() + to_add;
            call_count.set(new_count);
            new_count
        };

        assert_eq!(call_count.get(), 0);

        let with_b = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 1);
        assert_eq!(call_count.get(), with_b);

        storage.gc(); // won't drop any values, but sets all of the cached values to be dropped
        call_count.set(0);

        let rerun_b = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
        assert_eq!(rerun_b , 1);
        assert_eq!(call_count.get(), 0);

        storage.gc();
        // 'b' is not refreshed before we call gc again
        storage.gc();

        let again = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
        assert_eq!(again, 1);
        assert_eq!(call_count.get(), 1);
    }

    #[test]
    fn distinct_scopes_distinct_storage() {
        let storage = $shared::default();
        let call_count = std::cell::Cell::new(0);
        let increment_count = |&to_add: &i32| {
            let new_count = call_count.get() + to_add;
            call_count.set(new_count);
            new_count
        };

        assert_eq!(call_count.get(), 0);

        let a_with_1 = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 1);
        assert_eq!(call_count.get(), a_with_1);

        let b_with_1 = storage.cache_with(&'b', &1, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 2);
        assert_eq!(call_count.get(), b_with_1);

        let a_with_1_again = storage.cache_with(&'a', &1, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 2, "untouched");
        assert_eq!(a_with_1_again, a_with_1, "cached");

        let with_a_2 = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 4);
        assert_eq!(call_count.get(), with_a_2);

        let with_a_2_again = storage.cache_with(&'a', &2, &increment_count, Clone::clone);
        assert_eq!(call_count.get(), 4);
        assert_eq!(with_a_2_again, with_a_2);
    }

    #[test]
    fn hold_retains_across_gcs() {
        let storage = $shared::default();

        let guard_count_inc = Arc::new(Mutex::new(0));
        let drop_count_inc = Arc::new(Mutex::new(0));
        let (guard_count, drop_count) = (guard_count_inc.clone(), drop_count_inc.clone());

        macro_rules! assert_counts {
            ($guard:expr, $drop:expr) => {{
                assert_eq!($guard, *guard_count.lock());
                assert_eq!($drop, *drop_count.lock());
            }};
        }

        let make_guard = || {
            let (guard_count_inc, drop_count_inc) = (
                guard_count_inc.clone(),
                drop_count_inc.clone(),
            );
            storage.hold(
                &'a',
                &(),
                move |&()| {
                    *guard_count_inc.lock() += 1;
                    scopeguard::guard((), move |()| *drop_count_inc.lock() += 1)
                },
            );
        };

        assert_counts!(0, 0);
        make_guard();
        assert_counts!(1, 0);
        storage.gc();
        assert_counts!(1, 0);
        make_guard();
        assert_counts!(1, 0);
        storage.gc();
        assert_counts!(1, 0);
        storage.gc();
        assert_counts!(1, 1);
        make_guard();
        assert_counts!(2, 1);
    }
}
    };
}

define_cache!(LocalCache, Rc, RefCell::borrow_mut);
define_cache!(SendCache: Send, Arc, Mutex::lock);

/// The type of a dynamic cache query, used to shard storage in a fashion
/// similar to `anymap` or `typemap`.
pub struct Query<Scope, Input, Output, H = HashBuildHasher> {
    ty: PhantomData<(Scope, Input, Output)>,
    hasher: PhantomData<H>,
    hash: u64,
}

impl<Scope, Input, Output, H> Query<Scope, Input, Output, H>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
    H: BuildHasher,
{
    fn new(build: &H) -> Self {
        // this is a bit unrustic but it lets us keep the typeid defined once
        let mut new = Query { ty: PhantomData, hasher: PhantomData, hash: 0 };
        let mut hasher = build.build_hasher();
        new.ty().hash(&mut hasher);
        new.hash = hasher.finish();
        new
    }

    fn make_namespace(&self) -> Box<Namespace<Scope, Input, Output>> {
        Box::new(Namespace::default())
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn ty(&self) -> TypeId {
        TypeId::of::<(Scope, Input, Output)>()
    }
}
