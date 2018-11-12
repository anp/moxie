#[macro_use]
extern crate cfg_if;

use std::collections::BTreeMap;

use maplit::*;
use moxie::mox;
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

// Called by our JS entry point
#[wasm_bindgen]
pub fn run() {
    use std::borrow::Borrow;
    let document: Document = web_sys::window().unwrap().document().unwrap();
    let body = document.borrow().body().unwrap();
    let body: &web_sys::Node = body.as_ref();

    let element = mox! {
        <div id="container">
            <input value="foo" type="text"/>
            <a href="/bar"/>
            <span>hello world now</span>
        </div>
    };

    let two = mox! { <div><br />7x invalid-js-identifier</div> };

    // render(element, &body);
}

fn render(element: Element, container: &Node) {
    let instance: Instance = element.into();
    instance.reconcile(container);

    /*
    let rootInstance = null;

    function render(element, container) {
      const prevInstance = rootInstance;
      const nextInstance = reconcile(container, prevInstance, element);
      rootInstance = nextInstance;
    }

    */
}

#[derive(Default)]
pub struct Element {
    ty: String,
    props: Properties,
    inner: Inner<Element, String>,
}

enum Inner<SELF, TEXT> {
    Children(Vec<SELF>),
    Text(TEXT),
}

impl<S, T> Default for Inner<S, T> {
    fn default() -> Self {
        Inner::Children(vec![])
    }
}

#[derive(Default)]
struct Properties {
    attrs: BTreeMap<String, String>,
    // TODO(anp) add listeners
}

use web_sys::Element as DomElement;

impl Properties {
    fn set(&self, elem: &DomElement) {
        for (key, value) in &self.attrs {
            elem.set_attribute(key, value).unwrap();
        }
    }

    fn remove(&self, elem: &DomElement) {
        for key in self.attrs.keys() {
            elem.remove_attribute(key).unwrap();
        }
    }
}

pub struct Instance {
    ty: String,
    props: Properties,

    dom: web_sys::Element,
    inner: Inner<Instance, web_sys::Text>,
}

impl Instance {
    fn reconcile(&self, parent_dom: &Node) {
        // function reconcile(parentDom, instance, element) {
        //   if (instance == null) {
        //     // Create instance
        //     const newInstance = instantiate(element);
        //     parentDom.appendChild(newInstance.dom);
        //     return newInstance;
        //   } else if (element == null) {
        //     // Remove instance
        //     parentDom.removeChild(instance.dom);
        //     return null;
        //   } else if (instance.element.type === element.type) {
        //     // Update instance
        //     updateDomProperties(instance.dom, instance.element.props, element.props);
        //     instance.childInstances = reconcileChildren(instance, element);
        //     instance.element = element;
        //     return instance;
        //   } else {
        //     // Replace instance
        //     const newInstance = instantiate(element);
        //     parentDom.replaceChild(newInstance.dom, instance.dom);
        //     return newInstance;
        //   }
        // }

        // function reconcileChildren(instance, element) {
        //   const dom = instance.dom;
        //   const childInstances = instance.childInstances;
        //   const nextChildElements = element.props.children || [];
        //   const newChildInstances = [];
        //   const count = Math.max(childInstances.length, nextChildElements.length);
        //   for (let i = 0; i < count; i++) {
        //     const childInstance = childInstances[i];
        //     const childElement = nextChildElements[i];
        //     const newChildInstance = reconcile(dom, childInstance, childElement);
        //     newChildInstances.push(newChildInstance);
        //   }
        //   return newChildInstances.filter(instance => instance != null);
        // }
    }
}

impl From<Element> for Instance {
    fn from(elem: Element) -> Self {
        let Element { ty, props, inner } = elem;
        let document: Document = web_sys::window().unwrap().document().unwrap();
        let dom: web_sys::Element = document.create_element(&ty).unwrap().into();
        let dom_node: &Node = dom.as_ref();

        props.set(&dom);

        let inner = match inner {
            Inner::Children(children) => Inner::Children(
                children
                    .into_iter()
                    .map(|child| {
                        let instance: Instance = child.into();
                        dom_node.append_child(instance.dom.as_ref()).unwrap();
                        instance
                    })
                    .collect::<Vec<Instance>>(),
            ),
            Inner::Text(text) => {
                let text: web_sys::Text = document.create_text_node(&text);
                let text_node: &Node = text.as_ref();
                let dom_node: &Node = dom.as_ref();
                dom_node.append_child(&text_node).unwrap();

                Inner::Text(text)
            }
        };

        Instance {
            ty,
            props,
            dom,
            inner,
        }
    }
}
