use serde_json::Value;
use std::slice::Iter;

pub struct Context {
    payload: serde_json::Value,
}

#[derive(Debug, PartialEq)]
pub enum Selector<'a> {
    Key(&'a str),
    Index(usize),
}

impl Context {
    pub fn empty() -> Self {
        let payload = serde_json::Value::Null;
        Context { payload }
    }

    pub fn load(input: &str) -> Self {
        let payload = serde_json::from_str(input).unwrap_or(serde_json::Value::Null);
        Context { payload }
    }

    pub fn interpret(&self, value: Iter<Selector>) -> String {
        handle(&self.payload, value)
    }
}

fn handle(payload: &Value, mut value: Iter<Selector>) -> String {
    match value.next() {
        None => value_to_string(payload),
        Some(Selector::Key(key)) => match payload {
            Value::Object(map) => map
                .get(*key)
                .map(|value_for_key| handle(value_for_key, value))
                .unwrap_or(String::new()),
            _ => String::from("unable to parse"),
        },

        Some(Selector::Index(index)) => match payload {
            Value::Array(records) => records
                .get(*index)
                .map(value_to_string)
                .unwrap_or("".to_string()),
            _ => String::from("unable to parse"),
        },
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Number(n) => format!("{}", n),
        Value::String(s) => s.to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}
