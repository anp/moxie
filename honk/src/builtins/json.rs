use serde_json::Value as JsonValue;
use starlark::{
    collections::SmallMap,
    environment::GlobalsBuilder,
    values::{dict::Dict, list::List, Heap, Value},
};
use std::convert::TryInto;

#[starlark_module::starlark_module]
pub fn register(globals: &mut GlobalsBuilder) {
    // TODO make this work with the dotted syntax from the spec!!!
    fn json_decode(x: String) -> Value<'v> {
        Ok(json_to_starlark(heap, serde_json::from_str(&x)?))
    }
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
