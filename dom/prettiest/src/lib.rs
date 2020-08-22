use js_sys::{
    Array, Date, Error, Function, JsString, Map, Object, Promise, Reflect, RegExp, Set, Symbol,
};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use wasm_bindgen::{convert::IntoWasmAbi, JsCast, JsValue};

// TODO impl Display
// TODO impl serialize
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)] // TODO we can do Debug better!
pub enum Pretty {
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

    Object { name: String, contents: BTreeMap<String, Pretty> },
    Array(Vec<Pretty>),
    Map(BTreeMap<Pretty, Pretty>),
    Set(BTreeSet<Pretty>),
}

impl<T> From<T> for Pretty
where
    T: AsRef<JsValue>,
{
    fn from(val: T) -> Self {
        let mut collector = Collector::default();
        collector.collect(val.as_ref())
    }
}

#[derive(Default)]
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

    fn collect(&mut self, val: &JsValue) -> Pretty {
        if val.is_null() {
            Pretty::Null
        } else if val.is_undefined() {
            Pretty::Undefined
        } else if self.have_seen(val) {
            Pretty::Cycle
        } else if let Some(a) = val.dyn_ref::<Array>() {
            let mut children = vec![];
            for val in a.iter() {
                children.push(self.collect(&val));
            }

            Pretty::Array(children)
        } else if let Some(s) = val.dyn_ref::<Set>() {
            let mut contents = BTreeSet::new();
            let entries = s.entries();
            while let Ok(next) = entries.next() {
                if next.done() {
                    break;
                }
                contents.insert(self.collect(&next.value()));
            }

            Pretty::Set(contents)
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

            Pretty::Map(contents)
        } else if let Some(d) = val.dyn_ref::<Date>() {
            Pretty::Date(d.to_iso_string().as_string().unwrap())
        } else if let Some(e) = val.dyn_ref::<Error>() {
            Pretty::Error(e.to_string().as_string().unwrap())
        } else if let Some(r) = val.dyn_ref::<RegExp>() {
            Pretty::Regex(r.to_string().as_string().unwrap())
        } else if let Some(s) = val.dyn_ref::<Symbol>() {
            Pretty::Symbol(s.to_string().as_string().unwrap())
        } else if val.dyn_ref::<Function>().is_some() {
            Pretty::Function
        } else if val.dyn_ref::<Promise>().is_some() {
            Pretty::Promise
        } else if let Some(s) = val.dyn_ref::<JsString>() {
            Pretty::Str(s.as_string().unwrap())
        } else if let Some(n) = val.as_f64() {
            Pretty::Number(OrderedFloat(n))
        } else if let Some(b) = val.as_bool() {
            Pretty::Boolean(b)
        } else if let Some(obj) = val.dyn_ref::<Object>() {
            let mut contents = BTreeMap::new();
            let name = obj.constructor().name().as_string().unwrap();
            let mut proto = obj.clone();

            while !proto.is_falsy() {
                for raw_key in Object::get_own_property_names(&proto).iter() {
                    let key = raw_key.as_string().expect("object keys are always strings");
                    if contents.contains_key(&key) {
                        continue;
                    }
                    if let Ok(value) = Reflect::get(&obj, &raw_key) {
                        let value = self.collect(&value);
                        contents.insert(key, value);
                    }
                }
                proto = Object::get_prototype_of(proto.as_ref());
            }

            Pretty::Object { name, contents }
        } else {
            Pretty::Unknown(format!("{:?}", val))
        }
    }
}
