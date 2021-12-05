use std::collections::HashMap;
use std::path::Path;

use crate::error::Error;
use crate::parser::{NodeKind, NodePtr, Parser, Value, AST};
use crate::result::Result;
use crate::script::{Script, ScriptState};

pub const BANNER: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub type NativeFn = dyn Fn(Vec<Value>) -> Result<Value>;

pub struct Vm {
  version: String,
  scripts: Vec<Script>,
  asts: Vec<AST>,
  native_funcs: HashMap<String, Box<NativeFn>>,
}

impl Default for Vm {
  fn default() -> Self {
    let mut ret = Self {
      version: String::from(VERSION),
      scripts: vec![],
      asts: vec![],
      native_funcs: HashMap::new(),
    };
    ret.add_native_func("println", |v| Vm::native_println(v)).unwrap();
    ret.add_native_func("print", |v| Vm::native_println(v)).unwrap();
    ret
  }
}

impl Vm {
  pub fn add_native_func<S: AsRef<str>, F: 'static + Fn(Vec<Value>) -> Result<Value>>(&mut self, k: S, f: F) -> Result<()> {
    if self.native_funcs.contains_key(k.as_ref().into()) {
      return Err(Error::Unknown(format!("native function '{}' already registered", k.as_ref()), None));
    }
    self.native_funcs.insert(k.as_ref().into(), Box::new(f));
    Ok(())
  }
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
    self
      .scripts
      .iter_mut()
      .find(|scr| scr.name() == name.as_ref())
  }

  pub fn reset(&mut self) {
    self.scripts.clear()
  }

  pub fn reachable_nodes(&self, from: NodePtr) -> Vec<NodePtr> {
    let mut ret: Vec<NodePtr> = from.borrow().ancestors();
    for ast in &self.asts {
      for root_node in ast.root().borrow().children() {
        ret.push(root_node.clone());
      }
    }
    ret
  }

  pub fn load<S: AsRef<str>, P: AsRef<Path>>(
    &mut self,
    path: P,
    name: Option<S>,
  ) -> Result<&mut Script> {
    self.scripts.push(Script::import(path, name)?);
    let n = self.scripts.len() - 1;
    *self.scripts.get_mut(n).unwrap().state_mut() = ScriptState::LOADED;
    Ok(self.scripts.get_mut(n).unwrap())
  }

  fn execute_function_call(&self, node: NodePtr) -> Result<Value> {
    let scope = self.reachable_nodes(node.clone());
    let func = scope.iter().find(|n| {
      *n.borrow().kind() == NodeKind::Function && n.borrow().name() == node.borrow().name()
    });
    match func {
      Some(f) => {}
      None => {
        // check native funcs
        if node.borrow().name().is_some() {
          let native_func = self
            .native_funcs
            .get(node.borrow().name().as_ref().unwrap());
          if (native_func.is_some()) {
            // transform FunctionParam nodes into list of values
            let args = node
              .borrow()
              .children_by_kind(NodeKind::FunctionParam)
              .iter()
              .map(|n| {
                n.borrow()
                  .value()
                  .clone()
                  .or_else(|| Some(Value::None))
                  .unwrap()
              })
              .collect();
            return native_func.unwrap()(args);
          }
        }
        return Err(Error::Unknown(
          format!(
            "Unknown function {}",
            match node.borrow().name() {
              Some(n) => format!("'{}'", n),
              None => "<unnamed>".into(),
            }
          ),
          Some(node.borrow().location().clone()),
        ));
      }
    }
    Ok(Value::None)
  }

  fn execute_node(&self, node: NodePtr) -> Result<()> {
    println!("Execute node: {}", node.borrow());
    match node.borrow().kind() {
      NodeKind::Call => {
        self.execute_function_call(node.clone())?;
      }
      _ => {}
    }
    for child in node.borrow().children().iter() {
      self.execute_node(child.clone())?;
    }
    Ok(())
  }

  pub fn run(&mut self) -> Result<()> {
    let mut p = Parser::default();
    for mut script in self.scripts.iter_mut() {
      if *script.state() == ScriptState::INITIAL {
        println!("Load Script: {}", script.name());
        script.load()?;
      }
      if *script.state() == ScriptState::LOADED {
        println!("Parse Script: {}", script.name());
        self.asts.push(p.parse(&mut script)?);
      }
    }

    for ast in self.asts.iter() {
      println!("Execute AST: {}", ast.root().borrow().location().file());
      self.execute_node(ast.root().clone())?;
    }
    Ok(())
  }

  fn native_println(args: Vec<Value>) -> Result<Value> {
    let ret = Self::native_print(args);
    println!();
    ret
  }

  fn native_print(args: Vec<Value>) -> Result<Value> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    for a in args {
      write!(&mut stdout, "{}", a).map_err(|e| Error::IO(e))?;
    }
    Ok(Value::None)
  }
}
