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

    let element = Element::Standard(StdElem {
        ty: "div".to_string(),
        children: vec![
            Element::Standard(StdElem {
                ty: "input".to_string(),
                props: btreemap! {
                    "value".to_string() => "foo".to_string(),
                    "type".to_string() => "text".to_string(),
                },
                ..Default::default()
            }),
            Element::Standard(StdElem {
                ty: "a".to_string(),
                props: btreemap! {
                    "href".to_string() => "/bar".to_string(),
                },
                ..Default::default()
            }),
            Element::Standard(StdElem {
                ty: "span".to_string(),
                children: vec![Element::Text("hello world now".to_string())],
                ..Default::default()
            }),
        ],
        props: btreemap! {
            "id".to_string() => "container".to_string(),
        },
        ..Default::default()
    });

    element.render(&document, body);
}

pub enum Element {
    Standard(StdElem),
    Text(String),
}

#[derive(Default)]
pub struct StdElem {
    ty: String,
    children: Vec<Element>,
    props: BTreeMap<String, String>,
    // listeners: Listeners,
}

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::Event;

impl Element {
    fn render(&self, document: &Document, parent_dom: &Node) {
        match self {
            Element::Standard(StdElem {
                ty,
                children,
                props,
                // listeners,
            }) => {
                let dom = document.create_element(&ty).unwrap();

                // listeners.register(dom.as_ref());

                for (property, value) in props {
                    dom.set_attribute(property, value).unwrap();
                }

                let dom_node: Node = dom.into();
                children
                    .iter()
                    .for_each(|child| child.render(document, &dom_node));
                parent_dom.append_child(&dom_node).unwrap();
            }
            Element::Text(contents) => {
                let text: Node = document.create_text_node(&contents).into();

                // TODO make this the responsibility of the "renderer"
                parent_dom.append_child(&text).unwrap();
            }
        }
    }
}

// #[derive(Default)]
// pub struct Listeners {
//     pub listeners: HashMap<String, Closure<Fn(Event)>>,
// }

// impl Listeners {
//     fn register(&self, et: &web_sys::EventTarget) {
//         for (event_type, cb) in &self.listeners {
//             et.add_event_listener_with_callback(&event_type, cb.as_ref().unchecked_ref())
//                 .unwrap();
//         }
//     }
// }

// impl Drop for Listeners {
//     fn drop(&mut self) {
//         // FIXME(anp): we need to xfer ownership to the imported add event listener fn
//         // so that the callbacks can be correctly destroyed
//         for (_, callback) in self.listeners.drain() {
//             callback.forget();
//         }
//     }
// }
