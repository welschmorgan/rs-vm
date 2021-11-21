use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::{Rc, Weak};

use crate::error::Error;
use crate::location::Location;
use crate::result::Result;
use crate::script::Script;

use super::{Keyword, Scope, ScopeKind, ScopePtr, Value};

pub struct Variable {
  name: String,
  value: Value,
}

pub struct FunctionDecl {
  name: String,
  params: Vec<Variable>,
  return_type: Variable,
}

pub enum OpCode {
  // variable
  DeclareVariable(String),
  AssignVariable(String),

  // end of instruction
  FinishStatement,

  // function decl
  DeclareFunction(String),
  StartFunctionParams,
  AddFunctionParam(Variable),
  EndFunctionParams,
  StartFunctionImpl,
  EndFunctionImpl,
  SetReturnType(Variable),

  // function call
  CallFunction(String, Vec<Variable>),
  ReturnValue(Value),
}

impl Display for OpCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        &Self::DeclareVariable(..) => "declare_variable",
        &Self::AssignVariable(..) => "assign_variable",
        &Self::FinishStatement => "finish_statement",
        &Self::DeclareFunction(..) => "declare_function",
        &Self::StartFunctionParams => "start_function_params",
        &Self::AddFunctionParam(..) => "add_function_param",
        &Self::SetReturnType(..) => "set_return_type",
        &Self::EndFunctionParams => "",
        &Self::StartFunctionImpl => "start_function_impl",
        &Self::EndFunctionImpl => "end_function_impl",
        &Self::CallFunction(..) => "call_function",
        &Self::ReturnValue(..) => "return_value",
      }
    )
  }
}

pub struct Parser {
  location: Location,
  root_scope: ScopePtr,
  cur_scope: ScopePtr,
  next_scope: ScopeKind,
  accu: String,
  quote: Option<char>,
  keywords: Vec<Keyword>,
}

impl Default for Parser {
  fn default() -> Self {
    let root = Rc::new(RefCell::new(Scope::new(ScopeKind::Global)));
    Self {
      location: Default::default(),
      root_scope: root.clone(),
      cur_scope: root.clone(),
      next_scope: ScopeKind::None,
      accu: Default::default(),
      quote: Default::default(),
      keywords: Default::default(),
    }
  }
}

impl Parser {
  pub fn reset(&mut self) {
    *self = Parser::default();
  }

  pub fn parse(&mut self, s: &Script) -> Result<Vec<OpCode>> {
    self.reset();
    s.content().ok_or_else(|| {
      Error::IO(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "script has no content",
      ))
    })?;
    *self.location.file_mut() = s.name().clone();
    let res: Vec<OpCode> = Vec::new();
    let ch_it = s.content().unwrap().chars();
    for ch in ch_it {
      println!("parse: {}", ch);
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
    self.dump(self.root_scope.clone(), 0);
    Ok(res)
  }

  fn dump(&self, node: ScopePtr, indent: usize) {
    print!("{}", "\t".repeat(indent));
    if *node.borrow().kind() == ScopeKind::None {
      println!(
        "{} = {:?}",
        node.borrow().name().clone().unwrap_or("".into()),
        node.borrow().value());
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
    if *node.borrow().kind() != ScopeKind::None {
      println!("{}}}", "\t".repeat(indent))
    }
  }

  fn parse_keyword(&mut self) -> Result<()> {
    println!("parse kw: {:?}", self.accu);
    if let Some(kw) = Keyword::parse(&self.accu) {
      match kw {
        Keyword::Function => {
          self.next_scope = ScopeKind::Function;
          self.push_scope(self.next_scope)?;
        }
        Keyword::Class => {
          self.next_scope = ScopeKind::Class;
        }
        Keyword::Enum => {
          self.next_scope = ScopeKind::Enum;
        }
        Keyword::Private => {}
        Keyword::Protected => {}
        Keyword::Public => {}
        Keyword::Return => {}
      };
      self.keywords.push(kw);
      self.accu.clear();
    }
    Ok(())
  }

  fn push_fn_param(&mut self) -> Result<()> {
    *self
      .cur_scope
      .borrow_mut()
      .create_child(ScopeKind::FunctionParam)
      .borrow_mut()
      .name_mut() = Some(self.accu.clone());
    self.accu.clear();
    Ok(())
  }

  fn parse_comma(&mut self, ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == ScopeKind::FunctionParams {
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

  fn parse_lparen(&mut self, ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == ScopeKind::Function {
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
        .find(|ch| *ch.borrow().kind() == ScopeKind::FunctionParams)
        .is_some()
      {
        return Err(Error::Syntax(
          "unexpected '('".into(),
          self.location.clone(),
        ));
      }
      self.push_scope(ScopeKind::FunctionParams)?;
    }
    Ok(())
  }

  fn parse_rparen(&mut self, ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == ScopeKind::FunctionParams {
      if self.accu.trim().len() != 0 {
        self.push_fn_param()?;
      }
    }
    self.pop_scope()?;
    Ok(())
  }

  fn parse_lbrace(&mut self, ch: char) -> Result<()> {
    if *self.cur_scope.borrow().kind() == ScopeKind::Function {
      *self.cur_scope.borrow_mut().kind_mut() = ScopeKind::Function;
    } else {
      self.push_scope(self.next_scope)?;
      self.next_scope = *self.cur_scope.borrow().kind();
    }
    Ok(())
  }

  fn parse_rbrace(&mut self, ch: char) -> Result<()> {
    self.pop_scope()?;
    Ok(())
  }

  fn parse_semicolon(&mut self, ch: char) -> Result<()> {
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

  fn parse_space(&mut self, ch: char) -> Result<()> {
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

  fn parse_unknown(&mut self, ch: char) -> Result<()> {
    Ok(())
  }

  fn push_scope(&mut self, kind: ScopeKind) -> Result<ScopePtr> {
    let last_scope = self.cur_scope.clone();
    self.cur_scope = last_scope.borrow_mut().create_child(kind).clone();
    *self.cur_scope.borrow_mut().parent_mut() = Some(last_scope.clone());
    println!(
      "push_scope: {:?} -> {:?}",
      last_scope.borrow().kind(),
      self.cur_scope.borrow().kind()
    );
    Ok(self.cur_scope.clone())
  }

  fn pop_scope(&mut self) -> Result<ScopePtr> {
    if self.cur_scope.borrow().parent().is_none() {
      return Err(Error::Unknown("no active scope".into()));
    }
    let parent = self.cur_scope.borrow().parent().clone().unwrap();
    let last_kind = self.cur_scope.borrow().parent_kind().unwrap();
    self.cur_scope = parent;
    println!(
      "pop_scope: {:?} -> {:?}",
      last_kind,
      self.cur_scope.borrow().kind()
    );
    Ok(self.cur_scope.clone())
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  #[test]
  fn scope_parsing_works() {
    let script = Script::new(
      PathBuf::from("virtual://test"),
      Some("test"),
      Some("function hello(p1, p2) {\n\t\"\";}"),
    );
    let mut p = Parser::default();
    let op_codes = p.parse(&script).unwrap();
  }
}
