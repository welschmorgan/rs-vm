use std::{cell::RefCell, rc::Rc};

use crate::location::Location;

use super::Value;

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
  None,
}

impl Default for NodeKind {
  fn default() -> Self {
    NodeKind::Global
  }
}

pub type NodePtr = Rc<RefCell<Node>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
  parent: Option<NodePtr>,
  kind: NodeKind,
  name: Option<String>,
  location: Location,
  children: Vec<NodePtr>,
  value: Option<Value>,
}

impl Default for Node {
  fn default() -> Self {
    Self::new(NodeKind::default(), Location::default())
  }
}

impl Node {
  pub fn new(kind: NodeKind, loc: Location) -> Node {
    Node {
      parent: None,
      kind,
      name: None,
      location: loc,
      children: vec![],
      value: None,
    }
  }

  pub fn create_child(&mut self, scope: NodeKind, loc: Location) -> &mut NodePtr {
    self.add_child(NodePtr::new(RefCell::new(Node::new(scope, loc))))
  }

  pub fn add_child(&mut self, child: NodePtr) -> &mut NodePtr {
    self.children.push(child);
    let idx = self.children.len() - 1;
    self.children.get_mut(idx).unwrap()
  }

  pub fn get_child(&mut self, idx: usize) -> Option<&NodePtr> {
    self.children.get(idx)
  }

  pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut NodePtr> {
    self.children.get_mut(idx)
  }

  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  pub fn name_mut(&mut self) -> &mut Option<String> {
    &mut self.name
  }

  pub fn kind(&self) -> &NodeKind {
    &self.kind
  }

  pub fn kind_mut(&mut self) -> &mut NodeKind {
    &mut self.kind
  }

  pub fn parent_kind(&self) -> Option<NodeKind> {
    if let Some(p) = self.parent.clone() {
      let k = *p.borrow().kind();
      return Some(k);
    }
    None
  }

  pub fn children(&self) -> &Vec<NodePtr> {
    &self.children
  }

  pub fn children_mut(&mut self) -> &mut Vec<NodePtr> {
    &mut self.children
  }

  pub fn child_by_kind(&self, k: NodeKind) -> Option<NodePtr> {
    self
      .children
      .iter()
      .find(|child| *child.borrow().kind() == k)
      .map(|child| child.clone())
  }

  pub fn child_by_name<S: AsRef<str>>(&self, n: S) -> Option<NodePtr> {
    self
      .children
      .iter()
      .find(|child| *child.borrow().name() == Some(n.as_ref().to_string()))
      .map(|child| child.clone())
  }

  pub fn parent(&self) -> &Option<NodePtr> {
    &self.parent
  }

  pub fn parent_mut(&mut self) -> &mut Option<NodePtr> {
    &mut self.parent
  }

  pub fn location(&self) -> &Location {
    &self.location
  }

  pub fn location_mut(&mut self) -> &mut Location {
    &mut self.location
  }

  pub fn value(&self) -> &Option<Value> {
    &self.value
  }

  pub fn value_mut(&mut self) -> &mut Option<Value> {
    &mut self.value
  }
}
