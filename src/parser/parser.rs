use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::location::Location;
use crate::result::Result;
use crate::script::{Script, ScriptState};

use super::{AST, Keyword, Node, NodeKind, NodePtr, ParserOption, Symbol, Value};

pub struct Parser {
  location: Location,
  root_scope: NodePtr,
  cur_scope: NodePtr,
  accu: String,
  quote: Option<char>,
  keywords: Vec<Keyword>,
  options: Vec<ParserOption>,
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
      options: ParserOption::from_env(),
    }
  }
}

impl Parser {
  pub fn new(mut opts: Vec<ParserOption>) -> Self {
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

  pub fn options(&self) -> &Vec<ParserOption> {
    &self.options
  }

  pub fn options_mut(&mut self) -> &mut Vec<ParserOption> {
    &mut self.options
  }

  pub fn reset(&mut self) {
    let options = self.options.clone();
    *self = Parser::default();
    self.options = options;
  }

  pub fn has_option(&self, o: ParserOption) -> bool {
    let found = self.options.iter().find(|it| (*it).eq(&o));
    found.is_some()
  }

  pub fn cur_scope_kind(&self) -> NodeKind {
    self.cur_scope.borrow().kind().clone()
  }

  pub fn parse(&mut self, s: &mut Script) -> Result<AST> {
    self.reset();
    s.content().ok_or_else(|| {
      Error::IO(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "script has no content",
      ))
    })?;
    *self.location.file_mut() = s.name().clone();
    *self.root_scope.borrow_mut().location_mut() = self.location.clone();
    let ch_it = s.content().unwrap().chars();
    #[allow(unused_assignments)]
    let mut is_symbol = false;
    for ch in ch_it {
      if self.has_option(ParserOption::Debug) {
        println!("parse: {}", ch);
      }
      if self.quote.is_some() && ch != Symbol::DoubleQuote.repr() && ch != Symbol::SingleQuote.repr() {
        self.accu.push(ch);
      } else {
        is_symbol = false;
        if let Some(sym) = Symbol::parse(ch) {
          is_symbol = true;
          match sym {
            Symbol::LParent => self.parse_lparen(ch),
            Symbol::RParent => self.parse_rparen(ch),
            Symbol::LBrace => self.parse_lbrace(ch),
            Symbol::RBrace => self.parse_rbrace(ch),
            Symbol::LBracket => self.parse_lbracket(ch),
            Symbol::RBracket => self.parse_rbracket(ch),
            Symbol::DoubleQuote | Symbol::SingleQuote => self.parse_quote(ch),
            Symbol::Comma => self.parse_comma(ch),
            Symbol::SemiColon => self.parse_semicolon(ch),
            Symbol::Tab | Symbol::Space => self.parse_space(ch),
            Symbol::NewLine => self.parse_eol(ch),
          }?;
        }
        if !is_symbol {
          self.accu.push(ch);
        }
        self.parse_keyword();
      }
      *self.location.offset_mut() += 1;
      *self.location.column_mut() += 1;
    }
    *s.state_mut() = ScriptState::PARSED;
    if self.has_option(ParserOption::Debug) {
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

  fn parse_keyword(&mut self) -> Option<Keyword> {
    if self.accu.len() > 0 {
      if self.has_option(ParserOption::Debug) {
        println!("parse kw: {:?}", self.accu);
      }
      if let Some(kw) = Keyword::parse(&self.accu) {
        match kw {
          Keyword::Function => {
            self.push_scope(NodeKind::Function);
          }
          Keyword::Class => {
            self.push_scope(NodeKind::Class);
          }
          Keyword::Enum => {
            self.push_scope(NodeKind::Enum);
          }
          Keyword::Private => {}
          Keyword::Protected => {}
          Keyword::Public => {}
          Keyword::Return => {}
          Keyword::Throw => {}
          Keyword::Let => {}
          Keyword::Const => {}
        };
        self.keywords.push(kw);
        self.accu.clear();
        return Some(kw);
      }
    }
    None
  }


  fn parse_lparen(&mut self, _ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == NodeKind::Function {
      // parse function declaration
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
      self.push_scope(NodeKind::FunctionParams);
    } else {
      // parse function call
      self.accu = self.accu.trim().to_string();
      if self.accu.is_empty() {
        return Err(Error::Syntax(
          "unexpected '('".into(),
          self.location.clone()
        ));
      }
      let func = self.push_scope(NodeKind::Call);
      *func.borrow_mut().name_mut() = Some(self.accu.clone());
      self.accu.clear();
    }
    Ok(())
  }

  fn parse_rparen(&mut self, _ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == NodeKind::FunctionParams {
      if self.accu.trim().len() != 0 {
        self.push_fn_param()?;
      }
    } else if self.cur_scope_kind() == NodeKind::Call {
      self.accu = self.accu.trim().to_string();
      if !self.accu.is_empty() {
        let param = self.cur_scope.borrow_mut().create_child(NodeKind::FunctionParam, self.location.clone()).clone();
        *param.borrow_mut().value_mut() = Some(Value::String(self.accu.clone()));
        self.accu.clear();
      }
    }
    self.pop_scope()?;
    Ok(())
  }

  fn parse_lbrace(&mut self, _ch: char) -> Result<()> {
    let mut kind = NodeKind::None;
    match self.cur_scope_kind() {
      NodeKind::Function => {
        kind = NodeKind::FunctionImpl;
      }
      _ => {}
    }
    self.keywords.clear();
    self.push_scope(kind);
    Ok(())
  }

  fn parse_rbrace(&mut self, _ch: char) -> Result<()> {
    if !self.accu.is_empty() {
      self.parse_expr()?;
    }
    if self.cur_scope_kind() == NodeKind::FunctionImpl {
      // pop 2 scopes: FunctionImpl and Function
      self.pop_scope()?;
    }
    self.pop_scope()?;
    Ok(())
  }

  fn accu_empty(&self) -> bool {
    self.accu.trim().len() == 0
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
    } else if self.cur_scope_kind() == NodeKind::Call {
      let param = self.cur_scope.borrow_mut().create_child(NodeKind::FunctionParam, self.location.clone()).clone();
      *param.borrow_mut().value_mut() = Some(Value::String(self.accu.clone()));
      self.accu.clear();
    } else {
      return Err(Error::Syntax(
        "unexpected ','".into(),
        self.location.clone(),
      ));
    }
    Ok(())
  }

  fn parse_semicolon(&mut self, _ch: char) -> Result<()> {
    if !self.accu_empty() {
      self.parse_expr()?;
    }
    if self.cur_scope_kind() == NodeKind::Function && self.cur_scope().borrow().child_by_kind(NodeKind::FunctionImpl).is_none() {
      *self.cur_scope.borrow_mut().kind_mut() = NodeKind::Call;
    }
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
    // if self.parse_keyword().is_none() {
      // self.accu.push(_ch);
    // }
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

  fn push_scope(&mut self, kind: NodeKind) -> NodePtr {
    let last_scope = self.cur_scope.clone();
    self.cur_scope = last_scope
      .borrow_mut()
      .create_child(kind, self.location.clone())
      .clone();
    *self.cur_scope.borrow_mut().parent_mut() = Some(last_scope.clone());
    if self.has_option(ParserOption::Debug) {
      println!(
        "push_scope: {:?} -> {:?}",
        last_scope.borrow().kind(),
        self.cur_scope.borrow().kind()
      );
    }
    self.cur_scope.clone()
  }

  fn pop_scope(&mut self) -> Result<NodePtr> {
    if self.cur_scope.borrow().parent().is_none() {
      return Err(Error::Unknown("no active scope".into()));
    }
    if self.accu.trim().len() > 0 {
      return Err(Error::Syntax(
        format!("unprocessed expression: {}", self.accu),
        self.location.clone(),
      ));
    }
    let parent = self.cur_scope.borrow().parent().clone().unwrap();
    let last_kind = self.cur_scope.borrow().kind().clone();
    self.cur_scope = parent;
    if self.has_option(ParserOption::Debug) {
      println!(
        "pop_scope: {:?} -> {:?}",
        last_kind,
        self.cur_scope.borrow().kind()
      );
    }
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

  fn parse_expr(&mut self) -> Result<()> {
    Ok(())
  }

  fn parse_lbracket(&self, _ch: char) -> Result<()> {
    Ok(())
  }

  fn parse_rbracket(&self, _ch: char) -> Result<()> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  #[test]
  fn function_parsing_works() {
    let mut script = Script::new(
      PathBuf::from("virtual://test"),
      Some("test"),
      Some("function hello(p1, p2) {\n\t\"\";}"),
    );
    let mut p = Parser::default();
    let ast = p.parse(&mut script).unwrap();
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
