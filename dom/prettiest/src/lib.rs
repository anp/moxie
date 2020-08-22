use js_sys::{
    Array, Date, Error, Function, JsString, Map, Object, Promise, Reflect, RegExp, Set, Symbol,
};
use ordered_float::OrderedFloat;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};
use wasm_bindgen::{convert::IntoWasmAbi, JsCast, JsValue};

pub trait Pretty {
    fn pretty(&self) -> Prettified;
}

impl<T> Pretty for T
where
    T: AsRef<JsValue>,
{
    fn pretty(&self) -> Prettified {
        let mut collector = Collector { seen: Default::default() };
        collector.collect(self.as_ref())
    }
}

/// A pretty-printable value from Javascript.
// TODO impl serialize
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Prettified {
    Cycle,
    Function,
    Promise,
    Null,
    Undefined,
    Unknown(String),

    Number(OrderedFloat<f64>),
    Boolean(bool),

    Str(String),
    Date(String),
    Error(String),
    Regex(String),
    Symbol(String),

    Object { name: String, contents: BTreeMap<String, Self>, functions: BTreeSet<String> },
    Array(Vec<Self>),
    Map(BTreeMap<Self, Self>),
    Set(BTreeSet<Self>),
}

impl Prettified {
    /// Remove the property with the given `name` if this is an object and it
    /// has the property.
    pub fn delete_property(&mut self, name: &str) {
        match self {
            Prettified::Object { contents, .. } => {
                contents.remove(name);
            }
            _ => (),
        }
    }

    fn is_function(&self) -> bool {
        match self {
            Prettified::Function => true,
            _ => false,
        }
    }
}

impl Debug for Prettified {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Cycle => write!(f, "[Cycle]"),
            Self::Function => write!(f, "[Function]"),
            Self::Promise => write!(f, "[Promise]"),
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Unknown(u) => write!(f, "unknown ({})", u),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{:?}", b),
            Self::Str(s) => write!(f, "{:?}", s),
            Self::Date(d) => write!(f, "{}", d),
            Self::Error(e) => write!(f, "Error: {}", e),
            Self::Regex(r) => write!(f, "/{}/", r),
            Self::Symbol(sym) => write!(f, "{}", sym),
            Self::Array(arr) => f.debug_list().entries(arr.iter()).finish(),
            Self::Set(set) => f.debug_set().entries(set.iter()).finish(),
            Self::Map(map) => f.debug_map().entries(map.iter()).finish(),
            Self::Object { name, contents, functions } => {
                let mut f = f.debug_struct(name);
                for (key, value) in contents {
                    f.field(key, value);
                }
                for key in functions {
                    f.field(key, &Prettified::Function);
                }
                f.finish()
            }
        }
    }
}

impl Display for Prettified {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:#?}", self)
    }
}

struct Collector {
    seen: HashSet<u32>,
}

impl Collector {
    fn have_seen(&mut self, val: &JsValue) -> bool {
        let raw = val.clone().into_abi();
        let already_has = self.seen.contains(&raw);
        self.seen.insert(raw);
        already_has
    }

    fn collect(&mut self, val: &JsValue) -> Prettified {
        if val.is_null() {
            Prettified::Null
        } else if val.is_undefined() {
            Prettified::Undefined
        } else if self.have_seen(val) {
            Prettified::Cycle
        } else if let Some(a) = val.dyn_ref::<Array>() {
            let mut children = vec![];
            for val in a.iter() {
                children.push(self.collect(&val));
            }

            Prettified::Array(children)
        } else if let Some(s) = val.dyn_ref::<Set>() {
            let mut contents = BTreeSet::new();
            let entries = s.entries();
            while let Ok(next) = entries.next() {
                if next.done() {
                    break;
                }
                contents.insert(self.collect(&next.value()));
            }

            Prettified::Set(contents)
        } else if let Some(m) = val.dyn_ref::<Map>() {
            let mut contents = BTreeMap::new();
            let keys = m.keys();
            while let Ok(next) = keys.next() {
                if next.done() {
                    break;
                }
                let key = next.value();
                let value = m.get(&key);

                contents.insert(self.collect(&key), self.collect(&value));
            }

            Prettified::Map(contents)
        } else if let Some(d) = val.dyn_ref::<Date>() {
            Prettified::Date(d.to_iso_string().as_string().unwrap())
        } else if let Some(e) = val.dyn_ref::<Error>() {
            Prettified::Error(e.to_string().as_string().unwrap())
        } else if let Some(r) = val.dyn_ref::<RegExp>() {
            Prettified::Regex(r.to_string().as_string().unwrap())
        } else if let Some(s) = val.dyn_ref::<Symbol>() {
            Prettified::Symbol(s.to_string().as_string().unwrap())
        } else if val.dyn_ref::<Function>().is_some() {
            Prettified::Function
        } else if val.dyn_ref::<Promise>().is_some() {
            Prettified::Promise
        } else if let Some(s) = val.dyn_ref::<JsString>() {
            Prettified::Str(s.as_string().unwrap())
        } else if let Some(n) = val.as_f64() {
            Prettified::Number(OrderedFloat(n))
        } else if let Some(b) = val.as_bool() {
            Prettified::Boolean(b)
        } else if let Some(obj) = val.dyn_ref::<Object>() {
            let mut contents = BTreeMap::new();
            let mut functions = BTreeSet::new();
            let name = obj.constructor().name().as_string().unwrap();
            let mut proto = obj.clone();

            while !proto.is_falsy() {
                for raw_key in Object::get_own_property_names(&proto).iter() {
                    let key = raw_key.as_string().expect("object keys are always strings");
                    let is_quasi_builtin = key.starts_with("__") && key.ends_with("__");

                    // we don't need to capture internals at all or anything twice
                    if is_quasi_builtin || contents.contains_key(&key) || functions.contains(&key) {
                        continue;
                    }

                    if let Ok(value) = Reflect::get(&obj, &raw_key) {
                        let value = self.collect(&value);
                        if value.is_function() {
                            functions.insert(key);
                        } else {
                            contents.insert(key, value);
                        }
                    }
                }
                proto = Object::get_prototype_of(proto.as_ref());
            }

            Prettified::Object { name, contents, functions }
        } else {
            Prettified::Unknown(format!("{:?}", val))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn keyboard_event() {
        let event = web_sys::KeyboardEvent::new("keydown").unwrap();
        let mut pretty = event.pretty();
        // this is never going to stay still for us without browser support
        pretty.delete_property("timeStamp");

        assert_eq!(
            pretty.to_string(),
            r#"KeyboardEvent {
    AT_TARGET: 2,
    BUBBLING_PHASE: 3,
    CAPTURING_PHASE: 1,
    DOM_KEY_LOCATION_LEFT: 1,
    DOM_KEY_LOCATION_NUMPAD: 3,
    DOM_KEY_LOCATION_RIGHT: 2,
    DOM_KEY_LOCATION_STANDARD: 0,
    NONE: 0,
    altKey: false,
    bubbles: false,
    cancelBubble: false,
    cancelable: false,
    charCode: 0,
    code: "",
    composed: false,
    ctrlKey: false,
    currentTarget: null,
    defaultPrevented: false,
    detail: 0,
    eventPhase: 0,
    isComposing: false,
    isTrusted: false,
    key: "",
    keyCode: 0,
    location: 0,
    metaKey: false,
    path: [],
    repeat: false,
    returnValue: true,
    shiftKey: false,
    sourceCapabilities: null,
    srcElement: null,
    target: null,
    type: "keydown",
    view: null,
    which: 0,
    composedPath: [Function],
    constructor: [Function],
    getModifierState: [Function],
    hasOwnProperty: [Function],
    initEvent: [Function],
    initKeyboardEvent: [Function],
    initUIEvent: [Function],
    isPrototypeOf: [Function],
    preventDefault: [Function],
    propertyIsEnumerable: [Function],
    stopImmediatePropagation: [Function],
    stopPropagation: [Function],
    toLocaleString: [Function],
    toString: [Function],
    valueOf: [Function],
}"#,
        );
    }
}
