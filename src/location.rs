use std::default;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Location {
  file: String,
  offset: u64,
  line: u64,
  column: u64,
}

impl Default for Location {
  fn default() -> Self {
    Self {
      file: Default::default(),
      offset: 1,
      line: 1,
      column: 1,
    }
  }
}

impl Location {
  pub fn new<S: AsRef<str>>(name: S, offset: u64, line: u64, column: u64) -> Location {
    Location {
      file: String::from(name.as_ref()),
      offset,
      line,
      column,
    }
  }

  pub fn file(&self) -> &String {
    &self.file
  }

  pub fn file_mut(&mut self) -> &mut String {
    &mut self.file
  }

  pub fn offset(&self) -> &u64 {
    &self.offset
  }

  pub fn offset_mut(&mut self) -> &mut u64 {
    &mut self.offset
  }

  pub fn line(&self) -> &u64 {
    &self.line
  }

  pub fn line_mut(&mut self) -> &mut u64 {
    &mut self.line
  }

  pub fn column(&self) -> &u64 {
    &self.column
  }

  pub fn column_mut(&mut self) -> &mut u64 {
    &mut self.column
  }
}
