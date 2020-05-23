use owning_ref::OwningRef;
use std::{
    any::{type_name, Any, TypeId},
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::Deref,
    rc::Rc,
};

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
            name: type_name::<T>(),
            id: TypeId::of::<T>(),
            debug: inner.clone(),
            location,
            inner,
            depth,
        }
    }

    pub(crate) fn downcast_deref<T: 'static>(
        self,
    ) -> Result<impl Deref<Target = T> + 'static, impl Debug> {
        let from = self.name;
        OwningRef::new(self.inner)
            .try_map(|anon| anon.downcast_ref().ok_or(DowncastError { from, to: type_name::<T>() }))
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

/// compare the pointers by their data locations, ignoring vtables
macro_rules! data_ptrs_eq {
    ($trt:ident: $anon1:expr, $anon2:expr) => {
        std::ptr::eq(
            $anon1 as &dyn $trt as *const dyn $trt as *const u8,
            $anon2 as &dyn $trt as *const dyn $trt as *const u8,
        )
    };
}

impl PartialEq for AnonRc {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.depth == other.depth
            && self.name == other.name
            && self.location == other.location
            && data_ptrs_eq!(Any: &self.inner, &other.inner)
            && data_ptrs_eq!(Debug: &self.debug, &other.debug)
    }
}
impl Eq for AnonRc {}

struct DowncastError {
    from: &'static str,
    to: &'static str,
}

impl Debug for DowncastError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "asked {:?} to cast to {:?}", self.from, self.to)
    }
}
