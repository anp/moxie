#![forbid(unsafe_code)]
#![deny(clippy::all, missing_docs)]

//! Caches for storing the results of repeated function calls. The cache types
//! available use minimal dynamic dispatch to allow storing arbitrarily many
//! types of query results in a single parent store.
//!
//! There are two main flavors of cache available for use in this crate:
//!
//! | Shared type                 | Synchronized? |
//! |-----------------------------|---------------|
//! | [`sync::SharedSendCache`]   | Mutex         |
//! | [`local::SharedLocalCache`] | RefCell       |
//!
//! These variants are used by calling [`sync::SharedSendCache::cache_with`] or
//! [`local::SharedLocalCache::cache`].
//!
//! The shared cache types above are implemented by wrapping these "inner"
//! types:
//!
//! | Mutable type          | Requires `Send`? |
//! |-----------------------|------------------|
//! | [`sync::SendCache`]   | yes              |
//! | [`local::LocalCache`] | no               |
//!
//! These "inner" caches require mutable access to call their functions like
//! [`local::LocalCache::get`] which returns either a reference or a
//! [`CacheMiss`] that can be passed back to the cache in
//! [`local::LocalCache::store`] to initialize a value in the cache.
//!
//! See [`sync::SendCache::get`] and [`sync::SendCache::store`] for the
//! thread-safe equivalents.
//!
//! The shared variants are defined by wrapping these inner cache types in
//! reference counting and synchronized mutability.
//!
//! # Query types
//!
//! Each query type maps to a typed "namespace" within the unityped cache
//! storage, each query having a distinct type each for its scope, input, and
//! output.
//!
//! ## Scopes
//!
//! The scope of a query is its identifier within cache storage.
//! Scopes must implement `Eq` and `Hash` so that results can be
//! efficiently and uniquely indexed within a namespace.
//!
//! Each scope identifies 0-1 `(Input, Output)` pairs in each namespace. The
//! same type of scope can be used in multiple queries without collision if
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
//! Every value in the cache has a "liveness" which is set to
//! "alive" when the value is first stored and again when it is read.
//!
//! The inner caches offer [`local::LocalCache::gc`] and [`sync::SendCache::gc`]
//! which are also exposed through [`local::SharedLocalCache::gc`] and
//! [`sync::SharedSendCache::gc`]. When called, the `gc()` method retains only
//! those values which are still "alive" and then marks them all "dead".
//!
//! This behavior resembles a simple mark-and-sweep garbage collector where the
//! "mark phase" is the use of the cache in between `gc()` calls. Any values
//! which weren't used in the mark phase are dropped in the next "sweep phase"
//! when `gc()` is called.
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
use hashbrown::hash_map::DefaultHashBuilder;
use illicit::AsContext;
use std::{
    any::TypeId,
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

mod cache_cell;
mod dep_node;
mod namespace;

#[macro_use] // put this after other modules so we don't accidentally depend on root-only macros
mod definition;

use namespace::{KeyMiss, Namespace};

/// The result of a failed attempt to retrieve a value from the cache.
/// Initialize a full [`CacheEntry`] for storage with [`CacheMiss::init`].
pub struct CacheMiss<'k, Key: ?Sized, Scope, Input, Output, H = DefaultHashBuilder> {
    query: Query<Scope, Input, Output>,
    key: KeyMiss<'k, Key, H>,
}

impl<'k, Key: ?Sized, Scope, Input, Output, H> CacheMiss<'k, Key, Scope, Input, Output, H> {
    /// Prepare the cache miss to be populated by running `query(arg)`,
    /// returning a separate value. The value returned (`R`) is typically
    /// derived in some way from the stored `Output`.
    pub fn init<R>(
        self,
        input: Input,
        query: impl FnOnce(&Input) -> (Output, R),
    ) -> (CacheEntry<'k, Key, Scope, Input, Output, H>, R) {
        self.key.dependent().offer(|| {
            let (output, to_return) = query(&input);
            (CacheEntry { output, input, miss: self }, to_return)
        })
    }
}

/// A fully-initialized input/output pair, ready to be written to the store.
pub struct CacheEntry<'k, Key: ?Sized, Scope, Input, Output, H = DefaultHashBuilder> {
    miss: CacheMiss<'k, Key, Scope, Input, Output, H>,
    input: Input,
    output: Output,
}

/// A cache for types which are not thread-safe (`?Send`).
pub mod local {
    use super::{dep_node::Dependent, *};
    use hash_hasher::HashBuildHasher;
    use hashbrown::HashMap;
    use std::{
        any::TypeId,
        borrow::Borrow,
        cell::RefCell,
        cmp::Eq,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::Hash,
        rc::Rc,
    };

    define_cache!(local, LocalCache, Rc, RefCell::borrow_mut);
}

/// A thread-safe cache which requires stored types implement `Send`.
pub mod sync {
    use super::{dep_node::Dependent, *};
    use hash_hasher::HashBuildHasher;
    use hashbrown::HashMap;
    use parking_lot::Mutex;
    use std::{
        any::TypeId,
        borrow::Borrow,
        cmp::Eq,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::Hash,
        sync::Arc,
    };

    define_cache!(sync, SendCache: Send, Arc, Mutex::lock);
}

/// A type which can contain values of varying liveness, including itself.
trait Gc: Downcast + Debug {
    /// Traverse stored values, identifying rooted dependents.
    fn mark(&mut self);

    /// Remove dead entries, returning the container's own status afterwards.
    fn sweep(&mut self) -> Liveness;
}

impl_downcast!(Gc);

/// Describes the outcome of garbage collection for a cached value.
#[derive(Debug, PartialEq)]
enum Liveness {
    /// The value is still live.
    Live,
    /// The value should be dropped.
    Dead,
}

/// The type of a dynamic cache query, used to shard storage in a fashion
/// similar to `anymap` or `typemap`.
struct Query<Scope, Input, Output, H = HashBuildHasher> {
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
