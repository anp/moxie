use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        fmt::Debug,
        ops::Deref,
        sync::Arc,
    },
};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct AnonArc {
    name: &'static str,
    id: TypeId,
    inner: Arc<dyn Any>,
    debug: Arc<dyn Debug>,
}

impl AnonArc {
    /// The typename of the contained value.
    pub fn ty(&self) -> &str {
        self.name
    }

    /// Returns a debuggable representation of the contained value.
    pub fn debug(&self) -> &dyn std::fmt::Debug {
        &*self.debug
    }

    #[doc(hidden)]
    pub fn unstable_new<T: Debug + 'static>(inner: T) -> Self {
        let inner = Arc::new(inner);
        Self {
            name: std::any::type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            inner,
        }
    }

    #[doc(hidden)]
    pub fn unstable_insert_into(self, env: &mut super::Env) {
        env.values.insert(self.id, self);
    }

    #[doc(hidden)]
    // FIXME this should probably expose a fallible api somehow?
    pub fn unstable_deref<T: 'static>(self) -> impl Deref<Target = T> + 'static {
        OwningRef::new(self.inner).map(|anon| {
            anon.downcast_ref().unwrap_or_else(|| {
                panic!("asked {:?} to cast to {:?}", anon, TypeId::of::<T>(),);
            })
        })
    }
}

impl Deref for AnonArc {
    type Target = dyn Any;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
