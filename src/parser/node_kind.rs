#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum NodeKind {
  Global,
  Function,
  FunctionParams,
  FunctionParam,
  FunctionImpl,
  Class,
  Enum,
  Method,

  Assignment,

  Add,
  Subtract,
  Multiply,
  Divide,

  Call,
  Litteral,
  ObjectLitteral,

  None,
}

impl Default for NodeKind {
  fn default() -> Self {
    NodeKind::Global
  }
}
