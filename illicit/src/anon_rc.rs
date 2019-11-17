use {
    owning_ref::OwningRef,
    std::{
        any::{Any, TypeId},
        fmt::Debug,
        rc::Rc,
    },
};

#[doc(hidden)]
#[derive(Clone)]
pub(crate) struct AnonRc {
    name: &'static str,
    id: TypeId,
    location: (&'static str, u32, u32),
    depth: u32,
    inner: Rc<dyn Any>,
    debug: Rc<dyn Debug>,
}

impl AnonRc {
    /// Construct a new `AnonArc` from the provided value.
    pub fn new<T: Debug + 'static>(
        inner: T,
        location: (&'static str, u32, u32),
        depth: u32,
    ) -> Self {
        let inner = Rc::new(inner);
        Self {
            name: std::any::type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            location,
            inner,
            depth,
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

    /// The `TypeId` of the contained value.
    pub fn id(&self) -> TypeId {
        self.id
    }

    /// The typename of the contained value.
    pub fn ty(&self) -> &str {
        self.name
    }

    /// Returns a debug-printable reference to the contained value.
    pub fn debug(&self) -> &dyn Debug {
        &*self.debug
    }

    /// The depth of the environment where this was created.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// The source location at which this was initialized for an environment.
    pub fn location(&self) -> (&'static str, u32, u32) {
        self.location
    }
}
