use std::{cell::RefCell, rc::Rc};

use super::{Node, NodePtr};

pub struct AST(NodePtr);

impl Default for AST {
  fn default() -> Self {
    Self(Rc::new(RefCell::new(Node::default())))
  }
}

impl AST {
  pub fn new(node: NodePtr) -> AST {
    AST(node)
  }

  pub fn root(&self) -> &NodePtr {
    &self.0
  }

  pub fn root_mut(&mut self) -> &mut NodePtr {
    &mut self.0
  }

  pub fn walk<F: Fn(&NodePtr)>(&self, f: F) {
    for child in self.0.borrow().children() {
      f(child);
    }
  }

  pub fn walk_mut<F: FnMut(&mut NodePtr)>(&self, mut f: F) {
    for child in self.0.borrow_mut().children_mut() {
      f(child);
    }
  }
}
