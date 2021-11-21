use std::fmt::Display;

use enum_iterator::IntoEnumIterator;

#[derive(IntoEnumIterator)]
pub enum Keyword {
  Function,
  Class,
  Enum,
  Return,
  Public,
  Private,
  Protected,
}

impl Display for Keyword {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match *self {
        Keyword::Function => "function",
        Keyword::Class => "class",
        Keyword::Enum => "enum",
        Keyword::Return => "return",
        Keyword::Public => "public",
        Keyword::Protected => "protected",
        Keyword::Private => "private",
      }
    )
  }
}

impl Keyword {
  pub fn parse<S: AsRef<str>>(s: S) -> Option<Keyword> {
    Keyword::into_enum_iter().find(|kw| format!("{}", kw) == s.as_ref().trim())
  }
}
