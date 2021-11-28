use super::Value;

#[allow(dead_code)]
pub struct Variable {
  name: String,
  value: Value,
}

impl Variable {
  pub fn new(name: String, value: Value) -> Self {
    Self { name, value }
  }
}
