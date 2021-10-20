use std::sync::Arc;

use super::*;
use crate::*;

#[derive(Debug, Clone)]
pub enum Expression {
  Tuple(Vec<Expression>),
  Access(TupleAccessor),
  Constant(Constant),
  Binary(Binary),
  Unary(Unary),
}

impl Expression {
  pub fn eval(&self, comp: &DynTuple) -> DynTuple {
    match self {
      Self::Tuple(exprs) => DynTuple::Tuple(exprs.iter().map(|expr| expr.eval(comp)).collect()),
      Self::Access(acc) => comp[acc.clone()].clone(),
      Self::Constant(cst) => cst.eval(),
      Self::Binary(bin) => bin.eval(comp),
      Self::Unary(una) => una.eval(comp),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Constant {
  Integer(i64),
  Boolean(bool),
  String(Arc<String>),
  Symbol(usize),
}

impl Constant {
  pub fn eval(&self) -> DynTuple {
    match self {
      Self::Integer(i) => DynTuple::Integer(i.clone()),
      Self::Boolean(b) => DynTuple::Boolean(b.clone()),
      Self::String(s) => DynTuple::String(s.clone()),
      Self::Symbol(s) => DynTuple::Symbol(s.clone()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Binary {
  pub op: BinaryOp,
  pub lhs: Box<Expression>,
  pub rhs: Box<Expression>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
  And,
  Or,
  Eq,
  Ne,
  Lt,
  Lte,
  Gt,
  Gte,
}

impl Binary {
  pub fn eval(&self, comp: &DynTuple) -> DynTuple {
    let c1 = self.lhs.eval(comp);
    let c2 = self.rhs.eval(comp);
    match self.op {
      BinaryOp::Add => c1 + c2,
      BinaryOp::Sub => c1 - c2,
      BinaryOp::Mul => c1 * c2,
      BinaryOp::Div => c1 / c2,
      BinaryOp::And => c1 & c2,
      BinaryOp::Or => c1 | c2,
      BinaryOp::Eq => DynTuple::Boolean(c1 == c2),
      BinaryOp::Ne => DynTuple::Boolean(c1 != c2),
      BinaryOp::Lt => match (c1, c2) {
        (DynTuple::Integer(i1), DynTuple::Integer(i2)) => DynTuple::Boolean(i1 < i2),
        _ => panic!("Invalid < operation"),
      },
      BinaryOp::Lte => match (c1, c2) {
        (DynTuple::Integer(i1), DynTuple::Integer(i2)) => DynTuple::Boolean(i1 <= i2),
        _ => panic!("Invalid <= operation"),
      },
      BinaryOp::Gt => match (c1, c2) {
        (DynTuple::Integer(i1), DynTuple::Integer(i2)) => DynTuple::Boolean(i1 > i2),
        _ => panic!("Invalid > operation"),
      },
      BinaryOp::Gte => match (c1, c2) {
        (DynTuple::Integer(i1), DynTuple::Integer(i2)) => DynTuple::Boolean(i1 >= i2),
        _ => panic!("Invalid >= operation"),
      },
    }
  }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
  Not,
  Neg,
  Pos,
}

#[derive(Debug, Clone)]
pub struct Unary {
  pub op: UnaryOp,
  pub op0: Box<Expression>,
}

impl Unary {
  pub fn eval(&self, comp: &DynTuple) -> DynTuple {
    match self.op {
      UnaryOp::Not => !self.op0.eval(comp),
      UnaryOp::Neg => -self.op0.eval(comp),
      UnaryOp::Pos => self.op0.eval(comp),
    }
  }
}
