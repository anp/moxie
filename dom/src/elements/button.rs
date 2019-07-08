use {
    crate::{elements::Text, events::Handlers, *},
    derive_builder::*,
    moxie::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
    stdweb::{traits::*, *},
    tracing::*,
};

#[derive(Builder, Debug, PartialEq)]
#[builder(pattern = "owned", setter(into))]
pub struct Button {
    #[builder(default)]
    ty: ButtonType,
    #[builder(default)]
    events: Handlers,
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

impl Button {
    pub fn new() -> ButtonBuilder {
        ButtonBuilder::default()
    }
}

impl Component for Button {
    fn contents(self) {
        let button = web::document().create_element("button").unwrap();

        produce_dom!(button.clone(), || {
            show!(text!("increment"));
        });

        if let Some(mut click) = self.events.click {
            button.add_event_listener(move |_: web::event::ClickEvent| click());
        }

        match self.ty {
            ButtonType::Button => button.set_attribute("type", "button").unwrap(),
            _ => unimplemented!(),
        }
    }
}
