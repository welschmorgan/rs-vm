use crate::location::Location;

#[derive(Debug)]
pub enum Error {
  IO(std::io::Error),
  Syntax(String, Location),
  Unknown(String, Option<Location>),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Error::IO(e) => format!("I/O: {}", e.to_string()),
        Error::Syntax(s, loc) => format!("Syntax: {} at {}:{}", s, loc.file(), loc.line()),
        Error::Unknown(msg, loc) => {
          format!("Unknown: {}{}", msg.clone(), match loc {
              Some(l) => format!(" at {}:{}", l.file(), l.line()),
              None => format!("{}", ""),
          })
        }
      }
    )
  }
}
