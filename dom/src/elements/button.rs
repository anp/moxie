use crate::{
    events::{EventTarget, Handlers},
    *,
};

#[derive(Debug, Default)]
pub struct Button {
    ty: ButtonType,
    events: Handlers,
}

impl Button {
    pub fn new() -> Self {
        Default::default()
    }
}

impl EventTarget for Button {
    fn handlers(&mut self) -> &mut Handlers {
        &mut self.events
    }
}

impl Component for Button {
    fn contents(self) {
        let button = web::document().create_element("button").unwrap();

        produce_dom!(button.clone(), || {
            show!(text!("increment"));
        });

        self.events.apply(&button.as_node().clone().into());

        match self.ty {
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
