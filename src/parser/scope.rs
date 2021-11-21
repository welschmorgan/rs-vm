use std::{cell::RefCell, rc::Rc};

use super::Value;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum ScopeKind {
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

impl Default for ScopeKind {
  fn default() -> Self {
    ScopeKind::Global
  }
}

pub type ScopePtr = Rc<RefCell<Scope>>;

pub struct Scope {
  parent: Option<ScopePtr>,
  kind: ScopeKind,
  name: Option<String>,
  children: Vec<ScopePtr>,
  value: Option<Value>
}

impl Scope {
  pub fn new(kind: ScopeKind) -> Scope {
    Scope {
      parent: None,
      kind,
      name: None,
      children: vec![],
      value: None,
    }
  }

  pub fn with_name<S: AsRef<str>>(scope: ScopeKind, name: S) -> Scope {
    let mut b = Scope::new(scope);
    *b.name_mut() = Some(name.as_ref().into());
    b
  }

  pub fn create_child(&mut self, scope: ScopeKind) -> &mut ScopePtr {
    self.add_child(ScopePtr::new(RefCell::new(Scope::new(scope))))
  }

  pub fn add_child(&mut self, child: ScopePtr) -> &mut ScopePtr {
    self.children.push(child);
    let idx = self.children.len() - 1;
    self.children.get_mut(idx).unwrap()
  }

  pub fn get_child(&mut self, idx: usize) -> Option<&ScopePtr> {
    self.children.get(idx)
  }

  pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut ScopePtr> {
    self.children.get_mut(idx)
  }

  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  pub fn name_mut(&mut self) -> &mut Option<String> {
    &mut self.name
  }

  pub fn kind(&self) -> &ScopeKind {
    &self.kind
  }

  pub fn kind_mut(&mut self) -> &mut ScopeKind {
    &mut self.kind
  }

  pub fn parent_kind(&self) -> Option<ScopeKind> {
    if let Some(p) = self.parent.clone() {
      let k = *p.borrow().kind();
      return Some(k);
    }
    None
  }

  pub fn children(&self) -> &Vec<ScopePtr> {
    &self.children
  }

  pub fn children_mut(&mut self) -> &mut Vec<ScopePtr> {
    &mut self.children
  }

  pub fn parent(&self) -> &Option<ScopePtr> {
    &self.parent
  }

  pub fn parent_mut(&mut self) -> &mut Option<ScopePtr> {
    &mut self.parent
  }

  pub fn value(&self) -> &Option<Value> {
    &self.value
  }

  pub fn value_mut(&mut self) -> &mut Option<Value> {
    &mut self.value
  }
}