use {
    derive_builder::*,
    log::*,
    moxie_dom::*,
    std::fmt::{Debug, Formatter, Result as FmtResult},
    stdweb::{traits::*, *},
};

#[derive(Clone, Debug, PartialEq)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(self) {
        let (count, count_key) = state!((), |()| 0);

        show!(Text(format!("hello world from moxie! ({})", count)));

        show!(Button::new()
            .on(events()
                .set_click(move || {
                    let revision = count_key.update(|c| Some(c + 1));
                    info!("clicked, updated at revision {:?}", revision);
                })
                .build()
                .unwrap())
            .build()
            .unwrap());
    }
}

#[derive(Debug, PartialEq)]
struct Text(String);

impl Component for Text {
    fn contents(self) {
        let text_node = web::document().create_text_node(&self.0);
        produce_dom!(text_node, || {});
    }
}

#[derive(Builder, Debug, PartialEq)]
#[builder(pattern = "owned", setter(into))]
struct Button {
    #[builder(default)]
    ty: ButtonType,
    #[builder(default)]
    on: Handlers,
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
    fn new() -> ButtonBuilder {
        ButtonBuilder::default()
    }
}

impl Component for Button {
    fn contents(self) {
        let button = web::document().create_element("button").unwrap();

        produce_dom!(button.clone(), || {
            show!(Text("increment".into()));
        });

        if let Some(mut click) = self.on.click {
            button.add_event_listener(move |_: web::event::ClickEvent| click());
        }

        match self.ty {
            ButtonType::Button => button.set_attribute("type", "button").unwrap(),
            _ => unimplemented!(),
        }
    }
}

pub fn events() -> HandlersBuilder {
    HandlersBuilder::default()
}

#[derive(Builder, Default)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct Handlers {
    click: Option<Box<dyn FnMut() + 'static>>,
}

impl HandlersBuilder {
    pub fn set_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.click = Some(Some(Box::new(f)));
        self
    }
}

impl Debug for Handlers {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Handlers")
            .field("click", &self.click.as_ref().map(|_| "..."))
            .finish()
    }
}

impl PartialEq for Handlers {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

fn main() {
    web_logger::init();
    std::panic::set_hook(Box::new(|info| {
        error!("{:#?}", info);
    }));

    let root = web::document().create_element("div").unwrap();
    web::document().body().unwrap().append_child(&root);

    info!("mounting moxie-dom to root");
    moxie_dom::mount!(root, HackedApp);
}
