use {
    crate::{
        events::{EventTarget, Handlers},
        *,
    },
    std::collections::BTreeMap,
};

#[macro_export]
macro_rules! text {
    ($arg:expr) => {
        $crate::elements::Text($arg.to_string())
    };
    ($($arg:tt)+) => {
        $crate::elements::Text(format!( $($arg)* ))
    };
}

#[derive(Debug, PartialEq)]
pub struct Text(pub String);

impl Component for Text {
    fn contents(self) {
        let text_node = memo!(self.0, |text| document().create_text_node(text));
        produce_dom!(text_node, vec![], || {});
    }
}

#[derive(Debug, Default)]
struct Attrs {
    inner: BTreeMap<&'static str, String>,
}

#[derive(Debug)]
pub struct Element<C = Empty> {
    ty: &'static str,
    attrs: Attrs,
    handlers: Handlers,
    children: C,
}

pub fn element(ty: &'static str) -> Element {
    Element {
        ty,
        attrs: Default::default(),
        handlers: Default::default(),
        children: Empty,
    }
}

impl Element {
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

impl<C> Element<C>
where
    C: Component,
{
    pub fn inner<F>(self, f: F) -> Element<SibList<C, Clomp<F>>>
    where
        F: FnOnce(),
    {
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
            children: sib_cons(children, Clomp(f)),
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
        let element = once!(|| document().create_element(ty).unwrap());

        // TODO think about a "memo by map key" api
        let existing_attrs = element.attributes();
        let existing_attrs = (0..existing_attrs.length())
            .map(|i| existing_attrs.item(i).unwrap())
            .collect::<Vec<_>>();
        for attr in existing_attrs {
            let attr_name = attr.name();
            if !attrs.inner.contains_key(&*attr_name) {
                // element.remove_attribute_node(&attr).unwrap();
            }
        }

        for pair in attrs.inner {
            memo!(pair, |(name, value)| element
                .set_attribute(name, value)
                .unwrap());
        }

        let event_handler_dot_dot_dot_handles = handlers.apply(&element);

        produce_dom!(element, event_handler_dot_dot_dot_handles, || {
            show!(children);
        });
    }
}
