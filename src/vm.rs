use std::{path::Path};

use crate::parser::{AST, NodePtr, Parser};
use crate::script::{Script, ScriptState};
use crate::result::Result;

pub const BANNER: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Vm {
  version: String,
  scripts: Vec<Script>,
  asts: Vec<AST>
}

impl Default for Vm {
  fn default() -> Self {
    Self {
      version: String::from(VERSION),
      scripts: vec!(),
      asts: vec!(),
    }
  }
}

impl Vm {
  pub fn version(&self) -> &String {
    &self.version
  }

  pub fn version_mut(&mut self) -> &mut String {
    &mut self.version
  }

  pub fn scripts(&self) -> &Vec<Script> {
    &self.scripts
  }

  pub fn scripts_mut(&mut self) -> &mut Vec<Script> {
    &mut self.scripts
  }

  pub fn add_script(&mut self, s: Script) -> &mut Script {
    self.scripts.push(s);
    let idx = self.scripts.len() - 1;
    self.scripts.get_mut(idx).unwrap()
  }

  pub fn script<S: AsRef<str>>(&self, name: S) -> Option<&Script> {
    self.scripts.iter().find(|scr| scr.name() == name.as_ref())
  }

  pub fn script_mut<S: AsRef<str>>(&mut self, name: S) -> Option<&mut Script> {
    self.scripts.iter_mut().find(|scr| scr.name() == name.as_ref())
  }

  pub fn reset(&mut self) {
    self.scripts.clear()
  }
  
  pub fn load<S: AsRef<str>, P: AsRef<Path>>(&mut self, path: P, name: Option<S>) -> Result<&mut Script> {
    self.scripts.push(Script::import(path, name)?);
    let n = self.scripts.len() - 1;
    *self.scripts.get_mut(n).unwrap().state_mut() = ScriptState::LOADED;
    Ok(self.scripts.get_mut(n).unwrap())
  }

  fn execute_node(&self, node: NodePtr) -> Result<()> {
    println!("Execute node: {}", node.borrow());
    for child in node.borrow().children().iter() {
      self.execute_node(child.clone())?;
    }
    Ok(())
  }

  #[allow(unreachable_code)]
  pub fn run(&mut self) -> Result<()> {
    let mut p = Parser::default();
    for mut script in self.scripts.iter_mut() {
      if *script.state() == ScriptState::INITIAL {
        script.load()?;
      } 
      if *script.state() == ScriptState::LOADED {
        self.asts.push(p.parse(&mut script)?);
      }
    }

    for ast in self.asts.iter() {
      println!("Execute AST: {}", ast.root().borrow().location().file());
      self.execute_node(ast.root().clone())?;
    }
    Ok(())
  }
}
