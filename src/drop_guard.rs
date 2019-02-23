/// A `DropGuard` runs a user-provided "destructor" on arbitrary data.
///
/// The primary advantage of this implementation over some on crates.io is
/// that the concrete type of the destructor is erased, allowing for
/// straightforward heterogeneous storage.
pub(crate) struct DropGuard<Inner> {
    inner: Option<Inner>,
    dtor: Option<Box<dyn FnMut(Inner)>>,
}

impl<Inner> DropGuard<Inner> {
    pub(crate) fn new(inner: Inner, dtor: impl FnMut(Inner) + 'static) -> Self {
        Self {
            inner: Some(inner),
            dtor: Some(Box::new(dtor)),
        }
    }
}

impl<Inner> std::ops::Deref for DropGuard<Inner> {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<Inner> std::ops::DerefMut for DropGuard<Inner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<Inner> Drop for DropGuard<Inner> {
    fn drop(&mut self) {
        let mut inner = None;
        let mut dtor = None;
        std::mem::swap(&mut inner, &mut self.inner);
        std::mem::swap(&mut dtor, &mut self.dtor);
        dtor.as_mut().unwrap()(inner.unwrap());
    }
}
