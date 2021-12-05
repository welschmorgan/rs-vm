use enum_iterator::IntoEnumIterator;

#[derive(IntoEnumIterator, Copy, Clone, PartialEq, Debug)]
pub enum Symbol {
  LParent,
  RParent,
  LBrace,
  RBrace,
  LBracket,
  RBracket,
  Comma,
  SemiColon,
  Tab,
  Space,
  NewLine,
}

impl Symbol {
  pub fn repr(&self) -> char {
    match *self {
      Self::LParent => '(',
      Self::RParent => ')',
      Self::LBrace => '{',
      Self::RBrace => '}',
      Self::LBracket => '[',
      Self::RBracket => ']',
      Self::Comma => ',',
      Self::SemiColon => ';',
      Self::Tab => '\t',
      Self::Space => ' ',
      Self::NewLine => '\n',
    }
  }

  pub fn parse(ch: char) -> Option<Symbol> {
    for sym in Self::into_enum_iter() {
      if sym.repr() == ch {
        return Some(sym);
      }
    }
    None
  }
}
