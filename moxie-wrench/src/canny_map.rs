//! A `CannyMap` is a "concurrent [anymap](https://crates.io/crates/anymap)" built with
//! [chashmap](https://crates.io/crates/chashmap).

use {
    chashmap::CHashMap,
    std::any::{Any, TypeId},
};

/// A concurrent anymap. It holds 0-1 values of any type, and allows both read & write access to
/// them concurrently by multiple threads.
#[derive(Debug, Default)]
pub struct CannyMap {
    inner: CHashMap<TypeId, Box<Any + 'static>>,
}

impl CannyMap {
    /// Create an empty collection.
    fn new() -> Self {
        Default::default()
    }

    /// Creates an empty collection with the given initial capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: CHashMap::with_capacity(cap),
        }
    }

    /// Returns the number of elements the collection can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the collection.
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows usize.
    pub fn reserve(&self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Shrinks the capacity of the collection as much as possible. It will drop down as much as
    /// possible while maintaining the internal rules and possibly leaving some space in accordance
    /// with the resize policy.
    pub fn shrink_to_fit(&self) {
        self.inner.shrink_to_fit();
    }

    /// Returns the number of items in the collection.
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if there are no items in the collection.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all items from the collection. Keeps the allocated memory for reuse.
    fn clear(&self) {
        self.inner.clear();
    }

    /// Returns a reference to the value stored in the collection for the type T, if it exists.
    pub fn get<T: 'static>(&self) -> Option<ReadGuard<T>> {
        let id = TypeId::of::<T>();
        self.inner.get(&id).map(ReadGuard::from_raw)
    }

    /// Returns a mutable reference to the value stored in the collection for the type T, if it
    /// exists.
    pub fn get_mut<T: 'static>(&self) -> Option<WriteGuard<T>> {
        let id = TypeId::of::<T>();
        self.inner.get_mut(&id).map(WriteGuard::from_raw)
    }

    /// Sets the value stored in the collection for the type T. If the collection already had a
    /// value of type T, that value is returned. Otherwise, None is returned.
    pub fn insert<T: 'static>(&self, value: T) -> Option<T> {
        self.inner
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(|a| *(a.downcast().unwrap()))
    }

    /// Removes the T value from the collection, returning it if there was one or None if there was
    /// not.
    pub fn remove<T: 'static>(&self) -> Option<T> {
        let id = TypeId::of::<T>();
        self.inner.remove(&id).map(|a| *(a.downcast().unwrap()))
    }

    /// Returns true if the collection contains a value of type T.
    pub fn contains<T: 'static>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.inner.contains_key(&id)
    }

    /// Insert or update. If the key exists, the reference to its value is passed through to
    /// `update`. If it doesn't exist, the result of `insert` is inserted.
    pub fn upsert<T: 'static>(&self, inserter: impl FnOnce() -> T, updater: impl FnOnce(&mut T)) {
        self.inner.upsert(
            TypeId::of::<T>(),
            || Box::new(inserter()),
            |value| updater(value.downcast_mut().unwrap()),
        );
    }

    /// Map or insert an entry.
    ///
    /// This sets the value associated with key key to `mutator(Some(old_val))` (if it returns None,
    /// the entry is removed) if it exists. If it does not exist, it inserts it with value f(None),
    /// unless the closure returns None.
    ///
    /// Note that if `mutator` returns None, the entry of key key is removed unconditionally.
    pub fn alter<T: 'static>(&self, mutator: impl FnOnce(Option<T>) -> Option<T>) {
        self.inner.alter(TypeId::of::<T>(), |maybe_val| {
            mutator(maybe_val.map(|v| *v.downcast().unwrap())).map(|v| Box::new(v) as Box<Any>)
        })
    }
}

pub struct ReadGuard<'map, T: 'static> {
    inner: chashmap::ReadGuard<'map, TypeId, Box<Any + 'static>>,
    __ty: std::marker::PhantomData<T>,
}

pub struct WriteGuard<'map, T: 'static> {
    inner: chashmap::WriteGuard<'map, TypeId, Box<Any + 'static>>,
    __ty: std::marker::PhantomData<T>,
}

impl<'map, T: 'static> ReadGuard<'map, T> {
    fn from_raw(inner: chashmap::ReadGuard<'map, TypeId, Box<Any + 'static>>) -> Self {
        Self {
            inner,
            __ty: std::marker::PhantomData,
        }
    }
}

impl<'map, T: 'static> WriteGuard<'map, T> {
    fn from_raw(inner: chashmap::WriteGuard<'map, TypeId, Box<Any + 'static>>) -> Self {
        Self {
            inner,
            __ty: std::marker::PhantomData,
        }
    }
}

impl<'map, T> std::ops::Deref for ReadGuard<'map, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref().unwrap()
    }
}

impl<'map, T> std::ops::Deref for WriteGuard<'map, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.downcast_ref().unwrap()
    }
}

impl<'map, T: 'static> std::ops::DerefMut for WriteGuard<'map, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.downcast_mut().unwrap()
    }
}
