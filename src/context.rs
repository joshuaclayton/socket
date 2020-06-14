use serde_json::Value;

pub struct Context {
    payload: serde_json::Value,
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

    pub fn interpret(&self, value: &str) -> String {
        match &self.payload {
            Value::Object(map) => map.get(value).map(value_to_string).unwrap_or(String::new()),
            _ => String::from("unable to parse"),
        }
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
