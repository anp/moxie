use {
    crate::{
        elements::*,
        events::{Event, Handler, Target},
    },
    std::fmt::Debug,
};

#[derive(Debug, Default)]
pub struct Button<C = Empty, H = Empty> {
    ty: ButtonType,
    handlers: H,
    children: C,
}

impl Button {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<C, H, Next> Parent<Next> for Button<C, H>
where
    C: Component,
    H: Debug + 'static,
    Next: Component,
{
    type Output = Button<SibList<C, Next>, H>;

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

impl<C, H, Ev, State, Updater> Target<Ev, State, Updater> for Button<C, H>
where
    C: Component,
    Ev: Event,
    H: Debug + 'static,
    State: Debug + 'static,
    Updater: FnMut(&State, Ev) -> Option<State> + 'static,
{
    type Output = Button<C, SibList<H, Handler<State, Updater>>>;
    fn on(self, key: Key<State>, updater: Updater) -> Self::Output {
        unimplemented!()
    }
}

impl<C, H> Component for Button<C, H>
where
    C: Component,
    H: Debug + 'static,
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

        produce_dom!(button.clone(), || {
            show!(children);
        });

        // handlers.apply(&button);
        unimplemented!()
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
