use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
  String(String),
  Object(HashMap<String, Value>),
  Array(Vec<Value>),
  Integer(i64),
  Double(f64),
  Boolean(bool),
  Function(),
  None,
}
