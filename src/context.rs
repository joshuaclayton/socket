use serde_json::Value;
use std::slice::Iter;

pub struct Context {
    payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn interpret(&self, value: &[Selector]) -> String {
        self.at(value).map(value_to_string).unwrap_or("".into())
    }

    pub fn at(&self, value: &[Selector]) -> Option<&Value> {
        handle(&self.payload, value.iter())
    }

    pub fn extend(&self, key: &str, value: &Value) -> Self {
        match &self.payload {
            Value::Object(map) => {
                let mut new_map = map.clone();
                new_map.insert(key.into(), value.clone());
                Context {
                    payload: Value::Object(new_map),
                }
            }
            payload => Context {
                payload: payload.clone(),
            },
        }
    }
}

fn handle<'a>(payload: &'a Value, mut value: Iter<Selector>) -> Option<&'a Value> {
    match value.next() {
        None => Some(payload),
        Some(Selector::Key(key)) => match payload {
            Value::Object(map) => map
                .get(*key)
                .and_then(|value_for_key| handle(value_for_key, value)),
            _ => None,
        },

        Some(Selector::Index(index)) => match payload {
            Value::Array(records) => records.get(*index),
            _ => None,
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
