use crate::error::Error;
use serde_json::Value as JsonValue;
use starlark::values::{dict::Dictionary, list::List, none::NoneType, Value};

starlark_module! { globals =>
    // TODO make this work with the dotted syntax from the spec!!!
    json_decode(x: String) {
        Ok(decode_json(&x)?)
    }
}

fn decode_json(x: &str) -> Result<Value, Error> {
    Ok(json_to_starlark(serde_json::from_str(x)?))
}

fn json_to_starlark(value: JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::new(NoneType::None),
        JsonValue::Bool(b) => Value::new(b),
        JsonValue::Number(n) => Value::new(n.as_i64().expect("TODO support floats")),
        JsonValue::String(s) => Value::new(s),
        JsonValue::Array(a) => {
            let mut list = List::default();

            for value in a {
                list.push(json_to_starlark(value)).unwrap();
            }

            Value::new(list)
        }
        JsonValue::Object(o) => {
            let mut dict = Dictionary::default();

            for (key, value) in o {
                dict.insert(Value::new(key), json_to_starlark(value)).unwrap();
            }

            Value::new(dict)
        }
    }
}
