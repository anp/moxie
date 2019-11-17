use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        fmt::Debug,
        sync::Arc,
    },
};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub(crate) struct AnonArc {
    name: &'static str,
    id: TypeId,
    inner: Arc<dyn Any>,
    debug: Arc<dyn Debug>,
}

impl AnonArc {
    /// The `TypeId` of the contained value.
    pub fn id(&self) -> TypeId {
        self.id
    }

    /// The typename of the contained value.
    pub fn ty(&self) -> &str {
        self.name
    }

    /// Returns a debuggable representation of the contained value.
    pub fn debug(&self) -> &dyn std::fmt::Debug {
        &*self.debug
    }

    /// Construct a new `AnonArc` from the provided value.
    pub fn new<T: Debug + 'static>(inner: T) -> Self {
        let inner = Arc::new(inner);
        Self {
            name: std::any::type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            inner,
        }
    }

    // FIXME this should probably expose a fallible api somehow?
    pub fn downcast_deref<T: 'static>(self) -> impl std::ops::Deref<Target = T> + 'static {
        OwningRef::new(self.inner).map(|anon| {
            anon.downcast_ref().unwrap_or_else(|| {
                panic!("asked {:?} to cast to {:?}", anon, TypeId::of::<T>(),);
            })
        })
    }
}
