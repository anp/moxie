use owning_ref::OwningRef;
use std::{
    any::{type_name, Any, TypeId},
    fmt::Debug,
    ops::Deref,
    panic::Location,
    rc::Rc,
};

#[derive(Clone)]
pub(crate) struct AnonRc {
    name: &'static str,
    id: TypeId,
    location: &'static Location<'static>,
    depth: u32,
    inner: Rc<dyn Any>,
    debug: Rc<dyn Debug>,
}

impl AnonRc {
    /// Construct a new `AnonArc` from the provided value.
    pub fn new<T: Debug + 'static>(
        inner: T,
        location: &'static Location<'static>,
        depth: u32,
    ) -> Self {
        let inner = Rc::new(inner);
        Self {
            name: type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            location,
            inner,
            depth,
        }
    }

    pub(crate) fn downcast_deref<T: 'static>(self) -> Option<impl Deref<Target = T> + 'static> {
        OwningRef::new(self.inner)
            .try_map(|anon| {
                let res: Result<&T, &str> = anon.downcast_ref().ok_or("invalid cast");
                res
            })
            .ok()
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
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }
}
