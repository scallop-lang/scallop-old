use std::collections::*;

use super::ast;
use super::ast_analysis::*;
use super::common;
use super::error::*;
use super::visitor::*;
use super::options::CompileOptions;

pub struct ConstIntegerToConstSymbol<'a> {
  node_types: &'a NodeTypeMap,
}

impl<'a> ConstIntegerToConstSymbol<'a> {
  fn new(node_types: &'a NodeTypeMap) -> Self {
    Self { node_types }
  }
}

impl<'a> NodeVisitorMut for ConstIntegerToConstSymbol<'a> {
  fn visit_constant(&mut self, c: &mut ast::Constant) -> Result<(), CompileError> {
    match (&c.node, &self.node_types[&c.location.id]) {
      (ast::ConstantNode::Integer(i), common::Type::Symbol) => {
        *c = ast::Constant {
          location: c.location.clone(),
          node: ast::ConstantNode::SymbolId((*i) as usize),
        };
        Ok(())
      }
      _ => Ok(()),
    }
  }
}

fn demand_transform(prog: &mut ast::Program, anal: &mut AnalysisResult) -> Result<(), CompileError> {
  let demands = &anal.demands;

  // Shorthand when there is no demand
  if demands.is_empty() {
    return Ok(());
  }

  // Transform rules and add new set of rules
  let mut rules_to_add = Vec::new();
  let mut rules_to_remove = BTreeSet::new();
  let mut declared_demand_variables = HashSet::new();
  for (i, rule) in prog.rules.iter().enumerate() {
    let pred = &rule.node.head.node.predicate;
    let mut has_all_free = false;
    let mut cannot_demand_transform = false;
    let mut rules_to_add_for_this_rule = Vec::new();

    // Iterate through all demands
    for (dp, cs) in demands.iter().filter(|(d, _)| &d.predicate == pred) {
      if dp.is_all_free() {
        has_all_free = true;
        break;
      }

      // STEP 0: Add a new demand variable to the program
      let dp_name = dp.demand_name();
      if !declared_demand_variables.contains(&dp_name) {
        let dp_types = anal.decls[pred].iter().enumerate().filter_map(|(i, ty)| {
          match dp.bounds[i] {
            super::ast_analysis::Bound::Bound => Some(ty.clone()),
            _ => None
          }
        }).collect::<Vec<_>>();
        let decl_dp_types = dp_types.iter().map(|ty| ast::Type::new(ty.clone())).collect::<Vec<_>>();
        let decl = ast::Decl::new((dp_name.clone(), decl_dp_types));
        prog.decls.push(decl);
        anal.decls.insert(dp_name.clone(), dp_types);
        declared_demand_variables.insert(dp_name.clone());

        // STEP 1: Add demand facts DF
        for c in cs {
          let atom = ast::Atom::new((
            dp_name.clone(),
            c.constants.iter().map(|c| {
              ast::Argument::Constant(ast::Constant::new(c.clone()))
            }).collect::<Vec<_>>(),
          ));
          prog.facts.push(ast::Fact::new((None, atom)));
        }
      }

      // STEP 2: Transform the existing rule R -> R'
      let mut rule_prime = rule.clone();
      let bounded_args = rule.node.head.node.args.iter().enumerate().filter_map(|(i, arg)| {
        match dp.bounds[i] {
          super::ast_analysis::Bound::Bound => {
            match arg {
              ast::Argument::Binary(_) | ast::Argument::Unary(_) => {
                cannot_demand_transform = true;
                None
              }
              _ => Some(arg.clone())
            }
          },
          _ => None,
        }
      }).collect::<Vec<_>>();
      if cannot_demand_transform {
        break;
      }
      let atom = ast::Atom::new((dp_name.clone(), bounded_args));
      rule_prime.node.body.insert(0, ast::Literal::new(ast::LiteralNode::Pos(atom)));
      rules_to_add_for_this_rule.push(rule_prime.clone());

      // STEP 3: Add demand rules DR
      for (i, literal) in rule_prime.node.body.iter().enumerate() {
        match &literal.node {
          ast::LiteralNode::Pos(a) if &a.node.predicate == pred => {
            let bounded_vars = a.node.args.iter().enumerate().filter_map(|(i, arg)| {
              match dp.bounds[i] {
                super::ast_analysis::Bound::Bound => {
                  Some(arg.clone())
                },
                _ => None
              }
            }).collect::<Vec<_>>();
            let head = ast::Atom::new((dp_name.clone(), bounded_vars));
            let body = rule_prime.node.body.iter().take(i).cloned().collect::<Vec<_>>();
            let demand_rule = ast::Rule::new((head, body));
            rules_to_add_for_this_rule.push(demand_rule);
          }
          _ => {}
        }
      }
    }

    // Mark the rule to be removed later as there is demand transformation
    if !has_all_free && !cannot_demand_transform {
      rules_to_remove.insert(i);
      rules_to_add.extend(rules_to_add_for_this_rule);
    }
  }

  // Remove the rules from back to front
  for rule_id in rules_to_remove.into_iter().rev() {
    prog.rules.remove(rule_id);
  }

  // Add the rules we cached
  prog.rules.extend(rules_to_add);

  Ok(())
}

pub fn transform(
  prog: &mut ast::Program,
  anal: &mut AnalysisResult,
  options: &CompileOptions,
) -> Result<(), CompileError> {
  // First pass: in AST transformation
  let mut transfs = (
    ConstIntegerToConstSymbol::new(&anal.node_types),
  );
  visit_program_mut(&mut transfs, prog)?;

  if options.demand_transform {
    // Second pass: demand transformation
    demand_transform(prog, anal)?;
  }

  // Return
  Ok(())
}
