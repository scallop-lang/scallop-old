use std::collections::*;

use scallop_compiler::{ast, ast2ram, common, error::CompileError, parser, ram};

use super::*;
use crate::interpreter::DynTuple;
use crate::utils::IdAllocator;
use crate::*;

#[derive(Clone, Debug)]
pub enum VariableKind {
  Static,
  Dynamic,
}

#[derive(Clone, Debug)]
pub enum DynCompileError {
  CompileError(CompileError),
}

#[derive(Clone, Debug, Default)]
pub struct CompilerContext {
  pub variables: HashMap<String, (VariableKind, TupleType)>,
  pub rules: HashMap<usize, ast::Rule>,
  pub tmp_var_id_allocator: IdAllocator,
}

impl CompilerContext {
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      rules: HashMap::new(),
      tmp_var_id_allocator: IdAllocator::new(),
    }
  }

  pub fn add_variable(&mut self, name: &str, var_kind: VariableKind, var_type: TupleType) {
    self
      .variables
      .insert(name.to_string(), (var_kind, var_type));
  }

  pub fn remove_variable(&mut self, name: &str) {
    self.variables.remove(name);
  }

  pub fn compile_rule_from_str(&mut self, rule_str: &str) -> Result<RuleToAdd, DynCompileError> {
    let rule_ast = parser::parse_rule(rule_str).map_err(|e| DynCompileError::CompileError(e))?;
    self.compile_rule_from_ast(rule_ast)
  }

  fn analyze_rule_ast(&self, ast: &ast::Rule) -> Result<(), DynCompileError> {
    use scallop_compiler::{ast_analysis::*, visitor::*};

    // Generate declarations
    let decls = self.variables.iter().filter_map(|(name, (_, tup_type))| {
      match tup_type {
        TupleType::Tuple(tys) => {
          let tys = tys.iter().map(|ty| {
            match ty {
              TupleType::Integer => Some(common::Type::Integer),
              TupleType::Boolean => Some(common::Type::Boolean),
              TupleType::String => Some(common::Type::String),
              TupleType::Symbol => Some(common::Type::Symbol),
              _ => None
            }
          }).collect::<Option<Vec<_>>>()?;
          Some((name.clone(), tys))
        }
        _ => None
      }
    }).collect::<HashMap<_, _>>();

    // Get type assign context
    let mut type_assign = TypeAssign::new();
    type_assign.decls = decls;

    // First pass
    let mut first_pass = (
      type_assign,
      UnboundedVariableAnalyzer,
      InvalidWildcardAnalyzer,
      NoExprInBodyAtomAnalyzer,
    );
    visit_rule(&mut first_pass, ast).map_err(|e| DynCompileError::CompileError(e))?;

    // Second pass
    let type_assign = first_pass.0;
    let mut node_types = type_assign.node_types;
    let to_unify_args = type_assign.to_unify_args;
    let mut second_pass = TypeUnification::new(&mut node_types, to_unify_args);
    visit_rule(&mut second_pass, ast).map_err(|e| DynCompileError::CompileError(e))?;

    // Success
    Ok(())
  }

  fn ram_type_to_tuple_type(&self, ram_type: ram::VarType) -> TupleType {
    match ram_type {
      ram::VarType::Empty => TupleType::Tuple(vec![]),
      ram::VarType::Base(base_type) => {
        match base_type {
          common::Type::Boolean => TupleType::Boolean,
          common::Type::Integer => TupleType::Integer,
          common::Type::String => TupleType::String,
          common::Type::Symbol => TupleType::Symbol,
        }
      },
      ram::VarType::Tuple(fields) => {
        TupleType::Tuple(fields.into_iter().map(|field_type| {
          self.ram_type_to_tuple_type(field_type)
        }).collect::<Vec<_>>())
      }
    }
  }

  fn ram_variable_to_dyn_variable(&self, tmp_var: ram::Variable) -> (String, TupleType) {
    (tmp_var.name, self.ram_type_to_tuple_type(tmp_var.arg_types))
  }

  fn ram_const_to_dyn_tuple(&self, c: &ram::Constant) -> DynTuple {
    match c {
      ram::Constant::Boolean(b) => DynTuple::Boolean(b.clone()),
      ram::Constant::Integer(i) => DynTuple::Integer(i.clone()),
      ram::Constant::String(s) => DynTuple::String(std::sync::Arc::new(s.clone())),
      ram::Constant::Symbol(s) => DynTuple::Symbol(s.clone()),
    }
  }

  fn ram_consts_to_dyn_tuple(&self, args: &Vec<ram::Constant>) -> DynTuple {
    DynTuple::Tuple(args.iter().map(|c| {
      self.ram_const_to_dyn_tuple(c)
    }).collect::<Vec<_>>())
  }

  fn ram_fact_to_dyn_fact(&self, fact: ram::Fact) -> (String, DynTuple) {
    (fact.predicate, self.ram_consts_to_dyn_tuple(&fact.args))
  }

  fn ram_const_to_dyn_const(&self, ram_const: &ram::Constant) -> interpreter::Constant {
    match ram_const {
      ram::Constant::Boolean(b) => interpreter::Constant::Boolean(b.clone()),
      ram::Constant::Integer(i) => interpreter::Constant::Integer(i.clone()),
      ram::Constant::Symbol(s) => interpreter::Constant::Symbol(s.clone()),
      ram::Constant::String(s) => interpreter::Constant::String(std::sync::Arc::new(s.clone())),
    }
  }

  fn ram_arg_to_dyn_exp(&self, arg: &ram::Argument) -> interpreter::Expression {
    match arg {
      ram::Argument::Binary(bop, a1, a2) => {
        let op = match bop {
          common::BinaryOp::Add => interpreter::BinaryOp::Add,
          common::BinaryOp::Sub => interpreter::BinaryOp::Sub,
          common::BinaryOp::Mult => interpreter::BinaryOp::Mul,
          common::BinaryOp::Div => interpreter::BinaryOp::Div,
          common::BinaryOp::And => interpreter::BinaryOp::And,
          common::BinaryOp::Or => interpreter::BinaryOp::Or,
          common::BinaryOp::Eq => interpreter::BinaryOp::Eq,
          common::BinaryOp::Ne => interpreter::BinaryOp::Ne,
          common::BinaryOp::Gt => interpreter::BinaryOp::Gt,
          common::BinaryOp::Gte => interpreter::BinaryOp::Gte,
          common::BinaryOp::Lt => interpreter::BinaryOp::Lt,
          common::BinaryOp::Lte => interpreter::BinaryOp::Lte,
        };
        interpreter::Expression::Binary(interpreter::Binary {
          op: op,
          lhs: Box::new(self.ram_arg_to_dyn_exp(a1)),
          rhs: Box::new(self.ram_arg_to_dyn_exp(a2)),
        })
      },
      ram::Argument::Unary(uop, a) => {
        let op = match uop {
          common::UnaryOp::Neg => interpreter::UnaryOp::Neg,
          common::UnaryOp::Not => interpreter::UnaryOp::Not,
          common::UnaryOp::Pos => interpreter::UnaryOp::Pos,
        };
        interpreter::Expression::Unary(interpreter::Unary {
          op: op,
          op0: Box::new(self.ram_arg_to_dyn_exp(a)),
        })
      },
      ram::Argument::Element(acc) => {
        let indices = acc.iter().map(|i| *i as u8).collect::<Vec<_>>();
        interpreter::Expression::Access(TupleAccessor::from_indices(&indices))
      },
      ram::Argument::Constant(c) => {
        let c = self.ram_const_to_dyn_const(c);
        interpreter::Expression::Constant(c)
      },
      ram::Argument::Tuple(ts) => {
        interpreter::Expression::Tuple(ts.iter().map(|t| {
          self.ram_arg_to_dyn_exp(t)
        }).collect::<Vec<_>>())
      },
    }
  }

  fn ram_flow_to_dyn_flow(&self, ram_flow: &ram::Flow) -> interpreter::Flow {
    match ram_flow {
      ram::Flow::Product(f1, f2) => interpreter::Flow::Product(
        Box::new(self.ram_flow_to_dyn_flow(f1)),
        Box::new(self.ram_flow_to_dyn_flow(f2)),
      ),
      ram::Flow::Intersect(f1, f2) => interpreter::Flow::Intersect(
        Box::new(self.ram_flow_to_dyn_flow(f1)),
        Box::new(self.ram_flow_to_dyn_flow(f2)),
      ),
      ram::Flow::Join(f1, f2) => interpreter::Flow::Join(
        Box::new(self.ram_flow_to_dyn_flow(f1)),
        Box::new(self.ram_flow_to_dyn_flow(f2)),
      ),
      ram::Flow::Filter(f, a) => interpreter::Flow::Filter(
        Box::new(self.ram_flow_to_dyn_flow(f)),
        self.ram_arg_to_dyn_exp(a),
      ),
      ram::Flow::Project(f, a) => interpreter::Flow::Project(
        Box::new(self.ram_flow_to_dyn_flow(f)),
        self.ram_arg_to_dyn_exp(a),
      ),
      ram::Flow::Find(f, c) => interpreter::Flow::Find(
        Box::new(self.ram_flow_to_dyn_flow(f)),
        self.ram_const_to_dyn_tuple(c),
      ),
      ram::Flow::ContainsChain(s, cs, f) => interpreter::Flow::ContainsChain(
        Box::new(self.ram_flow_to_dyn_flow(s)),
        self.ram_consts_to_dyn_tuple(cs),
        Box::new(self.ram_flow_to_dyn_flow(f)),
      ),
      ram::Flow::Variable(name) => {
        match self.variables.get(name) {
          Some((vk, _)) => match vk {
            VariableKind::Dynamic => interpreter::Flow::DynamicVariable(name.clone()),
            VariableKind::Static => interpreter::Flow::StaticVariable(name.clone()),
          },
          None => {
            interpreter::Flow::DynamicVariable(name.clone())
          },
        }
      },
    }
  }

  fn ram_update_to_dyn_update(&self, ram_update: &ram::Update) -> interpreter::Update {
    interpreter::Update {
      target: ram_update.into_var.clone(),
      flow: self.ram_flow_to_dyn_flow(&ram_update.flow),
    }
  }

  fn tuple_type_to_var_type(&self, tup_type: &TupleType) -> ram::VarType {
    match tup_type {
      TupleType::Boolean => ram::VarType::Base(common::Type::Boolean),
      TupleType::Integer => ram::VarType::Base(common::Type::Integer),
      TupleType::Symbol => ram::VarType::Base(common::Type::Symbol),
      TupleType::String => ram::VarType::Base(common::Type::String),
      TupleType::Tuple(ts) => {
        if ts.is_empty() {
          ram::VarType::Empty
        } else {
          ram::VarType::Tuple(ts.iter().map(|t| self.tuple_type_to_var_type(t)).collect::<Vec<_>>())
        }
      }
    }
  }

  pub fn compile_rule_from_ast(&mut self, ast: ast::Rule) -> Result<RuleToAdd, DynCompileError> {
    // First do analysis on the ast to make sure it is well formed
    self.analyze_rule_ast(&ast)?;

    // Then we setup the environment for compilation
    let mut vars = self.variables.iter().map(|(name, (_, tup_type))| {
      ram::Variable {
        is_temporary: false,
        arg_types: self.tuple_type_to_var_type(tup_type),
        name: name.clone(),
      }
    }).collect::<Vec<_>>();
    let num_existing_vars = vars.len();
    let mut facts = Vec::new();
    let id_map = ast2ram::SymbolIdMap::new();

    // Then we compile
    let ram_updates = ast2ram::ast_rule_to_ram_updates(
      &ast,
      &mut vars,
      &mut facts,
      &id_map,
      &mut self.tmp_var_id_allocator.curr_id,
    )
    .map_err(|e| DynCompileError::CompileError(e))?;

    // Then we turn ram items to dyn items
    let tmp_vars_to_add = vars
      .into_iter()
      .skip(num_existing_vars)
      .map(|tmp_var| self.ram_variable_to_dyn_variable(tmp_var))
      .collect::<Vec<_>>();
    let facts_to_add = facts
      .into_iter()
      .map(|fact| self.ram_fact_to_dyn_fact(fact))
      .collect::<Vec<_>>();
    let updates_to_add = ram_updates
      .into_iter()
      .map(|ram_update| self.ram_update_to_dyn_update(&ram_update))
      .collect::<Vec<_>>();

    // We successfully compiled the ast into a rule
    Ok(RuleToAdd {
      tmp_vars_to_add,
      facts_to_add,
      updates_to_add,
    })
  }
}
