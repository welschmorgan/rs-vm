use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
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

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      Self::String(s) => format!("\"{}\"", s),
      Self::Object(m) => format!("\"{:?}\"", m),
      Self::Array(v) => format!("{:?}", v),
      Self::Integer(i) => format!("{}", i),
      Self::Double(d) => format!("{}", d),
      Self::Boolean(b) => format!("{}", b),
      Self::Function() => format!("fn () {{}}", ),
      Self::None => format!("{}", "none")
    })
  }
}
