use super::ast;
use super::error::*;
use super::location;

macro_rules! node_visitor_mut_func_def {
  ($func:ident, $node:ident) => {
    fn $func(&mut self, _: &mut ast::$node) -> Result<(), CompileError> {
      Ok(())
    }
  };
}

pub trait NodeVisitorMut {
  fn visit_location(&mut self, _: &mut location::Location) -> Result<(), CompileError> {
    Ok(())
  }

  node_visitor_mut_func_def!(visit_decl, Decl);
  node_visitor_mut_func_def!(visit_type, Type);
  node_visitor_mut_func_def!(visit_fact, Fact);
  node_visitor_mut_func_def!(visit_disjunction, Disjunction);
  node_visitor_mut_func_def!(visit_rule, Rule);
  node_visitor_mut_func_def!(visit_binary_constraint, BinaryConstraint);
  node_visitor_mut_func_def!(visit_unary_constraint, UnaryConstraint);
  node_visitor_mut_func_def!(visit_literal, Literal);
  node_visitor_mut_func_def!(visit_atom, Atom);
  node_visitor_mut_func_def!(visit_arg, Argument);
  node_visitor_mut_func_def!(visit_constant, Constant);
  node_visitor_mut_func_def!(visit_wildcard, Wildcard);
  node_visitor_mut_func_def!(visit_binary, BinaryExpr);
  node_visitor_mut_func_def!(visit_unary, UnaryExpr);
  node_visitor_mut_func_def!(visit_variable, Variable);
  node_visitor_mut_func_def!(visit_query, Query);
}

macro_rules! node_visitor_mut_visit_node {
  ($func:ident, $node:ident, ($($elem:ident),*)) => {
    #[allow(unused_variables)]
    fn $func(&mut self, node: &mut ast::$node) -> Result<(), CompileError> {
      paste::item! { let ($( [<$elem:lower>],)*) = self; }
      $( paste::item! { [<$elem:lower>].$func(node)?; } )*
      Ok(())
    }
  };
}

macro_rules! impl_node_visitor_mut_tuple {
  ( $($id:ident,)* ) => {
    impl<$($id,)*> NodeVisitorMut for ($($id,)*)
    where
      $($id: NodeVisitorMut,)*
    {
      node_visitor_mut_visit_node!(visit_decl, Decl, ($($id),*));
      node_visitor_mut_visit_node!(visit_type, Type, ($($id),*));
      node_visitor_mut_visit_node!(visit_fact, Fact, ($($id),*));
      node_visitor_mut_visit_node!(visit_disjunction, Disjunction, ($($id),*));
      node_visitor_mut_visit_node!(visit_rule, Rule, ($($id),*));
      node_visitor_mut_visit_node!(visit_binary_constraint, BinaryConstraint, ($($id),*));
      node_visitor_mut_visit_node!(visit_unary_constraint, UnaryConstraint, ($($id),*));
      node_visitor_mut_visit_node!(visit_literal, Literal, ($($id),*));
      node_visitor_mut_visit_node!(visit_atom, Atom, ($($id),*));
      node_visitor_mut_visit_node!(visit_arg, Argument, ($($id),*));
      node_visitor_mut_visit_node!(visit_constant, Constant, ($($id),*));
      node_visitor_mut_visit_node!(visit_wildcard, Wildcard, ($($id),*));
      node_visitor_mut_visit_node!(visit_binary, BinaryExpr, ($($id),*));
      node_visitor_mut_visit_node!(visit_unary, UnaryExpr, ($($id),*));
      node_visitor_mut_visit_node!(visit_variable, Variable, ($($id),*));
      node_visitor_mut_visit_node!(visit_query, Query, ($($id),*));
    }
  }
}

impl_node_visitor_mut_tuple!();
impl_node_visitor_mut_tuple!(A,);
impl_node_visitor_mut_tuple!(A, B,);
impl_node_visitor_mut_tuple!(A, B, C,);
impl_node_visitor_mut_tuple!(A, B, C, D,);
impl_node_visitor_mut_tuple!(A, B, C, D, E,);
impl_node_visitor_mut_tuple!(A, B, C, D, E, F,);
impl_node_visitor_mut_tuple!(A, B, C, D, E, F, G,);
impl_node_visitor_mut_tuple!(A, B, C, D, E, F, G, H,);
impl_node_visitor_mut_tuple!(A, B, C, D, E, F, G, H, I,);

pub fn visit_arg_mut(
  visitor: &mut impl NodeVisitorMut,
  arg: &mut ast::Argument,
) -> Result<(), CompileError> {
  visitor.visit_arg(arg)?;
  match arg {
    ast::Argument::Constant(c) => {
      visitor.visit_constant(c)?;
      visitor.visit_location(&mut c.location)
    }
    ast::Argument::Wildcard(w) => {
      visitor.visit_wildcard(w)?;
      visitor.visit_location(&mut w.location)
    }
    ast::Argument::Binary(b) => {
      visitor.visit_binary(b)?;
      visitor.visit_location(&mut b.location)?;
      visit_arg_mut(visitor, &mut b.node.op1)?;
      visit_arg_mut(visitor, &mut b.node.op2)
    }
    ast::Argument::Unary(u) => {
      visitor.visit_unary(u)?;
      visitor.visit_location(&mut u.location)?;
      visit_arg_mut(visitor, &mut u.node.op1)
    }
    ast::Argument::Variable(v) => {
      visitor.visit_variable(v)?;
      visitor.visit_location(&mut v.location)
    }
  }
}

pub fn visit_atom_mut(
  visitor: &mut impl NodeVisitorMut,
  atom: &mut ast::Atom,
) -> Result<(), CompileError> {
  visitor.visit_atom(atom)?;
  visitor.visit_location(&mut atom.location)?;
  for arg in &mut atom.node.args {
    visit_arg_mut(visitor, arg)?;
  }
  Ok(())
}

pub fn visit_constraint_mut(
  visitor: &mut impl NodeVisitorMut,
  constraint: &mut ast::Constraint,
) -> Result<(), CompileError> {
  match constraint {
    ast::Constraint::Binary(b) => {
      visitor.visit_binary_constraint(b)?;
      visitor.visit_location(&mut b.location)?;
      visit_arg_mut(visitor, &mut b.node.op1)?;
      visit_arg_mut(visitor, &mut b.node.op2)?;
    }
    ast::Constraint::Unary(u) => {
      visitor.visit_unary_constraint(u)?;
      visitor.visit_location(&mut u.location)?;
      visit_arg_mut(visitor, &mut u.node.op1)?;
    }
  }
  Ok(())
}

pub fn visit_literal_mut(
  visitor: &mut impl NodeVisitorMut,
  literal: &mut ast::Literal,
) -> Result<(), CompileError> {
  visitor.visit_literal(literal)?;
  visitor.visit_location(&mut literal.location)?;
  match &mut literal.node {
    ast::LiteralNode::Pos(a) => visit_atom_mut(visitor, a),
    ast::LiteralNode::Neg(n) => visit_atom_mut(visitor, n),
    ast::LiteralNode::Constraint(c) => visit_constraint_mut(visitor, c),
  }
}

pub fn visit_fact_mut(
  visitor: &mut impl NodeVisitorMut,
  fact: &mut ast::Fact,
) -> Result<(), CompileError> {
  visitor.visit_fact(fact)?;
  visitor.visit_location(&mut fact.location)?;
  visit_atom_mut(visitor, &mut fact.node.head)
}

pub fn visit_rule_mut(
  visitor: &mut impl NodeVisitorMut,
  rule: &mut ast::Rule,
) -> Result<(), CompileError> {
  visitor.visit_rule(rule)?;
  visitor.visit_location(&mut rule.location)?;
  visit_atom_mut(visitor, &mut rule.node.head)?;
  for literal in &mut rule.node.body {
    visit_literal_mut(visitor, literal)?;
  }
  Ok(())
}

pub fn visit_query_mut(visitor: &mut impl NodeVisitorMut, query: &mut ast::Query) -> Result<(), CompileError> {
  visitor.visit_query(query)?;
  visitor.visit_location(&mut query.location)?;
  visit_atom_mut(visitor, &mut query.node.atom)
}

pub fn visit_program_mut(
  visitor: &mut impl NodeVisitorMut,
  prog: &mut ast::Program,
) -> Result<(), CompileError> {
  for decl in &mut prog.decls {
    visitor.visit_decl(decl)?;
    visitor.visit_location(&mut decl.location)?;
    for ty in &mut decl.node.arg_types {
      visitor.visit_type(ty)?;
      visitor.visit_location(&mut ty.location)?;
    }
  }
  for fact in &mut prog.facts {
    visit_fact_mut(visitor, fact)?;
  }
  for disjunction in &mut prog.disjunctions {
    visitor.visit_disjunction(disjunction)?;
    for fact in &mut disjunction.node.facts {
      visit_fact_mut(visitor, fact)?;
    }
  }
  for rule in &mut prog.rules {
    visit_rule_mut(visitor, rule)?;
  }
  for query in &mut prog.queries {
    visit_query_mut(visitor, query)?;
  }
  Ok(())
}

macro_rules! node_visitor_func_def {
  ($func:ident, $node:ident) => {
    fn $func(&mut self, _: &ast::$node) -> Result<(), CompileError> {
      Ok(())
    }
  };
}

pub trait NodeVisitor {
  fn visit_location(&mut self, _: &location::Location) -> Result<(), CompileError> {
    Ok(())
  }

  node_visitor_func_def!(visit_decl, Decl);
  node_visitor_func_def!(visit_type, Type);
  node_visitor_func_def!(visit_fact, Fact);
  node_visitor_func_def!(visit_disjunction, Disjunction);
  node_visitor_func_def!(visit_rule, Rule);
  node_visitor_func_def!(visit_binary_constraint, BinaryConstraint);
  node_visitor_func_def!(visit_unary_constraint, UnaryConstraint);
  node_visitor_func_def!(visit_literal, Literal);
  node_visitor_func_def!(visit_atom, Atom);
  node_visitor_func_def!(visit_arg, Argument);
  node_visitor_func_def!(visit_constant, Constant);
  node_visitor_func_def!(visit_wildcard, Wildcard);
  node_visitor_func_def!(visit_binary, BinaryExpr);
  node_visitor_func_def!(visit_unary, UnaryExpr);
  node_visitor_func_def!(visit_variable, Variable);
  node_visitor_func_def!(visit_query, Query);
}

macro_rules! node_visitor_visit_node {
  ($func:ident, $node:ident, ($($elem:ident),*)) => {
    #[allow(unused_variables)]
    fn $func(&mut self, node: &ast::$node) -> Result<(), CompileError> {
      paste::item! { let ($( [<$elem:lower>],)*) = self; }
      $( paste::item! { [<$elem:lower>].$func(node)?; } )*
      Ok(())
    }
  };
}

macro_rules! impl_node_visitor_tuple {
  ( $($id:ident,)* ) => {
    impl<$($id,)*> NodeVisitor for ($($id,)*)
    where
      $($id: NodeVisitor,)*
    {
      node_visitor_visit_node!(visit_decl, Decl, ($($id),*));
      node_visitor_visit_node!(visit_type, Type, ($($id),*));
      node_visitor_visit_node!(visit_fact, Fact, ($($id),*));
      node_visitor_visit_node!(visit_disjunction, Disjunction, ($($id),*));
      node_visitor_visit_node!(visit_rule, Rule, ($($id),*));
      node_visitor_visit_node!(visit_binary_constraint, BinaryConstraint, ($($id),*));
      node_visitor_visit_node!(visit_unary_constraint, UnaryConstraint, ($($id),*));
      node_visitor_visit_node!(visit_literal, Literal, ($($id),*));
      node_visitor_visit_node!(visit_atom, Atom, ($($id),*));
      node_visitor_visit_node!(visit_arg, Argument, ($($id),*));
      node_visitor_visit_node!(visit_constant, Constant, ($($id),*));
      node_visitor_visit_node!(visit_wildcard, Wildcard, ($($id),*));
      node_visitor_visit_node!(visit_binary, BinaryExpr, ($($id),*));
      node_visitor_visit_node!(visit_unary, UnaryExpr, ($($id),*));
      node_visitor_visit_node!(visit_variable, Variable, ($($id),*));
      node_visitor_visit_node!(visit_query, Query, ($($id),*));
    }
  }
}

impl_node_visitor_tuple!();
impl_node_visitor_tuple!(A,);
impl_node_visitor_tuple!(A, B,);
impl_node_visitor_tuple!(A, B, C,);
impl_node_visitor_tuple!(A, B, C, D,);
impl_node_visitor_tuple!(A, B, C, D, E,);
impl_node_visitor_tuple!(A, B, C, D, E, F,);
impl_node_visitor_tuple!(A, B, C, D, E, F, G,);
impl_node_visitor_tuple!(A, B, C, D, E, F, G, H,);
impl_node_visitor_tuple!(A, B, C, D, E, F, G, H, I,);

pub fn visit_arg(visitor: &mut impl NodeVisitor, arg: &ast::Argument) -> Result<(), CompileError> {
  visitor.visit_arg(arg)?;
  match arg {
    ast::Argument::Constant(c) => {
      visitor.visit_constant(c)?;
      visitor.visit_location(&c.location)
    }
    ast::Argument::Wildcard(w) => {
      visitor.visit_wildcard(w)?;
      visitor.visit_location(&w.location)
    }
    ast::Argument::Binary(b) => {
      visitor.visit_binary(b)?;
      visitor.visit_location(&b.location)?;
      visit_arg(visitor, &b.node.op1)?;
      visit_arg(visitor, &b.node.op2)
    }
    ast::Argument::Unary(u) => {
      visitor.visit_unary(u)?;
      visitor.visit_location(&u.location)?;
      visit_arg(visitor, &u.node.op1)
    }
    ast::Argument::Variable(v) => {
      visitor.visit_variable(v)?;
      visitor.visit_location(&v.location)
    }
  }
}

pub fn visit_atom(visitor: &mut impl NodeVisitor, atom: &ast::Atom) -> Result<(), CompileError> {
  visitor.visit_atom(atom)?;
  visitor.visit_location(&atom.location)?;
  for arg in &atom.node.args {
    visit_arg(visitor, arg)?;
  }
  Ok(())
}

pub fn visit_constraint(
  visitor: &mut impl NodeVisitor,
  constraint: &ast::Constraint,
) -> Result<(), CompileError> {
  match constraint {
    ast::Constraint::Binary(b) => {
      visitor.visit_binary_constraint(b)?;
      visitor.visit_location(&b.location)?;
      visit_arg(visitor, &b.node.op1)?;
      visit_arg(visitor, &b.node.op2)?;
    }
    ast::Constraint::Unary(u) => {
      visitor.visit_unary_constraint(u)?;
      visitor.visit_location(&u.location)?;
      visit_arg(visitor, &u.node.op1)?;
    }
  }
  Ok(())
}

pub fn visit_literal(
  visitor: &mut impl NodeVisitor,
  literal: &ast::Literal,
) -> Result<(), CompileError> {
  visitor.visit_literal(literal)?;
  visitor.visit_location(&literal.location)?;
  match &literal.node {
    ast::LiteralNode::Pos(a) => visit_atom(visitor, a),
    ast::LiteralNode::Neg(n) => visit_atom(visitor, n),
    ast::LiteralNode::Constraint(c) => visit_constraint(visitor, c),
  }
}

pub fn visit_fact(visitor: &mut impl NodeVisitor, fact: &ast::Fact) -> Result<(), CompileError> {
  visitor.visit_fact(fact)?;
  visitor.visit_location(&fact.location)?;
  visit_atom(visitor, &fact.node.head)
}

pub fn visit_rule(visitor: &mut impl NodeVisitor, rule: &ast::Rule) -> Result<(), CompileError> {
  visitor.visit_rule(rule)?;
  visitor.visit_location(&rule.location)?;
  visit_atom(visitor, &rule.node.head)?;
  for literal in &rule.node.body {
    visit_literal(visitor, literal)?;
  }
  Ok(())
}

pub fn visit_query(visitor: &mut impl NodeVisitor, query: &ast::Query) -> Result<(), CompileError> {
  visitor.visit_query(query)?;
  visitor.visit_location(&query.location)?;
  visit_atom(visitor, &query.node.atom)
}

pub fn visit_program(
  visitor: &mut impl NodeVisitor,
  prog: &ast::Program,
) -> Result<(), CompileError> {
  for decl in &prog.decls {
    visitor.visit_decl(decl)?;
    visitor.visit_location(&decl.location)?;
    for ty in &decl.node.arg_types {
      visitor.visit_type(ty)?;
      visitor.visit_location(&ty.location)?;
    }
  }
  for fact in &prog.facts {
    visit_fact(visitor, fact)?;
  }
  for disjunction in &prog.disjunctions {
    visitor.visit_disjunction(disjunction)?;
    for fact in &disjunction.node.facts {
      visit_fact(visitor, fact)?;
    }
  }
  for rule in &prog.rules {
    visit_rule(visitor, rule)?;
  }
  for query in &prog.queries {
    visit_query(visitor, query)?;
  }
  Ok(())
}
