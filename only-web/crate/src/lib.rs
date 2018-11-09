#[macro_use]
extern crate cfg_if;

extern crate maplit;
extern crate only;
extern crate wasm_bindgen;
extern crate web_sys;

use std::collections::{BTreeMap, HashMap};

use maplit::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Node};

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// Called by our JS entry point to run the example
#[wasm_bindgen]
pub fn run() {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let body: &web_sys::Node = body.as_ref();

    let element = elements::Standard {
        ty: "div".to_string(),
        children: vec![
            Box::new(elements::Standard {
                ty: "input".to_string(),
                props: btreemap! {
                    "value".to_string() => "foo".to_string(),
                    "type".to_string() => "text".to_string(),
                },
                ..Default::default()
            }),
            Box::new(elements::Standard {
                ty: "a".to_string(),
                props: btreemap! {
                    "href".to_string() => "/bar".to_string(),
                },
                ..Default::default()
            }),
            Box::new(elements::Standard {
                ty: "span".to_string(),
                children: vec![Box::new(elements::Text {
                    contents: "hello world".to_string(),
                })],
                ..Default::default()
            }),
        ],
        props: btreemap! {
            "id".to_string() => "container".to_string(),
        },
        ..Default::default()
    };
    element.render(&document, body);
}

pub trait Element {
    fn render(&self, document: &Document, parent_dom: &Node);
}

mod elements {
    use super::*;
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::Event;

    #[derive(Default)]
    pub struct Standard {
        pub ty: String,
        pub children: Vec<Box<dyn Element>>,
        pub props: BTreeMap<String, String>,
        pub listeners: Listeners,
    }

    impl Element for Standard {
        fn render(&self, document: &Document, parent_dom: &Node) {
            let dom = document.create_element(&self.ty).unwrap();

            self.listeners.register(dom.as_ref());

            for (property, value) in &self.props {
                dom.set_attribute(&property, &value).unwrap();
            }

            let dom_node: Node = dom.into();
            self.children
                .iter()
                .for_each(|child| child.render(document, &dom_node));

            parent_dom.append_child(&dom_node).unwrap();
        }
    }

    #[derive(Default)]
    pub struct Listeners {
        pub listeners: HashMap<String, Closure<Fn(Event)>>,
    }

    impl Listeners {
        fn register(&self, et: &web_sys::EventTarget) {
            for (event_type, cb) in &self.listeners {
                et.add_event_listener_with_callback(&event_type, cb.as_ref().unchecked_ref())
                    .unwrap();
            }
        }
    }

    impl Drop for Listeners {
        fn drop(&mut self) {
            // FIXME(anp): we need to xfer ownership to the imported add event listener fn
            // so that the callbacks can be correctly destroyed
            for (_, callback) in self.listeners.drain() {
                callback.forget();
            }
        }
    }

    pub struct Text {
        pub contents: String,
    }

    impl Element for Text {
        fn render(&self, document: &Document, parent_dom: &Node) {
            let text: Node = document.create_text_node(&self.contents).into();

            // TODO make this the responsibility of the "renderer"
            parent_dom.append_child(&text).unwrap();
        }
    }
}
