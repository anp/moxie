use crate::{
    elements::*,
    events::{EventTarget, Handlers},
    *,
};

#[derive(Debug, Default)]
pub struct Button<C = NilChild> {
    ty: ButtonType,
    events: Handlers,
    children: C,
}

impl Button {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<C: Component, Next: Component> Parent<Next> for Button<C> {
    type Output = Button<Sibs<C, Next>>;
    type Child = C;

    fn child(self, next: Next) -> Self::Output {
        let Self {
            ty,
            events,
            children,
        } = self;

        Button {
            ty,
            events,
            children: sib_cons(children, next),
        }
    }
}

impl<C> EventTarget for Button<C> {
    fn handlers(&mut self) -> &mut Handlers {
        &mut self.events
    }
}

impl<C> Component for Button<C>
where
    C: Component,
{
    fn contents(self) {
        let Self {
            ty,
            events,
            children,
        } = self;
        let button = web::document().create_element("button").unwrap();

        produce_dom!(button.clone(), || {
            show!(children);
        });

        events.apply(&button.as_node().clone().into());

        match ty {
            ButtonType::Button => button.set_attribute("type", "button").unwrap(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonType {
    Button,
    Submit,
    Reset,
    Menu,
}

impl Default for ButtonType {
    fn default() -> Self {
        ButtonType::Button
    }
}
