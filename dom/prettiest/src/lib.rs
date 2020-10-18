//! Pretty printing for Javascript values from [wasm-bindgen](https://docs.rs/wasm-bindgen).

#![forbid(unsafe_code)]

use js_sys::{
    Array, Date, Error, Function, JsString, Map, Object, Promise, Reflect, RegExp, Set, Symbol,
    WeakSet,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, Window};

pub trait Pretty {
    fn pretty(&self) -> Prettified;
}

impl<T> Pretty for T
where
    T: AsRef<JsValue>,
{
    fn pretty(&self) -> Prettified {
        Prettified {
            value: self.as_ref().to_owned(),
            seen: WeakSet::new(),
            skip: Default::default(),
        }
    }
}

/// A pretty-printable value from Javascript.
pub struct Prettified {
    /// The current value we're visiting.
    value: JsValue,
    /// We just use a JS array here to avoid relying on wasm-bindgen's unstable
    /// ABI.
    seen: WeakSet,
    /// Properties we don't want serialized.
    skip: Rc<HashSet<String>>,
}

impl Prettified {
    /// Skip printing the property with `name` if it exists on any object
    /// visited (transitively).
    pub fn skip_property(&mut self, name: &str) -> &mut Self {
        let mut with_name = HashSet::to_owned(&self.skip);
        with_name.insert(name.to_owned());
        self.skip = Rc::new(with_name);
        self
    }

    fn child(&self, v: &JsValue) -> Self {
        Self { seen: self.seen.clone(), skip: self.skip.clone(), value: v.as_ref().clone() }
    }

    // TODO get a serde_json::Value from this too
}

impl Debug for Prettified {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        // detect and break cycles before trying to figure out Object subclass
        // keeps a single path here rather than separately in each branch below
        let mut _reset = None;
        if let Some(obj) = self.value.dyn_ref::<Object>() {
            if self.seen.has(obj) {
                return write!(f, "[Cycle]");
            }

            self.seen.add(obj);
            _reset = Some(scopeguard::guard(obj.to_owned(), |obj| {
                self.seen.delete(&obj);
            }));
        }

        if self.value.is_null() {
            write!(f, "null")
        } else if self.value.is_undefined() {
            write!(f, "undefined")
        } else if self.value.dyn_ref::<Function>().is_some() {
            JsFunction.fmt(f)
        } else if self.value.dyn_ref::<Promise>().is_some() {
            write!(f, "[Promise]")
        } else if self.value.dyn_ref::<Document>().is_some() {
            write!(f, "[Document]")
        } else if self.value.dyn_ref::<Window>().is_some() {
            write!(f, "[Window]")
        } else if let Some(s) = self.value.dyn_ref::<JsString>() {
            write!(f, "{:?}", s.as_string().unwrap())
        } else if let Some(n) = self.value.as_f64() {
            write!(f, "{}", n)
        } else if let Some(b) = self.value.as_bool() {
            write!(f, "{:?}", b)
        } else if let Some(d) = self.value.dyn_ref::<Date>() {
            write!(f, "{}", d.to_iso_string().as_string().unwrap())
        } else if let Some(d) = self.value.dyn_ref::<Element>() {
            let name = d.tag_name().to_ascii_lowercase();
            let (mut class, mut id) = (d.class_name(), d.id());
            if !class.is_empty() {
                class.insert_str(0, " .");
            }
            if !id.is_empty() {
                id.insert_str(0, " #");
            }
            write!(f, "<{}{}{}/>", name, id, class)
        } else if let Some(e) = self.value.dyn_ref::<Error>() {
            write!(f, "Error: {}", e.to_string().as_string().unwrap())
        } else if let Some(r) = self.value.dyn_ref::<RegExp>() {
            write!(f, "/{}/", r.to_string().as_string().unwrap())
        } else if let Some(s) = self.value.dyn_ref::<Symbol>() {
            write!(f, "{}", s.to_string().as_string().unwrap())
        } else if let Some(a) = self.value.dyn_ref::<Array>() {
            let mut f = f.debug_list();
            for val in a.iter() {
                f.entry(&self.child(&val));
            }
            f.finish()
        } else if let Some(s) = self.value.dyn_ref::<Set>() {
            let mut f = f.debug_set();
            let entries = s.entries();
            while let Ok(next) = entries.next() {
                if next.done() {
                    break;
                }
                f.entry(&self.child(&next.value()));
            }
            f.finish()
        } else if let Some(m) = self.value.dyn_ref::<Map>() {
            let mut f = f.debug_map();
            let keys = m.keys();
            while let Ok(next) = keys.next() {
                if next.done() {
                    break;
                }
                let key = next.value();
                let value = m.get(&key);

                f.entry(&self.child(&key), &self.child(&value));
            }

            f.finish()
        } else if let Some(obj) = self.value.dyn_ref::<Object>() {
            let mut proto = obj.clone();
            let mut props_seen = HashSet::new();
            let name = obj.constructor().name().as_string().unwrap();
            let mut f = f.debug_struct(&name);

            loop {
                let mut functions = BTreeSet::new();
                let mut props = BTreeMap::new();

                for raw_key in Object::get_own_property_names(&proto).iter() {
                    let key = raw_key.as_string().expect("object keys are always strings");
                    if (key.starts_with("__") && key.ends_with("__"))
                        || props_seen.contains(&key)
                        || functions.contains(&key)
                        || self.skip.contains(&key)
                    {
                        continue;
                    }

                    if let Ok(value) = Reflect::get(&obj, &raw_key) {
                        props_seen.insert(key.clone());
                        if value.is_function() {
                            functions.insert(key);
                        } else {
                            props.insert(key, self.child(&value));
                        }
                    }
                }

                for (key, value) in props {
                    f.field(&key, &value);
                }

                for key in functions {
                    f.field(&key, &JsFunction);
                }

                proto = Object::get_prototype_of(proto.as_ref());
                if proto.is_falsy() || proto.constructor().name().as_string().unwrap() == "Object" {
                    // we've reached the end of the prototype chain
                    break;
                }
            }

            f.finish()
        } else {
            write!(f, "unknown ({:?})", &self.value)
        }
    }
}

impl Display for Prettified {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:#?}", self)
    }
}

struct JsFunction;
impl Debug for JsFunction {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[Function]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::oneshot::channel;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::{Event, EventTarget};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn cycle_is_broken() {
        let with_cycles = js_sys::Function::new_no_args(
            r#"
            let root = { child: { nested: [] } };
            root.child.nested.push(root);
            return root;
        "#,
        )
        .call0(&JsValue::null())
        .unwrap();

        assert_eq!(
            with_cycles.pretty().to_string(),
            r#"Object {
    child: Object {
        nested: [
            [Cycle],
        ],
    },
}"#
        );
    }

    #[wasm_bindgen_test]
    fn repeated_siblings_are_not_cycles() {
        let with_siblings = js_sys::Function::new_no_args(
            r#"
            let root = { child: { nested: [] } };
            let repeated_child = { foo: "bar" };
            root.child.nested.push(repeated_child);
            root.child.nested.push(repeated_child);
            return root;
        "#,
        )
        .call0(&JsValue::null())
        .unwrap();

        assert_eq!(
            with_siblings.pretty().to_string(),
            r#"Object {
    child: Object {
        nested: [
            Object {
                foo: "bar",
            },
            Object {
                foo: "bar",
            },
        ],
    },
}"#
        );
    }

    #[wasm_bindgen_test]
    async fn live_keyboard_event() {
        // create an input element and bind it to the document
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let input = document.create_element("input").unwrap();
        // input.set_attribute("type", "text").unwrap();
        document.body().unwrap().append_child(input.as_ref()).unwrap();

        // create & add an event listener that will send the event back the test
        let (send, recv) = channel();
        let callback = Closure::once_into_js(move |ev: Event| {
            send.send(ev).unwrap();
        });
        let target: &EventTarget = input.as_ref();
        let event_type = "keydown";
        target.add_event_listener_with_callback(event_type, callback.dyn_ref().unwrap()).unwrap();

        // create & dispatch an event to the input element
        let sent_event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict(
            event_type,
            web_sys::KeyboardEventInit::new()
                .char_code(b'F' as u32)
                .bubbles(true)
                .cancelable(true)
                .view(Some(&window)),
        )
        .unwrap();
        let sent: &Event = sent_event.as_ref();
        assert!(target.dispatch_event(sent).unwrap());

        // wait for the event to come back
        let received_event: Event = recv.await.unwrap();
        // make sure we can print it without exploding due to nesting
        assert_eq!(
            received_event.pretty().skip_property("timeStamp").to_string(),
            r#"KeyboardEvent {
    isTrusted: false,
    DOM_KEY_LOCATION_LEFT: 1,
    DOM_KEY_LOCATION_NUMPAD: 3,
    DOM_KEY_LOCATION_RIGHT: 2,
    DOM_KEY_LOCATION_STANDARD: 0,
    altKey: false,
    charCode: 70,
    code: "",
    ctrlKey: false,
    isComposing: false,
    key: "",
    keyCode: 0,
    location: 0,
    metaKey: false,
    repeat: false,
    shiftKey: false,
    constructor: [Function],
    getModifierState: [Function],
    initKeyboardEvent: [Function],
    detail: 0,
    sourceCapabilities: null,
    view: [Window],
    which: 0,
    initUIEvent: [Function],
    AT_TARGET: 2,
    BUBBLING_PHASE: 3,
    CAPTURING_PHASE: 1,
    NONE: 0,
    bubbles: true,
    cancelBubble: false,
    cancelable: true,
    composed: false,
    currentTarget: null,
    defaultPrevented: false,
    eventPhase: 0,
    path: [
        <input/>,
        <body/>,
        <html/>,
        [Document],
        [Window],
    ],
    returnValue: true,
    srcElement: <input/>,
    target: <input/>,
    type: "keydown",
    composedPath: [Function],
    initEvent: [Function],
    preventDefault: [Function],
    stopImmediatePropagation: [Function],
    stopPropagation: [Function],
}"#,
        );
    }
}
