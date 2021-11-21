use std::time::Duration;
use std::{ffi::OsStr, fs::read_to_string, path::Path};

use crate::script::Script;
use crate::result::Result;

pub const BANNER: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Vm {
  version: String,
  scripts: Vec<Script>,
}

impl Default for Vm {
  fn default() -> Self {
    Self {
      version: String::from(VERSION),
      scripts: vec!(),
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
    Ok(self.scripts.get_mut(n).unwrap())
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
  }
}
