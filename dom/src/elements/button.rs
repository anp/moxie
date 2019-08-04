use crate::{
    elements::*,
    events::{EventTarget, Handlers},
};

#[derive(Debug)]
pub struct Element<C = Empty> {
    ty: &'static str,
    attrs: Attrs,
    handlers: Handlers,
    children: C,
}

impl Element {
    pub fn new(ty: &'static str) -> Self {
        Self {
            ty,
            attrs: Default::default(),
            handlers: Default::default(),
            children: Empty,
        }
    }

    pub fn attr(mut self, key: &'static str, value: impl ToString) -> Self {
        self.attrs.inner.insert(key, value.to_string());
        self
    }
}

impl<C> EventTarget for Element<C>
where
    C: Component,
{
    fn handlers(&mut self) -> &mut Handlers {
        &mut self.handlers
    }
}

impl<C, Next> Parent<Next> for Element<C>
where
    C: Component,
    Next: Component,
{
    type Output = Element<SibList<C, Next>>;

    fn child(self, next: Next) -> Self::Output {
        let Self {
            attrs,
            ty,
            handlers,
            children,
        } = self;

        Element {
            attrs,
            ty,
            handlers,
            children: sib_cons(children, next),
        }
    }
}

impl<C> Component for Element<C>
where
    C: Component,
{
    fn contents(self) {
        let Self {
            ty,
            attrs,
            handlers,
            children,
        } = self;
        let element = document().create_element(ty).unwrap();

        for (key, val) in attrs.inner {
            element.set_attribute(key, &val).unwrap();
        }

        produce_dom!(element.clone(), handlers.apply(&element), || {
            show!(children);
        });
    }
}
