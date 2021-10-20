use super::*;

#[derive(Clone, Debug)]
pub struct Update {
  pub target: String,
  pub flow: Flow,
}

#[derive(Clone, Debug)]
pub enum Flow {
  Product(Box<Flow>, Box<Flow>),
  Intersect(Box<Flow>, Box<Flow>),
  Join(Box<Flow>, Box<Flow>),
  Filter(Box<Flow>, Expression),
  Project(Box<Flow>, Expression),
  Find(Box<Flow>, DynTuple),
  ContainsChain(Box<Flow>, DynTuple, Box<Flow>),
  StaticVariable(String),
  DynamicVariable(String),
}
