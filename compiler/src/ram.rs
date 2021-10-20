use super::common::*;

#[derive(Clone, Debug)]
pub struct Program {
  pub variables: Vec<Variable>,
  pub facts: Vec<Fact>,
  pub disjunctions: Vec<Disjunction>,
  pub updates: Vec<Update>,
}

#[derive(Clone, Debug)]
pub struct Fact {
  pub prob: Option<f32>,
  pub predicate: String,
  pub args: Vec<Constant>,
}

#[derive(Clone, Debug)]
pub struct Disjunction {
  pub id: usize,
  pub facts: Vec<Fact>,
}

#[derive(Clone, Debug)]
pub enum VarType {
  Empty,
  Base(Type),
  Tuple(Vec<VarType>),
}

#[derive(Clone, Debug)]
pub struct Variable {
  pub is_temporary: bool,
  pub name: String,
  pub arg_types: VarType,
}

#[derive(Clone, Debug)]
pub struct Update {
  pub into_var: String,
  pub flow: Flow,
}

#[derive(Clone, Debug)]
pub enum Flow {
  Product(Box<Flow>, Box<Flow>),
  Intersect(Box<Flow>, Box<Flow>),
  Join(Box<Flow>, Box<Flow>),
  Filter(Box<Flow>, Argument),
  Project(Box<Flow>, Argument),
  Find(Box<Flow>, Constant),
  ContainsChain(Box<Flow>, Vec<Constant>, Box<Flow>),
  Variable(String),
}

#[derive(Clone, Debug)]
pub enum Argument {
  /// Given the input tuple, use this index to get one of its element
  /// e.g. |((a, b), (c, d))| b ==> |arg| arg.0.1 ==> vec![0, 1]
  Element(Vec<usize>),
  Tuple(Vec<Argument>),
  Constant(Constant),
  Binary(BinaryOp, Box<Argument>, Box<Argument>),
  Unary(UnaryOp, Box<Argument>),
}

#[derive(Clone, Debug)]
pub enum Constant {
  Symbol(usize),
  Integer(i64),
  Boolean(bool),
  String(String),
}
