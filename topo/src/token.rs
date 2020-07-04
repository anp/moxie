use crate::cache::Cache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

static TOKENS: Lazy<Mutex<Cache>> = Lazy::new(|| Mutex::new(Cache::default()));

/// A unique identifer in the global cache. Each type can have
/// [`std::usize::MAX`] unique values cached.
pub struct Token<T: 'static> {
    index: usize,
    ty: PhantomData<T>,
}

impl<S> Token<S>
where
    S: Eq + Hash + Send + 'static,
{
    /// Get the token for the provided slot.
    pub fn get<Q>(slot: &Q) -> Token<S>
    where
        Q: Eq + Hash + ToOwned<Owned = S> + ?Sized,
        S: Borrow<Q>,
    {
        let mut existing_tokens = TOKENS.lock();

        if let Some(token) = existing_tokens.get::<_, S, _, _, _>(slot, &()) {
            *token
        } else {
            let new_token = Self::next();
            existing_tokens.store(slot.to_owned(), (), new_token);
            new_token
        }
    }

    /// Get the next Token for this type.
    fn next() -> Self {
        static INDICES: Lazy<Mutex<HashMap<TypeId, usize>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));

        let mut indices = INDICES.lock();
        let count = indices.entry(TypeId::of::<S>()).or_default();
        *count += 1;
        Self { index: *count, ty: PhantomData }
    }

    /// Fabricate a token. Used for creating a root `crate::Id`.
    pub(crate) fn fake() -> Self {
        Self { index: usize::max_value(), ty: PhantomData }
    }

    /// Erase the type of this token, storing it as a [`TypeId`] in the
    /// resulting [`OpaqueToken`].
    pub fn opaque(self) -> OpaqueToken {
        OpaqueToken { index: self.index, ty: TypeId::of::<S>() }
    }
}

impl<T> Clone for Token<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, ty: PhantomData }
    }
}

impl<T> Copy for Token<T> {}

impl<T> Debug for Token<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Token").field("index", &self.index).field("ty", &type_name::<T>()).finish()
    }
}

impl<T> Hash for Token<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher)
    }
}

impl<T> PartialEq for Token<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Token<T> {}

impl<T> PartialOrd for Token<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}
impl<T> Ord for Token<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

/// A unique type-erased identifier in the global cache.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct OpaqueToken {
    ty: TypeId,
    index: usize,
}
