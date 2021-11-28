use std::fmt::Display;

use super::{Value, Variable};


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
