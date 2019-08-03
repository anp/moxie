use crate::{
    elements::*,
    events::{EventTarget, Handlers},
};

#[derive(Debug, Default)]
pub struct Button<C = Empty> {
    ty: ButtonType,
    handlers: Handlers,
    children: C,
}

impl Button {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<C> EventTarget for Button<C>
where
    C: Component,
{
    fn handlers(&mut self) -> &mut Handlers {
        &mut self.handlers
    }
}

impl<C, Next> Parent<Next> for Button<C>
where
    C: Component,
    Next: Component,
{
    type Output = Button<SibList<C, Next>>;

    fn child(self, next: Next) -> Self::Output {
        let Self {
            ty,
            handlers,
            children,
        } = self;

        Button {
            ty,
            handlers,
            children: sib_cons(children, next),
        }
    }
}

impl<C> Component for Button<C>
where
    C: Component,
{
    fn contents(self) {
        let Self {
            ty,
            handlers,
            children,
        } = self;
        let button = document().create_element("button").unwrap();
        match ty {
            ButtonType::Button => button.set_attribute("type", "button").unwrap(),
            _ => unimplemented!(),
        }

        produce_dom!(button.clone(), handlers.apply(&button), || {
            show!(children);
        });
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
