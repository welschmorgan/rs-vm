use std::cell::RefCell;
use std::rc::{Rc};

use crate::error::Error;
use crate::location::Location;
use crate::result::Result;
use crate::script::Script;

use super::{Keyword, Node, NodeKind, NodePtr, AST};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Options {
  Debug,
}

pub struct Parser {
  location: Location,
  root_scope: NodePtr,
  cur_scope: NodePtr,
  accu: String,
  quote: Option<char>,
  keywords: Vec<Keyword>,
  options: Vec<Options>,
}

impl Default for Parser {
  fn default() -> Self {
    let root_scope = Rc::new(RefCell::new(Node::default()));
    Self {
      location: Default::default(),
      root_scope: root_scope.clone(),
      cur_scope: root_scope.clone(),
      accu: Default::default(),
      quote: Default::default(),
      keywords: Default::default(),
      options: Default::default(),
    }
  }
}

impl Parser {
  pub fn new(mut opts: Vec<Options>) -> Self {
    let mut p = Self::default();
    p.options.append(&mut opts);
    p
  }

  pub fn location(&self) -> &Location {
    &self.location
  }

  pub fn location_mut(&mut self) -> &mut Location {
    &mut self.location
  }

  pub fn root_scope(&self) -> &NodePtr {
    &self.root_scope
  }

  pub fn root_scope_mut(&mut self) -> &mut NodePtr {
    &mut self.root_scope
  }

  pub fn cur_scope(&self) -> &NodePtr {
    &self.cur_scope
  }

  pub fn cur_scope_mut(&mut self) -> &mut NodePtr {
    &mut self.cur_scope
  }

  pub fn keywords(&self) -> &Vec<Keyword> {
    &self.keywords
  }

  pub fn keywords_mut(&mut self) -> &mut Vec<Keyword> {
    &mut self.keywords
  }

  pub fn options(&self) -> &Vec<Options> {
    &self.options
  }

  pub fn options_mut(&mut self) -> &mut Vec<Options> {
    &mut self.options
  }

  pub fn reset(&mut self) {
    let options = self.options.clone();
    *self = Parser::default();
    self.options = options;
  }

  pub fn has_option(&self, o: Options) -> bool {
    let found = self.options.iter().find(|it| (*it).eq(&o));
    found.is_some()
  }

  pub fn cur_scope_kind(&self) -> NodeKind {
    self.cur_scope.borrow().kind().clone()
  }

  pub fn parse(&mut self, s: &Script) -> Result<AST> {
    self.reset();
    s.content().ok_or_else(|| {
      Error::IO(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "script has no content",
      ))
    })?;
    *self.location.file_mut() = s.name().clone();
    let ch_it = s.content().unwrap().chars();
    for ch in ch_it {
      if self.has_option(Options::Debug) {
        println!("parse: {}", ch);
      }
      if self.quote != None {
        self.accu.push(ch);
      } else if ch == '\'' || ch == '"' {
        self.parse_quote(ch)?;
      } else {
        match ch {
          '(' => self.parse_lparen(ch),
          ')' => self.parse_rparen(ch),
          '{' => self.parse_lbrace(ch),
          '}' => self.parse_rbrace(ch),
          ',' => self.parse_comma(ch),
          ';' => self.parse_semicolon(ch),
          '\t' | ' ' => self.parse_space(ch),
          '\n' => self.parse_eol(ch),
          _ => self.parse_unknown(ch),
        }?;
        if ch.is_alphanumeric() {
          self.accu.push(ch);
        } else {
          self.parse_keyword()?;
        }
      }
      *self.location.offset_mut() += 1;
      *self.location.column_mut() += 1;
    }
    if self.has_option(Options::Debug) {
      self.dump(self.root_scope.clone(), 0);
    }
    Ok(AST::new(self.root_scope.clone()))
  }

  pub fn dump(&self, node: NodePtr, indent: usize) {
    print!("{}", "\t".repeat(indent));
    if *node.borrow().kind() == NodeKind::None {
      println!(
        "{} = {:?}",
        node.borrow().name().clone().unwrap_or("".into()),
        node.borrow().value()
      );
    } else {
      println!(
        "{:?}:{} {{",
        node.borrow().kind(),
        node.borrow().name().clone().unwrap_or("".into())
      );
    }
    for child in node.borrow().children() {
      self.dump(child.clone(), indent + 1);
    }
    if *node.borrow().kind() != NodeKind::None {
      println!("{}}}", "\t".repeat(indent))
    }
  }

  fn parse_keyword(&mut self) -> Result<()> {
    if self.accu.len() > 0 {
      if self.has_option(Options::Debug) {
        println!("parse kw: {:?}", self.accu);
      }
      if let Some(kw) = Keyword::parse(&self.accu) {
        match kw {
          Keyword::Function => {
            self.push_scope(NodeKind::Function)?;
          }
          Keyword::Class => {
            self.push_scope(NodeKind::Class)?;
          }
          Keyword::Enum => {
            self.push_scope(NodeKind::Enum)?;
          }
          Keyword::Private => {}
          Keyword::Protected => {}
          Keyword::Public => {}
          Keyword::Return => {}
        };
        self.keywords.push(kw);
        self.accu.clear();
      }
    }
    Ok(())
  }

  fn parse_comma(&mut self, _ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == NodeKind::FunctionParams {
      if self.accu.trim().len() == 0 {
        return Err(Error::Syntax(
          "unexpected ','".into(),
          self.location.clone(),
        ));
      }
      self.push_fn_param()?;
    } else {
      return Err(Error::Syntax(
        "unexpected ','".into(),
        self.location.clone(),
      ));
    }
    Ok(())
  }

  fn parse_lparen(&mut self, _ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == NodeKind::Function {
      if self.accu.len() > 0 {
        // register function name if given
        *self.cur_scope.borrow_mut().name_mut() = Some(self.accu.clone());
        self.accu.clear();
      }
      // check first param decl
      if self
        .cur_scope
        .borrow()
        .children()
        .iter()
        .find(|ch| *ch.borrow().kind() == NodeKind::FunctionParams)
        .is_some()
      {
        return Err(Error::Syntax(
          "unexpected '('".into(),
          self.location.clone(),
        ));
      }
      self.push_scope(NodeKind::FunctionParams)?;
    }
    Ok(())
  }

  fn parse_rparen(&mut self, _ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == NodeKind::FunctionParams {
      if self.accu.trim().len() != 0 {
        self.push_fn_param()?;
      }
    }
    self.pop_scope()?;
    Ok(())
  }

  fn parse_lbrace(&mut self, _ch: char) -> Result<()> {
    let mut kind = NodeKind::None;
    match self.cur_scope_kind() {
      NodeKind::Function => kind = NodeKind::FunctionImpl,
      _ => {}
    }
    self.push_scope(kind)?;
    Ok(())
  }

  fn parse_rbrace(&mut self, _ch: char) -> Result<()> {
    self.pop_scope()?;
    Ok(())
  }

  fn parse_semicolon(&mut self, _ch: char) -> Result<()> {
    self.keywords.clear();
    Ok(())
  }

  fn parse_quote(&mut self, ch: char) -> Result<()> {
    if self.quote == Some(ch) {
      self.quote = None;
    } else if self.quote == None {
      self.quote = Some(ch);
    }
    Ok(())
  }

  fn parse_space(&mut self, _ch: char) -> Result<()> {
    self.parse_keyword()?;
    Ok(())
  }

  fn parse_eol(&mut self, ch: char) -> Result<()> {
    if self.quote != None {
      self.accu.push(ch);
    }
    *self.location.line_mut() += 1;
    *self.location.column_mut() = 0;
    Ok(())
  }

  fn parse_unknown(&mut self, _ch: char) -> Result<()> {
    Ok(())
  }

  fn push_scope(&mut self, kind: NodeKind) -> Result<NodePtr> {
    let last_scope = self.cur_scope.clone();
    self.cur_scope = last_scope
      .borrow_mut()
      .create_child(kind, self.location.clone())
      .clone();
    *self.cur_scope.borrow_mut().parent_mut() = Some(last_scope.clone());
    println!(
      "push_scope: {:?} -> {:?}",
      last_scope.borrow().kind(),
      self.cur_scope.borrow().kind()
    );
    Ok(self.cur_scope.clone())
  }

  fn pop_scope(&mut self) -> Result<NodePtr> {
    if self.cur_scope.borrow().parent().is_none() {
      return Err(Error::Unknown("no active scope".into()));
    }
    let parent = self.cur_scope.borrow().parent().clone().unwrap();
    let last_kind = self.cur_scope.borrow().kind().clone();
    self.cur_scope = parent;
    println!(
      "pop_scope: {:?} -> {:?}",
      last_kind,
      self.cur_scope.borrow().kind()
    );
    Ok(self.cur_scope.clone())
  }

  fn push_fn_param(&mut self) -> Result<()> {
    *self
      .cur_scope
      .borrow_mut()
      .create_child(NodeKind::FunctionParam, self.location.clone())
      .borrow_mut()
      .name_mut() = Some(self.accu.clone());
    self.accu.clear();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  #[test]
  fn function_parsing_works() {
    let script = Script::new(
      PathBuf::from("virtual://test"),
      Some("test"),
      Some("function hello(p1, p2) {\n\t\"\";}"),
    );
    let mut p = Parser::new(vec![Options::Debug]);
    let ast = p.parse(&script).unwrap();
    let root = ast.root().clone();
    assert!(root.borrow().children().len() > 0);
    assert_ne!(root.borrow().children().first(), None);
    let func = root.borrow().children().first().unwrap().borrow().clone();
    assert_eq!(*func.kind(), NodeKind::Function);
    assert_eq!(*func.name(), Some("hello".into()));
    assert_eq!(func.children().len(), 2);
    let func_args = func.children().get(0);
    assert_ne!(func_args, None);
    assert_eq!(
      *func_args.unwrap().borrow().kind(),
      NodeKind::FunctionParams
    );
    // check param1
    assert_eq!(
      *func_args
        .unwrap()
        .borrow()
        .children()
        .get(0)
        .unwrap()
        .borrow()
        .kind(),
      NodeKind::FunctionParam
    );
    assert_eq!(
      *func_args
        .unwrap()
        .borrow()
        .children()
        .get(0)
        .unwrap()
        .borrow()
        .name(),
      Some("p1".into())
    );
    // check param2

    assert_eq!(
      *func_args
        .unwrap()
        .borrow()
        .children()
        .get(0)
        .unwrap()
        .borrow()
        .kind(),
      NodeKind::FunctionParam
    );
    assert_eq!(
      *func_args
        .unwrap()
        .borrow()
        .children()
        .get(0)
        .unwrap()
        .borrow()
        .name(),
      Some("p1".into())
    );
    let func_impl = func.children().get(1);
    assert_ne!(func_impl, None);
  }
}
