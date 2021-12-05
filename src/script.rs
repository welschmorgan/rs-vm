use crate::{error::Error, result::Result};
use std::{ffi::OsStr, fs::read_to_string, path::{Path, PathBuf}};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScriptState {
  INITIAL,
  LOADED,
  PARSED,
  RUNNING,
  FINISHED,
}

impl std::fmt::Display for ScriptState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      &Self::INITIAL => "initial",
      &Self::LOADED => "loaded",
      &Self::PARSED => "parsed",
      &Self::RUNNING => "running",
      &Self::FINISHED => "finished",
    })
  }
}

#[derive(Debug)]
pub struct Script {
  name: String,
  path: PathBuf,
  content: Option<String>,
  state: ScriptState,
}

impl std::fmt::Display for Script {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ({})", self.name, self.state)
  }
}

impl Script {
  pub fn new<S: AsRef<str>, P: AsRef<Path>>(
    path: P,
    name: Option<S>,
    content: Option<S>,
  ) -> Script {
    let stem = path
      .as_ref()
      .file_stem()
      .map(|v: &OsStr| v.to_str().unwrap())
      .map(|v| String::from(v))
      .unwrap();
    let state = match content {
      Some(_) => ScriptState::LOADED,
      None => ScriptState::INITIAL
    };
    Script {
      name: name.map_or_else(|| stem, |v| String::from(v.as_ref())),
      path: PathBuf::from(path.as_ref()),
      content: content.map(|c| c.as_ref().clone().into()),
      state
    }
  }

  pub fn import<S: AsRef<str>, P: AsRef<Path>>(path: P, name: Option<S>) -> Result<Script> {
    let mut s = Script::new(path, name, None);
    s.load()?;
    Ok(s)
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn name_mut(&mut self) -> &mut String {
    &mut self.name
  }

  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  pub fn path_mut(&mut self) -> &mut PathBuf {
    &mut self.path
  }

  pub fn state(&self) -> &ScriptState {
    &self.state
  }

  pub fn state_mut(&mut self) -> &mut ScriptState {
    &mut self.state
  }

  pub fn content(&self) -> Option<&String> {
    self.content.as_ref()
  }

  pub fn content_mut(&mut self) -> Option<&mut String> {
    self.content.as_mut()
  }

  pub fn load(&mut self) -> Result<()> {
    self.content = Some(read_to_string(&self.path).map_or_else(|e| Err(Error::IO(e)), |c| Ok(c))?);
    Ok(())
  }
}
