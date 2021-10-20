pub use super::common::Type as TypeNode;

use super::common::{BinaryOp, UnaryOp};
use super::location::*;

#[derive(Clone, Debug)]
pub struct Program {
  pub decls: Vec<Decl>,
  pub rules: Vec<Rule>,
  pub facts: Vec<Fact>,
  pub disjunctions: Vec<Disjunction>,
  pub queries: Vec<Query>,
}

impl Program {
  pub fn codify(&self) -> String {
    let decls = self.decls.iter().map(Decl::codify).collect::<Vec<_>>();
    let facts = self.facts.iter().map(Fact::codify).collect::<Vec<_>>();
    let rules = self.rules.iter().map(Rule::codify).collect::<Vec<_>>();
    let disjs = self.disjunctions.iter().map(Disjunction::codify).collect::<Vec<_>>();
    let queries = self.queries.iter().map(Query::codify).collect::<Vec<_>>();
    vec![decls, facts, rules, disjs, queries].into_iter().flatten().collect::<Vec<_>>().join("\n")
  }
}

impl Node for TypeNode {
  type T = Self;

  fn new(t: Self) -> Self {
    t
  }
}

pub type Type = Located<TypeNode>;

impl Type {
  pub fn codify(&self) -> String {
    format!("{}", self.node)
  }
}

#[derive(Clone, Debug)]
pub struct DeclNode {
  pub predicate: String,
  pub arg_types: Vec<Type>,
}

impl Node for DeclNode {
  type T = (String, Vec<Type>);

  fn new((predicate, arg_types): Self::T) -> Self {
    Self {
      predicate,
      arg_types,
    }
  }
}

pub type Decl = Located<DeclNode>;

impl Decl {
  pub fn codify(&self) -> String {
    format!("decl {}({}).", self.node.predicate, self.node.arg_types.iter().map(Type::codify).collect::<Vec<_>>().join(", "))
  }
}

#[derive(Clone, Debug)]
pub struct FactNode {
  pub prob: Option<f32>,
  pub head: Atom,
}

impl Node for FactNode {
  type T = (Option<f32>, Atom);

  fn new((prob, head): Self::T) -> Self {
    Self { prob, head }
  }
}

pub type Fact = Located<FactNode>;

impl Fact {
  pub fn codify(&self) -> String {
    match &self.node.prob {
      Some(prob) => format!("{}::{}.", prob, self.node.head.codify()),
      _ => format!("{}.", self.node.head.codify()),
    }
  }
}

#[derive(Clone, Debug)]
pub struct DisjunctionNode {
  pub facts: Vec<Fact>,
}

impl Node for DisjunctionNode {
  type T = Vec<Fact>;

  fn new(facts: Self::T) -> Self {
    Self { facts }
  }
}

pub type Disjunction = Located<DisjunctionNode>;

impl Disjunction {
  pub fn codify(&self) -> String {
    format!("{}.", self.node.facts.iter().map(Fact::codify).collect::<Vec<_>>().join(";\n"))
  }
}

#[derive(Clone, Debug)]
pub struct RuleNode {
  pub head: Atom,
  pub body: Vec<Literal>,
}

impl Node for RuleNode {
  type T = (Atom, Vec<Literal>);

  fn new((head, body): Self::T) -> Self {
    Self { head, body }
  }
}

pub type Rule = Located<RuleNode>;

impl Rule {
  pub fn codify(&self) -> String {
    format!("{} :- {}.", self.node.head.codify(), self.node.body.iter().map(Literal::codify).collect::<Vec<_>>().join(", "))
  }
}

#[derive(Clone, Debug)]
pub struct AtomNode {
  pub predicate: String,
  pub args: Vec<Argument>,
}

impl Node for AtomNode {
  type T = (String, Vec<Argument>);

  fn new((predicate, args): Self::T) -> Self {
    Self { predicate, args }
  }
}

pub type Atom = Located<AtomNode>;

impl Atom {
  pub fn codify(&self) -> String {
    format!("{}({})", self.node.predicate, self.node.args.iter().map(Argument::codify).collect::<Vec<_>>().join(", "))
  }
}

#[derive(Clone, Debug)]
pub enum LiteralNode {
  Pos(Atom),
  Neg(Atom),
  Constraint(Constraint),
}

impl Node for LiteralNode {
  type T = Self;

  fn new(data: Self) -> Self {
    data
  }
}

pub type Literal = Located<LiteralNode>;

impl Literal {
  pub fn codify(&self) -> String {
    match &self.node {
      LiteralNode::Pos(a) => a.codify(),
      LiteralNode::Neg(n) => format!("~{}", n.codify()),
      LiteralNode::Constraint(c) => c.codify(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Constraint {
  Binary(BinaryConstraint),
  Unary(UnaryConstraint),
}

impl Constraint {
  pub fn codify(&self) -> String {
    match self {
      Self::Binary(b) => b.codify(),
      Self::Unary(u) => u.codify(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct BinaryConstraintNode {
  pub op: BinaryOp,
  pub op1: Argument,
  pub op2: Argument,
}

impl Node for BinaryConstraintNode {
  type T = (BinaryOp, Argument, Argument);

  fn new((op, op1, op2): Self::T) -> Self {
    Self { op, op1, op2 }
  }
}

pub type BinaryConstraint = Located<BinaryConstraintNode>;

impl BinaryConstraint {
  pub fn codify(&self) -> String {
    format!("{} {} {}", self.node.op1.codify(), self.node.op.codify(), self.node.op2.codify())
  }
}

#[derive(Clone, Debug)]
pub struct UnaryConstraintNode {
  pub op: UnaryOp,
  pub op1: Argument,
}

impl Node for UnaryConstraintNode {
  type T = (UnaryOp, Argument);

  fn new((op, op1): Self::T) -> Self {
    Self { op, op1 }
  }
}

pub type UnaryConstraint = Located<UnaryConstraintNode>;

impl UnaryConstraint {
  pub fn codify(&self) -> String {
    format!("{}{}", self.node.op.codify(), self.node.op1.codify())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstantNode {
  Symbol(String),
  Boolean(bool),
  Integer(i64),
  SymbolId(usize),
  String(String),
}

impl Node for ConstantNode {
  type T = Self;

  fn new(t: Self::T) -> Self {
    t
  }
}

pub type Constant = Located<ConstantNode>;

impl Constant {
  pub fn codify(&self) -> String {
    match &self.node {
      ConstantNode::Symbol(s) => format!("{}", s),
      ConstantNode::Boolean(b) => format!("{}", b),
      ConstantNode::Integer(i) => format!("{}", i),
      ConstantNode::SymbolId(i) => format!("{}", i),
      ConstantNode::String(s) => format!("\"{}\"", s),
    }
  }
}

#[derive(Clone, Debug)]
pub struct VariableNode {
  pub name: String,
}

impl Node for VariableNode {
  type T = String;

  fn new(name: Self::T) -> Self {
    Self { name }
  }
}

pub type Variable = Located<VariableNode>;

impl Variable {
  pub fn codify(&self) -> String {
    self.node.name.clone()
  }
}

#[derive(Clone, Debug)]
pub enum Argument {
  Wildcard(Wildcard),
  Unary(UnaryExpr),
  Binary(BinaryExpr),
  Constant(Constant),
  Variable(Variable),
}

impl Argument {
  pub fn location(&self) -> &Location {
    match self {
      Self::Wildcard(w) => &w.location,
      Self::Unary(u) => &u.location,
      Self::Binary(b) => &b.location,
      Self::Constant(c) => &c.location,
      Self::Variable(v) => &v.location,
    }
  }

  pub fn codify(&self) -> String {
    match self {
      Self::Binary(b) => b.codify(),
      Self::Unary(u) => u.codify(),
      Self::Wildcard(w) => w.codify(),
      Self::Constant(c) => c.codify(),
      Self::Variable(v) => v.codify(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct WildcardNode;

impl Node for WildcardNode {
  type T = ();

  fn new((): Self::T) -> Self {
    Self
  }
}

pub type Wildcard = Located<WildcardNode>;

impl Wildcard {
  pub fn codify(&self) -> String {
    "_".to_string()
  }
}

#[derive(Clone, Debug)]
pub struct UnaryExprNode {
  pub op: UnaryOp,
  pub op1: Box<Argument>,
}

impl Node for UnaryExprNode {
  type T = (UnaryOp, Box<Argument>);

  fn new((op, op1): Self::T) -> Self {
    Self { op, op1 }
  }
}

pub type UnaryExpr = Located<UnaryExprNode>;

impl UnaryExpr {
  pub fn codify(&self) -> String {
    format!("{}{}", self.node.op.codify(), self.node.op1.codify())
  }
}

#[derive(Clone, Debug)]
pub struct BinaryExprNode {
  pub op: BinaryOp,
  pub op1: Box<Argument>,
  pub op2: Box<Argument>,
}

impl Node for BinaryExprNode {
  type T = (BinaryOp, Box<Argument>, Box<Argument>);

  fn new((op, op1, op2): Self::T) -> Self {
    Self { op, op1, op2 }
  }
}

pub type BinaryExpr = Located<BinaryExprNode>;

impl BinaryExpr {
  pub fn codify(&self) -> String {
    format!("{} {} {}", self.node.op1.codify(), self.node.op.codify(), self.node.op2.codify())
  }
}

#[derive(Clone, Debug)]
pub struct QueryNode {
  pub atom: Atom
}

impl Node for QueryNode {
  type T = Atom;

  fn new(atom: Self::T) -> Self {
    Self { atom }
  }
}

pub type Query = Located<QueryNode>;

impl Query {
  pub fn codify(&self) -> String {
    format!("query {}.", self.node.atom.codify())
  }
}
