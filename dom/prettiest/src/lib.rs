//! Pretty printing for Javascript values from [wasm-bindgen](https://docs.rs/wasm-bindgen).

#![forbid(unsafe_code)]

use js_sys::{
    Array, Date, Error, Function, JsString, Map, Object, Promise, Reflect, RegExp, Set, Symbol,
    WeakSet,
};
use std::{
    collections::HashSet,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};

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
    pub fn skip_property(&mut self, name: &str) {
        let mut with_name = HashSet::to_owned(&self.skip);
        with_name.insert(name.to_owned());
        self.skip = Rc::new(with_name);
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
        } else if let Some(s) = self.value.dyn_ref::<JsString>() {
            write!(f, "{:?}", s.as_string().unwrap())
        } else if let Some(n) = self.value.as_f64() {
            write!(f, "{}", n)
        } else if let Some(b) = self.value.as_bool() {
            write!(f, "{:?}", b)
        } else if let Some(d) = self.value.dyn_ref::<Date>() {
            write!(f, "{}", d.to_iso_string().as_string().unwrap())
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
            let mut functions = Vec::new();
            let mut props_seen = HashSet::new();
            let name = obj.constructor().name().as_string().unwrap();
            let mut f = f.debug_struct(&name);

            loop {
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
                            functions.push(key);
                        } else {
                            f.field(&key, &self.child(&value));
                        }
                    }
                }
                proto = Object::get_prototype_of(proto.as_ref());
                if proto.is_falsy() || proto.constructor().name().as_string().unwrap() == "Object" {
                    // we've reached the end of the prototype chain
                    break;
                }
            }

            for key in functions {
                f.field(&key, &JsFunction);
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
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

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
    fn keyboard_event() {
        let event = web_sys::KeyboardEvent::new("keydown").unwrap();
        let mut pretty = event.pretty();
        // this is never going to stay still for us without browser support
        pretty.skip_property("timeStamp");

        assert_eq!(
            pretty.to_string(),
            r#"KeyboardEvent {
    isTrusted: false,
    DOM_KEY_LOCATION_STANDARD: 0,
    DOM_KEY_LOCATION_LEFT: 1,
    DOM_KEY_LOCATION_RIGHT: 2,
    DOM_KEY_LOCATION_NUMPAD: 3,
    key: "",
    code: "",
    location: 0,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    metaKey: false,
    repeat: false,
    isComposing: false,
    charCode: 0,
    keyCode: 0,
    view: null,
    detail: 0,
    sourceCapabilities: null,
    which: 0,
    NONE: 0,
    CAPTURING_PHASE: 1,
    AT_TARGET: 2,
    BUBBLING_PHASE: 3,
    type: "keydown",
    target: null,
    currentTarget: null,
    eventPhase: 0,
    bubbles: false,
    cancelable: false,
    defaultPrevented: false,
    composed: false,
    srcElement: null,
    returnValue: true,
    cancelBubble: false,
    path: [],
    getModifierState: [Function],
    initKeyboardEvent: [Function],
    constructor: [Function],
    initUIEvent: [Function],
    composedPath: [Function],
    stopPropagation: [Function],
    stopImmediatePropagation: [Function],
    preventDefault: [Function],
    initEvent: [Function],
}"#,
        );
    }
}
