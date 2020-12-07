use dyn_cache::sync::SendCache;
use once_cell::sync::Lazy;
use simple_mutex::Mutex;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

static TOKENS: Lazy<Mutex<SendCache>> = Lazy::new(|| Mutex::new(SendCache::default()));

/// A unique identifer in the global cache. Each type can have
/// [`std::u32::MAX`] unique values cached. Constructed with [`Token::make`],
/// which will always produce the same value for the same input.
///
/// # Memory Usage
///
/// Token inputs are not yet dropped and care should be taken when creating
/// large numbers of them, as the memory used over time will grow with
/// proportion to the number of unique tokens created.
///
/// See [issue #141](https://github.com/anp/moxie/issues/141) for future work.
///
/// A typed token can be converted into an [`OpaqueToken`] to allow
/// differentiating between unique values of different types.
pub(crate) struct Slot<T> {
    index: u32,
    ty: PhantomData<T>,
}

impl<T> Slot<T>
where
    T: Eq + Hash + Send + 'static,
{
    /// Makes a unique token from the provided value, interning it in the global
    /// cache. Later calls with the same input will return the same token.
    pub fn make<Q>(value: &Q) -> Slot<T>
    where
        Q: Eq + Hash + ToOwned<Owned = T> + ?Sized,
        T: Borrow<Q>,
    {
        static INDICES: Lazy<Mutex<HashMap<TypeId, u32>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));
        let mut existing_tokens = TOKENS.lock();

        match existing_tokens.get(value, &()) {
            Ok(token) => *token,
            Err(miss) => {
                let (to_store, new_token) = miss.init(|_| {
                    let mut indices = INDICES.lock();
                    let count = indices.entry(TypeId::of::<T>()).or_default();
                    *count += 1;
                    let new_token = Self { index: *count, ty: PhantomData };
                    (new_token, new_token)
                });
                existing_tokens.store(to_store);
                new_token
            }
        }
    }

    /// Fabricate a token. Used for e.g. creating a root `crate::CallId`.
    pub(crate) fn fake() -> Self {
        Self { index: 0, ty: PhantomData }
    }
}

impl<T> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, ty: PhantomData }
    }
}

impl<T> Copy for Slot<T> {}

impl<T> Debug for Slot<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Token").field("index", &self.index).field("ty", &type_name::<T>()).finish()
    }
}

impl<Q, T> From<&Q> for Slot<T>
where
    Q: Eq + Hash + ToOwned<Owned = T> + ?Sized,
    T: Borrow<Q> + Eq + Hash + Send + 'static,
{
    fn from(query: &Q) -> Self {
        Slot::make(query)
    }
}

impl<T> Hash for Slot<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher)
    }
}

impl<T> PartialEq for Slot<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Slot<T> {}

impl<T> PartialOrd for Slot<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl<T> Ord for Slot<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

/// A unique type-erased identifier for a cached value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct OpaqueSlot {
    ty: TypeId,
    index: u32,
}

impl<T: 'static> From<Slot<T>> for OpaqueSlot {
    fn from(token: Slot<T>) -> Self {
        OpaqueSlot { index: token.index, ty: TypeId::of::<T>() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_tokens() {
        let foo: Slot<String> = Slot::make("foo");
        assert_eq!(foo, Slot::make("foo"));
        assert_ne!(foo, Slot::make("bar"));
    }

    #[test]
    fn make_opaque() {
        let first: OpaqueSlot = Slot::make(&10u8).into();
        let second: OpaqueSlot = Slot::make(&10u16).into();
        assert_ne!(first, second);
    }
}
