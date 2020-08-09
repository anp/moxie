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
//! | Allocation                         | Causes        |
//! |------------------------------------|---------------|
//! | box a new, empty namespace         | (1)           |
//! | resize a cache's map of namespaces | (1)           |
//! | call `.to_owned()` on a scope/key  | (2)           |
//! | resize a namespace's storage       | (2)           |
//! | call `.to_owned()` on an input/arg | (2), (3)      |
//! | update an output's dependents      | (1), (2), (3) |
//!
//! Outside of these, only user-defined functions should perform any allocation.
//!
//! # Garbage Collection
//!
//! All of the caches have a `gc()` method which retains only used values. A
//! value is used if it or a value which depends on it has been used/rooted
//! since the last call to `gc()`.
//!
//! ## Rooting
//!
//! When a cache read *fails*, we expect that the value will be populated
//! immediately after and a new node in the dependency graph is created. The new
//! dependency node is marked as an incoming dependent on any cache values which
//! are accessed during the initialization of the new value. The new node is
//! then marked as a "root" for the garbage collector once it has
//! been initialized and the cache populated. If in subsequent epochs the rooted
//! value is accessed again it will be re-rooted and its dependents will be
//! marked as live even if they were not directly accessed in that epoch.
//!
//! When a cache read *succeeds*, its dependency node is marked as being
//! depended upon by the node (if any) which was being initialized during the
//! read, linking the two dependencies together.

use downcast_rs::{impl_downcast, Downcast};
use hash_hasher::HashBuildHasher;
use hashbrown::hash_map::DefaultHashBuilder;
use std::{
    any::TypeId,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

#[macro_use]
mod definition;

mod cache_cell;
mod dep_node;
mod namespace;

use namespace::{KeyMiss, Namespace};

/// The result of a failed attempt to retrieve a value from the cache.
/// Initialize a full [`CacheEntry`] for storage with [`CacheMiss::init`].
#[derive(Clone, Eq, PartialEq)]
pub struct CacheMiss<'k, Key: ?Sized, Scope, Input, Output, H = DefaultHashBuilder> {
    query: Query<Scope, Input, Output>,
    key_miss: KeyMiss<'k, Key, Input, H>,
}

impl<'k, Key: ?Sized, Scope, Input, Output, H> CacheMiss<'k, Key, Scope, Input, Output, H> {
    /// Prepare the cache miss to be populated by running `query(arg)`,
    /// returning a separate value. The value returned (`R`) is typically
    /// derived in some way from the stored `Output`.
    pub fn init<R>(
        self,
        query: impl FnOnce(&Input) -> (Output, R),
    ) -> (CacheEntry<'k, Key, Scope, Input, Output, H>, R) {
        let (output, to_return) = self.key_miss.init(query);
        (CacheEntry { output, miss: self }, to_return)
    }
}

impl<'k, Key, Scope, Input, Output, H> Debug for CacheMiss<'k, Key, Scope, Input, Output, H>
where
    Key: Debug + ?Sized,
    Scope: Debug,
    Input: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("CacheMiss")
            .field("query", &self.query)
            .field("key_miss", &self.key_miss)
            .finish()
    }
}

/// A fully-initialized input/output entry, ready to be written to the cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry<'k, Key: ?Sized, Scope, Input, Output, H = DefaultHashBuilder> {
    miss: CacheMiss<'k, Key, Scope, Input, Output, H>,
    output: Output,
}

/// A cache for types which are not thread-safe (`?Send`).
pub mod local {
    use std::{cell::RefCell, rc::Rc};

    define_cache!(local, LocalCache, Rc, RefCell::borrow_mut);
}

/// A thread-safe cache which requires stored types implement `Send`.
pub mod sync {
    use parking_lot::Mutex;
    use std::sync::Arc;

    define_cache!(sync, SendCache: Send, Arc, Mutex::lock);
}

/// A type which can contain values of varying liveness.
trait Storage: Downcast + Debug {
    /// Traverse stored values, identifying roots.
    fn mark(&mut self);

    /// Remove dead entries.
    fn sweep(&mut self);
}

impl_downcast!(Storage);

/// Describes the outcome of garbage collection for a cached value.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Liveness {
    /// The value is still live.
    Live,
    /// The value should be dropped.
    Dead,
}

/// The type of a dynamic cache query, used to shard storage in a fashion
/// similar to `anymap` or `typemap`.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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

impl<Scope, Input, Output, H> Debug for Query<Scope, Input, Output, H> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Query")
            .field("ty", &std::any::type_name::<(Scope, Input, Output)>())
            .field("hasher", &std::any::type_name::<H>())
            .field("hash", &self.hash)
            .finish()
    }
}
