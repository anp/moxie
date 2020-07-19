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
    ($module:ident, $cache:ident $(: $bound:ident)?, $($rest:tt)*) => {
paste::item! {
    define_cache! {@
        $module,
        $cache $(: $bound)?,
        [<$cache:snake _tests>],
        [<Shared $cache>],
        $($rest)*
    }
}
    };
    (@
        $module:ident,
        $cache:ident $(: $bound:ident)?,
        $test_mod:ident,
        $shared:ident,
        $refct:ident,
        $lock:ident :: $acquire:ident
    ) => {

doc_comment! {"
Holds arbitrary query results which are namespaced by arbitrary scope types. Usually used
through [`" stringify!($shared) "::cache_with`] and [`" stringify!($shared) "::gc`].

# Query types

> Note: the types referenced in this documentation are only visible on individual methods, as
> `" stringify!($cache) "` is not itself a generic type.

Storage is sharded by the type of the query. The type of a query has three parts:
 
The query scope is the value which indexes the storage for a particular query type, it has the
bound `Scope: 'static + Eq + Hash" $(" + " stringify!($bound))? "`.

Each `Scope` corresponds to at most a single `Input: 'static" $(" + " stringify!($bound))? "`
and a single `Output: 'static" $(" + " stringify!($bound))? "` value at any given time.

# Reading stored values

See [`" stringify!($cache) "::get`] which accepts borrowed forms of `Scope`
and `Input`: `Key` and `Arg` respectively. `Arg` must satisfy `PartialEq<Input>` to determine
whether to return a stored output.

# Garbage Collection

Each time [`" stringify!($cache) "::gc`] is called it removes any values which haven't been
referenced since the prior call.

After each GC, all values still in the cache are marked garbage. They are marked live again when
inserted with [`" stringify!($cache) "::store`] or read with
[`" stringify!($cache) "::get`].
"=>
#[derive(Debug, Default)]
pub struct $cache {
    /// We use a [`hash_hasher::HashBuildHasher`] here because we know that `TypeId`s
    /// are globally unique and pre-hashed courtesy of rustc.
    inner: HashMap<TypeId, Box<dyn Gc $(+ $bound)?>, HashBuildHasher>,
}}

impl $cache {
doc_comment! {"
Return a reference to a query's stored output if a result is stored *and* `arg` equals the
previously-stored `Input`. If a reference is returned, the stored input/output
is marked as a root and will not be GC'd the next call.

If no reference is found, a [`CacheMiss`] is returned. Call [`CacheMiss::init`] to get
a [`CacheEntry`] to pass to [`" stringify!($cache) "::store`].
"=>
    pub fn get<'k, Key, Scope, Arg, Input, Output>(
        &self,
        key: &'k Key,
        arg: &Arg,
    ) -> Result<&Output, CacheMiss<'k, Key, Scope, Input, Output>>
    where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Arg: PartialEq<Input> + ?Sized,
        Input: 'static + Borrow<Arg> $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let query = Query::new(self.inner.hasher());
        if let Some(ns) = self.get_namespace(&query) {
            ns.get(key, arg).map_err(|h| CacheMiss { query, key: h })
        } else {
            Err(CacheMiss { query, key: KeyMiss::just_key(key) })
        }
    }}

doc_comment! {"
Stores a fresh [`CacheEntry`] whose input/output will not be GC'd at the next call.
Call [`" stringify!($cache) "::get`] to get a [`CacheMiss`] and [`CacheMiss::init`] to get a
[`CacheEntry`].
    "=>
    pub fn store<Key, Scope, Input, Output>(
        &mut self,
        entry: CacheEntry<'_, Key, Scope, Input, Output>,
    ) where
        Key: Eq + Hash + ToOwned<Owned = Scope> + ?Sized,
        Scope: 'static + Borrow<Key> + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let CacheEntry {
            miss: CacheMiss { query, key },
            input,
            output,
        } = entry;
        self.get_namespace_mut(&query).store(key, input, output);
    }}

    fn get_namespace<Scope, Input, Output>(
        &self,
        query: &Query<Scope, Input, Output>,
    ) -> Option<&Namespace<Scope, Input, Output>>
    where
        Scope: 'static + Eq + Hash $(+ $bound)?,
        Input: 'static $(+ $bound)?,
        Output: 'static $(+ $bound)?,
    {
        let gc: &(dyn Gc $(+ $bound)?) = &**self
            .inner
            .raw_entry()
            .from_hash(query.hash(), |t| t == &query.ty())?.1;
        Some(gc.as_any().downcast_ref().unwrap())
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

    /// Drop any values which have not been marked alive since the last call to this method.
    pub fn gc(&mut self) {
        self.sweep();
    }
}

impl Gc for $cache {
    fn sweep(&mut self) -> Liveness {
        self.inner.values_mut()
            .fold(Liveness::Dead, |l, namespace| {
                if namespace.sweep() == Liveness::Live {
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

For convenience wrappers around [`" stringify!($shared) "::cache_with`] see
[`" stringify!($shared) "::cache`] for returned types that implement
`Clone` and [`" stringify!($shared) "::hold`] for values that just need to be stored
without returning a reference.

# Example

```
let storage = dyn_cache::" stringify!($module) "::" stringify!($shared) r#"::default();
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
        let miss = match { self.inner.$acquire().get(key, arg) } {
            Ok(stored) => return with(stored),
            Err(m) => m,
        };

        let (to_store, to_return) = miss.init(arg.to_owned(), |arg| {
            let store = init(arg);
            let ret = with(&store);
            (store, ret)
        });

        self.inner.$acquire().store(to_store);
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
Forwards to [`" stringify!($cache) "::gc`].
"=>
    pub fn gc(&self) {
        self.inner.$acquire().sweep();
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
