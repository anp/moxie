use {
    log::*,
    moxie_dom::*,
    stdweb::{traits::*, *},
};

#[derive(Clone, Debug, PartialEq)]
struct HackedApp;

impl Component for HackedApp {
    fn contents(&self) {
        let (count, count_key) = state!((), |()| 0);

        let message = format!("hello world from moxie! ({})", count);

        memo!(message, |msg| {
            let text_node = web::document().create_text_node(msg);
            produce_dom!(text_node, || {});
        });

        once!(|| {
            let button = web::document().create_element("button").unwrap();
            button.set_attribute("type", "button").unwrap();

            button.add_event_listener(move |_: web::event::ClickEvent| {
                let revision = count_key.update(|c| {
                    let new_count = c + 1;
                    Some(new_count)
                });
                info!("clicked, updated at revision {:?}", revision);
            });

            produce_dom!(button, || {
                once!(|| {
                    produce_dom!(web::document().create_text_node("increment"), || {});
                });
            });
        });
    }
}

fn main() {
    web_logger::init();
    std::panic::set_hook(Box::new(|info| {
        error!("{:#?}", info);
    }));

    let body = web::document().body().unwrap().as_node().to_owned();
    let root = web::document().create_element("div").unwrap();
    body.append_child(&root);

    info!("mounting moxie-dom to root");
    moxie_dom::mount!(root.as_node().to_owned(), HackedApp);
}
