use {
    moxie_dom::{elements::*, events::*, *},
    stdweb::{traits::*, *},
    tracing::*,
};

#[derive(Clone, Debug, PartialEq)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(self) {
        let (count, count_key) = state!((), |()| 0);

        show!(text!("hello world from moxie! ({})", count));

        show!(Button::new().on_click(count_key, |c, _| Some(c + 1)));
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
