use serde_json::Value as JsonValue;
use starlark::{
    collections::SmallMap,
    environment::GlobalsBuilder,
    starlark_immutable_value,
    values::{dict::Dict, list::List, Heap, TypedValue, Value},
};
use starlark_module::starlark_module;
use std::convert::TryInto;

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    const json: JsonModule = JsonModule;
}

#[derive(Debug)]
struct JsonModule;

#[starlark_module]
fn register_json_methods(globals: &mut GlobalsBuilder) {
    fn decode(_this: RefJsonModule, x: String) -> Value<'v> {
        Ok(json_to_starlark(heap, serde_json::from_str(&x)?))
    }
}

starlark_immutable_value!(JsonModule);

impl TypedValue<'_> for JsonModule {
    starlark::starlark_type!("json");
    declare_members!(register_json_methods);
}

fn json_to_starlark<'h>(heap: &'h Heap, value: JsonValue) -> Value<'h> {
    match value {
        JsonValue::Null => Value::new_none(),
        JsonValue::Bool(b) => Value::new_bool(b),
        JsonValue::Number(n) => {
            let n = n.as_i64().expect("floats not yet supported");
            let n: i32 = n.try_into().expect("only numbers within +/- 2gb are supported");
            Value::new_int(n)
        }
        JsonValue::String(s) => heap.alloc(s),
        JsonValue::Array(a) => {
            let mut list = List::default();
            for value in a {
                list.push(json_to_starlark(heap, value));
            }
            heap.alloc(list)
        }
        JsonValue::Object(o) => {
            let mut dict: SmallMap<Value, Value> = Default::default();
            for (key, value) in o {
                let key = heap.alloc(key);
                dict.insert_hashed(key.get_hashed().unwrap(), json_to_starlark(heap, value));
            }
            heap.alloc(Dict::new(dict))
        }
    }
}
