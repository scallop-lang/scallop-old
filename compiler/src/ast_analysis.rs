use std::collections::*;

use super::common::*;
use super::error::*;
use super::location::*;
use super::visitor::*;
use super::*;
use super::options::CompileOptions;

#[derive(Debug, Clone)]
pub struct AnalysisResult {
  pub is_probabilistic: bool,
  pub decls: Decls,
  pub node_types: NodeTypeMap,
  pub disj_rela_map: DisjunctionRelationMap,
  pub demands: Demands,
}

impl Default for AnalysisResult {
  fn default() -> Self {
    Self {
      is_probabilistic: false,
      decls: Decls::new(),
      node_types: NodeTypeMap::new(),
      disj_rela_map: DisjunctionRelationMap::new(),
      demands: Demands::new(),
    }
  }
}

pub struct IsProbabilisticAnalyzer {
  pub is_probabilistic: bool,
}

impl IsProbabilisticAnalyzer {
  pub fn new() -> Self {
    Self {
      is_probabilistic: false,
    }
  }
}

impl NodeVisitor for IsProbabilisticAnalyzer {
  fn visit_fact(&mut self, fact: &ast::Fact) -> Result<(), CompileError> {
    if fact.node.prob.is_some() {
      self.is_probabilistic = true;
    }
    Ok(())
  }

  fn visit_disjunction(&mut self, _: &ast::Disjunction) -> Result<(), CompileError> {
    self.is_probabilistic = true;
    Ok(())
  }
}

pub type DisjunctionRelationMap = HashMap<usize, String>;

pub struct DisjunctionOnSameRelationChecker {
  pub disj_rela_map: DisjunctionRelationMap,
}

impl DisjunctionOnSameRelationChecker {
  pub fn new() -> Self {
    Self {
      disj_rela_map: DisjunctionRelationMap::new(),
    }
  }
}

impl NodeVisitor for DisjunctionOnSameRelationChecker {
  fn visit_disjunction(&mut self, disjunction: &ast::Disjunction) -> Result<(), CompileError> {
    let mut relation = None;
    for fact in &disjunction.node.facts {
      match &relation {
        None => {
          relation = Some(fact.node.head.node.predicate.clone());
        }
        Some(curr_relation) => {
          if curr_relation != &fact.node.head.node.predicate {
            return Err(CompileError::DisjunctionHasDifferentRelation {
              loc: disjunction.location.clone(),
              expected: curr_relation.clone(),
              found: fact.node.head.node.predicate.clone(),
            });
          }
        }
      }
    }
    let relation = relation.unwrap();
    self.disj_rela_map.insert(disjunction.location.id, relation);
    Ok(())
  }
}

pub struct UnboundedVariableAnalyzer;

impl NodeVisitor for UnboundedVariableAnalyzer {
  fn visit_rule(&mut self, rule: &ast::Rule) -> Result<(), CompileError> {
    fn get_var_name(arg: &ast::Argument) -> Option<String> {
      match arg {
        ast::Argument::Variable(v) => Some(v.node.name.clone()),
        _ => None,
      }
    }

    fn try_get_and_insert_var_name(
      arg: &ast::Argument,
      set: &mut HashMap<String, Vec<Location>>,
    ) -> Result<(), CompileError> {
      if let Some(name) = get_var_name(arg) {
        set.entry(name).or_default().push(arg.location().clone());
      }
      Ok(())
    }

    struct VarsInHead {
      set: HashMap<String, Vec<Location>>,
    }

    impl NodeVisitor for VarsInHead {
      fn visit_arg(&mut self, arg: &ast::Argument) -> Result<(), CompileError> {
        try_get_and_insert_var_name(arg, &mut self.set)
      }
    }

    struct VarsInExprs {
      set: HashMap<String, Vec<Location>>,
    }

    impl NodeVisitor for VarsInExprs {
      fn visit_binary(&mut self, bin: &ast::BinaryExpr) -> Result<(), CompileError> {
        try_get_and_insert_var_name(&bin.node.op1, &mut self.set)?;
        try_get_and_insert_var_name(&bin.node.op2, &mut self.set)
      }

      fn visit_unary(&mut self, una: &ast::UnaryExpr) -> Result<(), CompileError> {
        try_get_and_insert_var_name(&una.node.op1, &mut self.set)
      }

      fn visit_binary_constraint(
        &mut self,
        bin: &ast::BinaryConstraint,
      ) -> Result<(), CompileError> {
        try_get_and_insert_var_name(&bin.node.op1, &mut self.set)?;
        try_get_and_insert_var_name(&bin.node.op2, &mut self.set)
      }

      fn visit_unary_constraint(&mut self, una: &ast::UnaryConstraint) -> Result<(), CompileError> {
        try_get_and_insert_var_name(&una.node.op1, &mut self.set)
      }
    }

    struct VarsInAtomArg {
      set: HashSet<String>,
    }

    impl NodeVisitor for VarsInAtomArg {
      fn visit_atom(&mut self, atom: &ast::Atom) -> Result<(), CompileError> {
        for arg in &atom.node.args {
          if let Some(name) = get_var_name(arg) {
            self.set.insert(name);
          }
        }
        Ok(())
      }
    }

    // First obtain all the vars appeared in head
    let mut vars_in_head = VarsInHead {
      set: HashMap::new(),
    };
    visit_atom(&mut vars_in_head, &rule.node.head)?;

    // Then obtain all the vars appeared in constraints and expressions; and also in atoms
    let mut visitors = (
      VarsInExprs {
        set: HashMap::new(),
      },
      VarsInAtomArg {
        set: HashSet::new(),
      },
    );
    for literal in &rule.node.body {
      visit_literal(&mut visitors, literal)?;
    }
    let vars_in_exprs = visitors.0;
    let vars_in_atom_arg = visitors.1;

    for (name, locs) in vars_in_exprs.set.iter().chain(vars_in_head.set.iter()) {
      if !vars_in_atom_arg.set.contains(name) {
        return Err(CompileError::UnboundedVariable {
          rule_loc: rule.location.clone(),
          var_loc: locs[0].clone(),
          var_name: name.clone(),
        });
      }
    }
    Ok(())
  }
}

pub struct FactHasOnlyConstantAnalyzer;

impl NodeVisitor for FactHasOnlyConstantAnalyzer {
  fn visit_fact(&mut self, fact: &ast::Fact) -> Result<(), CompileError> {
    for arg in &fact.node.head.node.args {
      match arg {
        ast::Argument::Constant(_) => {}
        _ => {
          return Err(CompileError::FactWithNonConstant {
            loc: arg.location().clone(),
          })
        }
      }
    }
    Ok(())
  }
}

pub struct NoExprInBodyAtomAnalyzer;

impl NodeVisitor for NoExprInBodyAtomAnalyzer {
  fn visit_rule(&mut self, rule: &ast::Rule) -> Result<(), CompileError> {
    for literal in &rule.node.body {
      match &literal.node {
        ast::LiteralNode::Pos(a) | ast::LiteralNode::Neg(a) => {
          for arg in &a.node.args {
            match arg {
              ast::Argument::Binary(_) | ast::Argument::Unary(_) => {
                return Err(CompileError::ExpressionInBodyLiteral {
                  loc: arg.location().clone(),
                })
              }
              _ => {}
            }
          }
        }
        _ => {}
      }
    }
    Ok(())
  }

  fn visit_query(&mut self, query: &ast::Query) -> Result<(), CompileError> {
    for arg in &query.node.atom.node.args {
      match arg {
        ast::Argument::Binary(_) | ast::Argument::Unary(_) => {
          return Err(CompileError::ExpressionInQuery {
            loc: arg.location().clone(),
          })
        }
        _ => {}
      }
    }
    Ok(())
  }
}

// Create an inner visitor
struct NoWildcard;

impl NodeVisitor for NoWildcard {
  fn visit_arg(&mut self, arg: &ast::Argument) -> Result<(), CompileError> {
    match arg {
      ast::Argument::Wildcard(w) => Err(CompileError::InvalidWildcard {
        loc: w.location.clone(),
      }),
      _ => Ok(()),
    }
  }
}

pub struct InvalidWildcardAnalyzer;

impl NodeVisitor for InvalidWildcardAnalyzer {
  fn visit_rule(&mut self, rule: &ast::Rule) -> Result<(), CompileError> {
    visit_atom(&mut NoWildcard, &rule.node.head)
  }

  fn visit_binary(&mut self, binary_expr: &ast::BinaryExpr) -> Result<(), CompileError> {
    visit_arg(&mut NoWildcard, &binary_expr.node.op1)?;
    visit_arg(&mut NoWildcard, &binary_expr.node.op2)?;
    Ok(())
  }

  fn visit_unary(&mut self, unary_expr: &ast::UnaryExpr) -> Result<(), CompileError> {
    visit_arg(&mut NoWildcard, &unary_expr.node.op1)
  }

  fn visit_binary_constraint(&mut self, bin: &ast::BinaryConstraint) -> Result<(), CompileError> {
    visit_arg(&mut NoWildcard, &bin.node.op1)?;
    visit_arg(&mut NoWildcard, &bin.node.op2)?;
    Ok(())
  }

  fn visit_unary_constraint(&mut self, una: &ast::UnaryConstraint) -> Result<(), CompileError> {
    visit_arg(&mut NoWildcard, &una.node.op1)
  }
}

pub type Decls = HashMap<String, Vec<Type>>;

pub type NodeTypeMap = HashMap<usize, Type>;

#[derive(Debug)]
pub enum ToUnifyArg {
  /// Constant Node Id
  Integer(usize),

  /// Variable Node Id, Variable Name
  Variable(usize, String),
}

pub type ToUnifyArgs = Vec<(usize, ToUnifyArg, ToUnifyArg)>;

pub struct TypeAssign {
  pub decls: Decls,
  pub node_types: NodeTypeMap,
  pub rule_arg_map: HashMap<usize, usize>,
  pub to_unify_args: ToUnifyArgs,
}

impl TypeAssign {
  pub fn new() -> Self {
    Self {
      decls: Decls::new(),
      node_types: NodeTypeMap::new(),
      rule_arg_map: HashMap::new(),
      to_unify_args: ToUnifyArgs::new(),
    }
  }

  pub fn type_of(&self, node_id: &usize) -> &Type {
    &self.node_types[node_id]
  }
}

fn type_of_arg(
  node_types: &mut NodeTypeMap,
  to_unify_args: &mut ToUnifyArgs,
  rule_arg_map: &HashMap<usize, usize>,
  arg: &ast::Argument,
) -> Result<Option<Type>, CompileError> {
  match arg {
    ast::Argument::Constant(c) => match &c.node {
      ast::ConstantNode::Boolean(_) => Ok(Some(Type::Boolean)),
      ast::ConstantNode::Integer(_) => Ok(None),
      ast::ConstantNode::Symbol(_) => Ok(Some(Type::Symbol)),
      ast::ConstantNode::SymbolId(_) => Ok(Some(Type::Symbol)),
      ast::ConstantNode::String(_) => Ok(Some(Type::String)),
    },
    ast::Argument::Binary(b) => match &b.node.op {
      BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mult | BinaryOp::Div => {
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &Type::Integer,
        )?;
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op2,
          &Type::Integer,
        )?;
        Ok(Some(Type::Integer))
      }
      BinaryOp::Eq | BinaryOp::Ne => {
        unify_two_args(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &b.node.op2,
        )?;
        Ok(Some(Type::Boolean))
      }
      BinaryOp::Gt | BinaryOp::Gte | BinaryOp::Lt | BinaryOp::Lte => {
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &Type::Integer,
        )?;
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op2,
          &Type::Integer,
        )?;
        Ok(Some(Type::Boolean))
      }
      BinaryOp::And | BinaryOp::Or => {
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &Type::Boolean,
        )?;
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op2,
          &Type::Boolean,
        )?;
        Ok(Some(Type::Boolean))
      }
    },
    ast::Argument::Unary(u) => match &u.node.op {
      UnaryOp::Neg | UnaryOp::Pos => {
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &u.node.op1,
          &Type::Integer,
        )?;
        Ok(Some(Type::Integer))
      }
      UnaryOp::Not => {
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &u.node.op1,
          &Type::Boolean,
        )?;
        Ok(Some(Type::Boolean))
      }
    },
    ast::Argument::Variable(_) => Ok(None),
    ast::Argument::Wildcard(w) => Err(CompileError::InvalidWildcard {
      loc: w.location.clone(),
    }),
  }
}

fn unify_two_args(
  node_types: &mut NodeTypeMap,
  to_unify_args: &mut ToUnifyArgs,
  rule_arg_map: &HashMap<usize, usize>,
  arg_1: &ast::Argument,
  arg_2: &ast::Argument,
) -> Result<(), CompileError> {
  match (arg_1, arg_2) {
    (ast::Argument::Constant(_), ast::Argument::Constant(_)) => {
      Err(CompileError::UnnecessaryConstantComparison)
    }
    (ast::Argument::Wildcard(w), _) | (_, ast::Argument::Wildcard(w)) => {
      Err(CompileError::InvalidWildcard {
        loc: w.location.clone(),
      })
    }
    (arg_1, arg_2) => {
      let ty_1 = type_of_arg(node_types, to_unify_args, rule_arg_map, arg_1)?;
      let ty_2 = type_of_arg(node_types, to_unify_args, rule_arg_map, arg_2)?;
      match (ty_1, ty_2) {
        (Some(ty_1), Some(ty_2)) => {
          if ty_1 == ty_2 {
            node_types.insert(arg_1.location().id, ty_1);
            node_types.insert(arg_2.location().id, ty_2);
            Ok(())
          } else {
            Err(CompileError::TypeUnificationError {
              loc_1: arg_1.location().clone(),
              loc_2: arg_2.location().clone(),
              ty_1: ty_1.clone(),
              ty_2: ty_2.clone(),
            })
          }
        }
        (Some(ty_1), None) => unify_arg_type(node_types, to_unify_args, rule_arg_map, arg_2, &ty_1),
        (None, Some(ty_2)) => unify_arg_type(node_types, to_unify_args, rule_arg_map, arg_1, &ty_2),
        (None, None) => match (arg_1, arg_2) {
          (ast::Argument::Variable(v1), ast::Argument::Variable(v2)) => {
            if v1.node.name == v2.node.name {
              Err(CompileError::UnnecessaryIdentityComparison)
            } else {
              let rule_id = rule_arg_map[&v1.location.id];
              to_unify_args.push((
                rule_id,
                ToUnifyArg::Variable(v1.location.id, v1.node.name.clone()),
                ToUnifyArg::Variable(v2.location.id, v2.node.name.clone()),
              ));
              Ok(())
            }
          }
          (ast::Argument::Variable(v), ast::Argument::Constant(c))
          | (ast::Argument::Constant(c), ast::Argument::Variable(v)) => {
            let rule_id = rule_arg_map[&v.location.id];
            to_unify_args.push((
              rule_id,
              ToUnifyArg::Variable(v.location.id, v.node.name.clone()),
              ToUnifyArg::Integer(c.location.id),
            ));
            Ok(())
          }
          _ => Err(CompileError::SomethingStrangeInTypeUnification),
        },
      }
    }
  }
}

fn unify_arg_type(
  node_types: &mut NodeTypeMap,
  to_unify_args: &mut ToUnifyArgs,
  rule_arg_map: &HashMap<usize, usize>,
  arg: &ast::Argument,
  arg_type: &Type,
) -> Result<(), CompileError> {
  match arg {
    ast::Argument::Constant(c) => match (&c.node, arg_type) {
      (ast::ConstantNode::Boolean(_), Type::Boolean) => {
        node_types.insert(c.location.id, Type::Boolean);
      }
      (ast::ConstantNode::Integer(_), Type::Symbol) => {
        node_types.insert(c.location.id, Type::Symbol);
      }
      (ast::ConstantNode::Integer(_), Type::Integer) => {
        node_types.insert(c.location.id, Type::Integer);
      }
      (ast::ConstantNode::String(_), Type::String) => {
        node_types.insert(c.location.id, Type::String);
      }
      _ => {
        return Err(CompileError::TypeMismatch {
          loc: c.location.clone(),
          ty: arg_type.clone(),
        });
      }
    },
    ast::Argument::Variable(v) => {
      node_types.insert(v.location.id, arg_type.clone());
    }
    ast::Argument::Wildcard(w) => {
      node_types.insert(w.location.id, arg_type.clone());
    }
    ast::Argument::Binary(b) => match (&b.node.op, arg_type) {
      (BinaryOp::Add, Type::Integer)
      | (BinaryOp::Sub, Type::Integer)
      | (BinaryOp::Mult, Type::Integer)
      | (BinaryOp::Div, Type::Integer)
      | (BinaryOp::Gt, Type::Boolean)
      | (BinaryOp::Gte, Type::Boolean)
      | (BinaryOp::Lt, Type::Boolean)
      | (BinaryOp::Lte, Type::Boolean) => {
        node_types.insert(b.location.id, arg_type.clone());
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &Type::Integer,
        )?;
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op2,
          &Type::Integer,
        )?;
      }
      (BinaryOp::Eq, Type::Boolean) | (BinaryOp::Ne, Type::Boolean) => {
        node_types.insert(b.location.id, arg_type.clone());
        unify_two_args(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &b.node.op2,
        )?;
      }
      (BinaryOp::And, Type::Boolean) | (BinaryOp::Or, Type::Boolean) => {
        node_types.insert(b.location.id, arg_type.clone());
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op1,
          &Type::Boolean,
        )?;
        unify_arg_type(
          node_types,
          to_unify_args,
          rule_arg_map,
          &b.node.op2,
          &Type::Boolean,
        )?;
      }
      _ => {
        return Err(CompileError::TypeMismatch {
          loc: b.location.clone(),
          ty: arg_type.clone(),
        })
      },
    },
    ast::Argument::Unary(u) => match (&u.node.op, arg_type) {
      (UnaryOp::Neg, Type::Integer) | (UnaryOp::Pos, Type::Integer) => {
        node_types.insert(u.location.id, arg_type.clone());
      }
      (UnaryOp::Not, Type::Boolean) => {
        node_types.insert(u.location.id, arg_type.clone());
      }
      _ => {
        return Err(CompileError::TypeMismatch {
          loc: u.location.clone(),
          ty: arg_type.clone(),
        })
      },
    },
  }
  Ok(())
}

impl NodeVisitor for TypeAssign {
  fn visit_decl(&mut self, decl: &ast::Decl) -> Result<(), CompileError> {
    if self.decls.contains_key(&decl.node.predicate) {
      Err(CompileError::DuplicatedDeclaration {
        dup: decl.location.clone(),
        rela_name: decl.node.predicate.clone(),
      })
    } else {
      self.decls.insert(
        decl.node.predicate.clone(),
        decl
          .node
          .arg_types
          .iter()
          .map(|t| t.node.clone())
          .collect::<Vec<_>>(),
      );
      Ok(())
    }
  }

  fn visit_rule(&mut self, rule: &ast::Rule) -> Result<(), CompileError> {
    struct Inner {
      arg_ids: Vec<usize>,
    }

    impl NodeVisitor for Inner {
      fn visit_arg(&mut self, arg: &ast::Argument) -> Result<(), CompileError> {
        self.arg_ids.push(arg.location().id);
        Ok(())
      }
    }

    let rule_id = rule.location.id;
    let mut inner = Inner { arg_ids: vec![] };
    visit_rule(&mut inner, rule)?;
    for arg_id in inner.arg_ids {
      self.rule_arg_map.insert(arg_id, rule_id);
    }
    Ok(())
  }

  fn visit_atom(&mut self, atom: &ast::Atom) -> Result<(), CompileError> {
    if self.decls.contains_key(&atom.node.predicate) {
      let arg_types = &self.decls[&atom.node.predicate];
      if arg_types.len() == atom.node.args.len() {
        for (arg, arg_type) in atom.node.args.iter().zip(arg_types.iter()) {
          unify_arg_type(
            &mut self.node_types,
            &mut self.to_unify_args,
            &self.rule_arg_map,
            arg,
            arg_type,
          )?;
        }
        return Ok(());
      } else {
        return Err(CompileError::IncorrectArity {
          loc: atom.location.clone(),
          rela_name: atom.node.predicate.clone(),
          found: atom.node.args.len(),
          expected: arg_types.len(),
        });
      }
    } else {
      return Err(CompileError::UnknownRelation {
        loc: atom.location.clone(),
        rela_name: atom.node.predicate.clone(),
      });
    }
  }

  fn visit_binary_constraint(&mut self, bin: &ast::BinaryConstraint) -> Result<(), CompileError> {
    match &bin.node.op {
      BinaryOp::Eq | BinaryOp::Ne => {
        unify_two_args(
          &mut self.node_types,
          &mut self.to_unify_args,
          &self.rule_arg_map,
          &bin.node.op1,
          &bin.node.op2,
        )?;
        Ok(())
      }
      BinaryOp::Gt | BinaryOp::Gte | BinaryOp::Lt | BinaryOp::Lte => {
        unify_arg_type(
          &mut self.node_types,
          &mut self.to_unify_args,
          &self.rule_arg_map,
          &bin.node.op1,
          &Type::Integer,
        )?;
        unify_arg_type(
          &mut self.node_types,
          &mut self.to_unify_args,
          &self.rule_arg_map,
          &bin.node.op2,
          &Type::Integer,
        )?;
        Ok(())
      }
      _ => Err(CompileError::ShouldNotHappen),
    }
  }
}

pub struct TypeUnification<'a> {
  node_types: &'a mut NodeTypeMap,
  to_unify_args: ToUnifyArgs,
}

impl<'a> TypeUnification<'a> {
  pub fn new(node_types: &'a mut NodeTypeMap, to_unify_args: ToUnifyArgs) -> Self {
    Self {
      node_types,
      to_unify_args,
    }
  }
}

impl<'a> NodeVisitor for TypeUnification<'a> {
  fn visit_rule(&mut self, rule: &ast::Rule) -> Result<(), CompileError> {
    struct VarFinder {
      vars: HashMap<String, Vec<usize>>,
    }
    impl NodeVisitor for VarFinder {
      fn visit_arg(&mut self, arg: &ast::Argument) -> Result<(), CompileError> {
        match arg {
          ast::Argument::Variable(v) => {
            self
              .vars
              .entry(v.node.name.clone())
              .or_insert(vec![])
              .push(v.location.id);
          }
          _ => {}
        }
        Ok(())
      }
    }

    // Find all the variables
    let mut var_finder = VarFinder {
      vars: HashMap::new(),
    };
    visit_rule(&mut var_finder, rule)?;

    // Unify the same variable
    let mut var_types = HashMap::new();
    for (var_name, instance_ids) in var_finder.vars {
      let mut first_type = None;
      for instance_id in &instance_ids {
        match self.node_types.get(&instance_id) {
          Some(instance_ty) => {
            first_type = Some(instance_ty.clone());
            break;
          }
          _ => {}
        }
      }
      if let Some(first_type) = first_type {
        var_types.insert(var_name, first_type.clone());
        for instance_id in &instance_ids {
          match self.node_types.get(&instance_id) {
            Some(instance_ty) => {
              if &first_type != instance_ty {
                return Err(CompileError::CannotUnifyType);
              }
            }
            None => {
              self
                .node_types
                .insert(instance_id.clone(), first_type.clone());
            }
          }
        }
      } else {
        return Err(CompileError::CannotInferType);
      }
    }

    // Unify cross variables
    for (rule_id, var_a, var_b) in &self.to_unify_args {
      if rule_id == &rule.location.id {
        match (var_a, var_b) {
          (ToUnifyArg::Variable(_, var_a), ToUnifyArg::Variable(_, var_b)) => {
            let ty_a = var_types.get(var_a);
            let ty_b = var_types.get(var_b);
            match (ty_a, ty_b) {
              (Some(ty_a), Some(ty_b)) if ty_a == ty_b => {
                // Good
              }
              _ => return Err(CompileError::CannotUnifyType),
            }
          }
          (ToUnifyArg::Variable(_, var), ToUnifyArg::Integer(int_id))
          | (ToUnifyArg::Integer(int_id), ToUnifyArg::Variable(_, var)) => {
            let var_ty = var_types.get(var).unwrap();
            match var_ty {
              Type::Integer => {
                self.node_types.insert(int_id.clone(), Type::Integer);
              }
              Type::Symbol => {
                self.node_types.insert(int_id.clone(), Type::Symbol);
              }
              _ => return Err(CompileError::CannotUnifyType),
            }
          }
          _ => return Err(CompileError::ShouldNotHappen),
        }
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConcreteBound {
  Bound(ast::ConstantNode),
  Free,
}

impl ConcreteBound {
  pub fn is_bound(&self) -> bool {
    match self {
      Self::Bound(_) => true,
      _ => false
    }
  }

  pub fn is_free(&self) -> bool {
    match self {
      Self::Free => true,
      _ => false
    }
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Demand {
  pub predicate: String,
  pub bounds: Vec<ConcreteBound>,
}

impl Demand {
  pub fn is_all_free(&self) -> bool {
    for bound in &self.bounds {
      if bound.is_bound() {
        return false
      }
    }
    true
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DemandConstants {
  pub constants: Vec<ast::ConstantNode>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Bound {
  Bound,
  Free,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DemandPattern {
  pub predicate: String,
  pub bounds: Vec<Bound>,
}

impl DemandPattern {
  pub fn demand_name(&self) -> String {
    format!("_dmd_{}_{}", self.predicate, self.bounds.iter().map(|b| {
      match b {
        Bound::Bound => "b",
        Bound::Free => "f",
      }
    }).collect::<Vec<_>>().join(""))
  }

  pub fn is_all_free(&self) -> bool {
    for bound in &self.bounds {
      match bound {
        Bound::Bound => return false,
        _ => {},
      }
    }
    true
  }
}

impl From<&Demand> for DemandPattern {
  fn from(demand: &Demand) -> Self {
    Self {
      predicate: demand.predicate.clone(),
      bounds: demand.bounds.iter().map(|bound| {
        match bound {
          ConcreteBound::Bound(_) => Bound::Bound,
          ConcreteBound::Free => Bound::Free,
        }
      }).collect::<Vec<_>>(),
    }
  }
}

impl From<&Demand> for DemandConstants {
  fn from(demand: &Demand) -> Self {
    Self {
      constants: demand.bounds.iter().filter_map(|bound| {
        match bound {
          ConcreteBound::Bound(c) => Some(c.clone()),
          ConcreteBound::Free => None,
        }
      }).collect::<Vec<_>>(),
    }
  }
}

pub type Demands = HashMap<DemandPattern, HashSet<DemandConstants>>;

pub struct DemandCollector {
  pub demands: Demands,
  pub to_explore: Vec<Demand>,
}

impl DemandCollector {
  pub fn new() -> Self {
    Self {
      demands: Demands::new(),
      to_explore: Vec::new(),
    }
  }

  pub fn has_to_explore(&self) -> bool {
    !self.to_explore.is_empty()
  }
}

impl NodeVisitor for DemandCollector {
  fn visit_query(&mut self, query: &ast::Query) -> Result<(), CompileError> {
    let atom = &query.node.atom;
    let bounds = atom.node.args.iter().map(|arg| {
      match arg {
        ast::Argument::Constant(c) => ConcreteBound::Bound(c.node.clone()),
        _ => ConcreteBound::Free,
      }
    }).collect::<Vec<_>>();
    let pattern = Demand {
      predicate: atom.node.predicate.clone(),
      bounds,
    };
    self.to_explore.push(pattern);
    Ok(())
  }
}

fn analyze_demand_patterns(
  demand_collector: &mut DemandCollector,
  prog: &ast::Program,
) -> Result<(), CompileError> {
  // First get the mapping from predicate to rule ids
  let mut pred_to_rule_ids = HashMap::<String, Vec<usize>>::new();
  for (i, rule) in prog.rules.iter().enumerate() {
    let pred = rule.node.head.node.predicate.clone();
    pred_to_rule_ids.entry(pred).or_default().push(i);
  }

  // Enter fix point iteration
  while demand_collector.has_to_explore() {
    // Pop one demand
    let demand = demand_collector.to_explore.pop().unwrap();

    // Generate demand pattern
    let demand_pattern = DemandPattern::from(&demand);
    let constants = DemandConstants::from(&demand);
    let demands = demand_collector.demands.entry(demand_pattern).or_default();
    let already_explored = !demands.is_empty(); // Check if explored already
    demands.insert(constants);

    // In case the demand is already explored, we can skip the propagation
    if already_explored {
      continue;
    }

    // Propagate the demands to other rules
    if let Some(rule_ids) = pred_to_rule_ids.get(&demand.predicate) {
      for rule_id in rule_ids {
        let rule = &prog.rules[*rule_id];

        // Infer the bounded variables
        let bounded_vars = demand.bounds.iter().zip(rule.node.head.node.args.iter()).filter_map(|(bound, arg)| {
          match (bound, arg) {
            (ConcreteBound::Bound(c), ast::Argument::Variable(v)) => {
              Some((v.node.name.clone(), c.clone()))
            }
            _ => None
          }
        }).collect::<HashMap<_, _>>();

        // Iterate through body atoms
        for literal in &rule.node.body {
          match &literal.node {
            ast::LiteralNode::Pos(a) | ast::LiteralNode::Neg(a) => {
              if a.node.predicate == demand.predicate {
                continue;
              }

              // Generate demand for the atom
              let demand = Demand {
                predicate: a.node.predicate.clone(),
                bounds: a.node.args.iter().map(|arg| {
                  match arg {
                    ast::Argument::Constant(c) => {
                      ConcreteBound::Bound(c.node.clone())
                    },
                    ast::Argument::Variable(v) => {
                      match bounded_vars.get(&v.node.name) {
                        Some(c) => ConcreteBound::Bound(c.clone()),
                        None => ConcreteBound::Free,
                      }
                    },
                    _ => {
                      // TODO:
                      ConcreteBound::Free
                    }
                  }
                }).collect::<Vec<_>>(),
              };

              // Only if the demand is NOT all free do we propagate the demand
              if !demand.is_all_free() {
                demand_collector.to_explore.push(demand);
              }
            }
            _ => {}
          }
        }
      }
    }
  }
  Ok(())
}

pub fn analyze(prog: &ast::Program, options: &CompileOptions) -> Result<AnalysisResult, CompileError> {
  // Run analysis first pass
  let mut first_pass = (
    TypeAssign::new(),
    IsProbabilisticAnalyzer::new(),
    DisjunctionOnSameRelationChecker::new(),
    DemandCollector::new(),
    UnboundedVariableAnalyzer,
    FactHasOnlyConstantAnalyzer,
    InvalidWildcardAnalyzer,
    NoExprInBodyAtomAnalyzer,
  );
  visit_program(&mut first_pass, prog)?;
  let type_assign = first_pass.0;
  let decls = type_assign.decls;
  let mut node_types = type_assign.node_types;
  let to_unify_args = type_assign.to_unify_args;
  let is_probabilistic = first_pass.1.is_probabilistic;
  let disj_rela_map = first_pass.2.disj_rela_map;
  let mut demand_collector = first_pass.3;

  // Run second pass to unify types
  let mut second_pass = (TypeUnification::new(&mut node_types, to_unify_args),);
  visit_program(&mut second_pass, prog)?;

  // Propagate demand patterns if enabled
  let demands = if options.demand_transform {
    analyze_demand_patterns(&mut demand_collector, prog)?;
    demand_collector.demands
  } else {
    Demands::default()
  };

  Ok(AnalysisResult {
    is_probabilistic,
    decls,
    node_types,
    disj_rela_map,
    demands,
  })
}
