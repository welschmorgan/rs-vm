use crate::location::Location;

#[derive(Debug)]
pub enum Error {
  IO(std::io::Error),
  Syntax(String, Location),
  Unknown(String)
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      Error::IO(e) => format!("I/O: {}", e.to_string()),
      Error::Syntax(s, loc) => format!("Syntax: {} at {}:{}", s, loc.file(), loc.line()),
      Error::Unknown(msg) => msg.clone(),
    })
  }
}
