//! Caches for storing the results of repeated queries.
//!
//! Every query to a cache has a "scope" by which it is namespaced. The types in
//! this module cache queries which have arbitrary scope types, storing one
//! input value and one ouput value per scope:
//!
//! ```
//! let storage = topo::cache::SharedLocalCache::default();
//! let count = std::cell::Cell::new(0);
//! let increment = |n| {
//!     storage.cache_with(
//!         "increment by arg",
//!         &n,
//!         |&n| {
//!             let new_count = count.get() + n;
//!             count.set(new_count);
//!             new_count
//!         },
//!         Clone::clone,
//!     )
//! };
//!
//! assert_eq!(count.get(), 0);
//!
//! assert_eq!(increment(2), 2);
//! assert_eq!(count.get(), 2);
//!
//! // running the query again with the same input has no external effect
//! assert_eq!(increment(2), 2);
//! assert_eq!(count.get(), 2);
//!
//! // same query with a different input will run
//! assert_eq!(increment(1), 3);
//! assert_eq!(count.get(), 3);
//!
//! // these all have the same scope, so this runs again
//! assert_eq!(increment(3), 6);
//! assert_eq!(count.get(), 6);
//!
//! let decrement = |n| {
//!     storage.cache_with(
//!         "decrement by arg",
//!         &n,
//!         |&n| {
//!             let new_count = count.get() - n;
//!             count.set(new_count);
//!             new_count
//!         },
//!         Clone::clone,
//!     )
//! };
//!
//! assert_eq!(decrement(1), 5);
//! assert_eq!(count.get(), 5);
//!
//! // increment's last query is still cached
//! assert_eq!(increment(3), 6);
//! assert_eq!(count.get(), 5);
//!
//! // this one is still cached too, because they're different queries
//! assert_eq!(decrement(1), 5);
//! assert_eq!(count.get(), 5);
//! ```

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
use namespace::{Gc, Namespace};

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
through [`Shared" stringify!($cache) "::cache_with`] and [`Shared" stringify!($cache) "::gc`].

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

Each time [`" stringify!($cache) "::gc`] is called it removes any values which haven't been
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
            .from_hash(query.hash, |t| t == &query.ty())
            .or_insert_with(|| {
                (query.ty(), query.make_namespace())
            }).1;
        gc.as_any_mut().downcast_mut().unwrap()
    }

    /// Drops any values which were not referenced since the last call to this method.
    pub fn gc(&mut self) {
        for namespace in self.inner.values_mut() {
            namespace.gc();
        }
    }
}

impl std::panic::UnwindSafe for $cache {}
impl std::panic::RefUnwindSafe for $cache {}

doc_comment! {"
Provides shared, synchronized access to a [`" stringify!($cache) "`] and a function-memoization
API in [`" stringify!($shared) "::cache_with`].

# Example

```
let storage = topo::cache::" stringify!($shared) r#"::default();
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
    /// Caches the result of `init(arg)` once per `key`, re-running it when `arg` changes. Always
    /// runs `with` on the stored `Output` before returning the result.
    ///
    /// See the [moxie](https://docs.rs/moxie) crate for ergonomic wrappers.
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
    }

doc_comment!{"
Forwards to [`" stringify!($cache) "::gc`].
"=>
    pub fn gc(&self) {
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
}
    };
}

define_cache!(LocalCache, Rc, RefCell::borrow_mut);
define_cache!(Cache: Send, Arc, Mutex::lock);

/// A query type that was hashed as part of an initial lookup and which can be
/// used to store fresh values back to the cache.
pub struct Query<Scope, Input, Output> {
    ty: PhantomData<(Scope, Input, Output)>,
    hash: u64,
}

impl<Scope, Input, Output> Query<Scope, Input, Output>
where
    Scope: 'static,
    Input: 'static,
    Output: 'static,
{
    fn new(build: &impl BuildHasher) -> Self {
        // this is a bit unrustic but it lets us keep the typeid defined in a *single*
        // place
        let mut new = Query { ty: PhantomData, hash: 0 };
        let mut hasher = build.build_hasher();
        new.ty().hash(&mut hasher);
        new.hash = hasher.finish();
        new
    }

    fn make_namespace(&self) -> Box<Namespace<Scope, Input, Output>> {
        Box::new(Namespace::default())
    }

    fn ty(&self) -> TypeId {
        TypeId::of::<(Scope, Input, Output)>()
    }
}
