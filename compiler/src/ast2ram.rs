use std::collections::*;

use super::{ast, common::*, error::*, ram};

pub type SymbolIdMap = HashMap<String, usize>;

static MAX_TUPLE_SIZE : usize = 10;

pub fn identifier_map(ast: &ast::Program) -> SymbolIdMap {
  let mut identifier = 0;
  let mut map = SymbolIdMap::new();
  for rule in &ast.rules {
    for arg in &rule.node.head.node.args {
      match arg {
        ast::Argument::Constant(c) => match &c.node {
          ast::ConstantNode::Symbol(s) => {
            let id = identifier;
            identifier += 1;
            map.insert(s.clone(), id);
          }
          _ => {}
        },
        _ => {}
      }
    }
  }
  map
}

pub fn ast_arg_types_to_var_type(arg_types: &[ast::Type]) -> ram::VarType {
  match arg_types.len() {
    0 => ram::VarType::Empty,
    1 => ram::VarType::Base(arg_types[0].node.clone()),
    length => {
      if length > MAX_TUPLE_SIZE {
        let first_part = ast_arg_types_to_var_type(&arg_types[0..length / 2]);
        let second_part = ast_arg_types_to_var_type(&arg_types[length / 2..]);
        ram::VarType::Tuple(vec![first_part, second_part])
      } else {
        ram::VarType::Tuple(
          arg_types
            .iter()
            .cloned()
            .map(|e| ram::VarType::Base(e.node))
            .collect::<Vec<_>>(),
        )
      }
    }
  }
}

pub fn ast_const_to_ram_const(
  c: &ast::ConstantNode,
  id_map: &HashMap<String, usize>,
) -> ram::Constant {
  match c {
    ast::ConstantNode::Symbol(s) => {
      let symbol_id = id_map[s];
      ram::Constant::Symbol(symbol_id)
    }
    ast::ConstantNode::SymbolId(i) => ram::Constant::Symbol(i.clone()),
    ast::ConstantNode::String(s) => ram::Constant::String(s.clone()),
    ast::ConstantNode::Integer(i) => ram::Constant::Integer(i.clone()),
    ast::ConstantNode::Boolean(b) => ram::Constant::Boolean(b.clone()),
  }
}

pub fn type_of_ram_const(c: &ram::Constant) -> Type {
  match c {
    ram::Constant::Boolean(_) => Type::Boolean,
    ram::Constant::Integer(_) => Type::Integer,
    ram::Constant::String(_) => Type::String,
    ram::Constant::Symbol(_) => Type::Symbol,
  }
}

pub fn ast_fact_to_ram_fact(
  f: &ast::Fact,
  id_map: &SymbolIdMap,
) -> Result<ram::Fact, CompileError> {
  let mut args = vec![];
  for arg in &f.node.head.node.args {
    if let ast::Argument::Constant(c) = arg {
      args.push(ast_const_to_ram_const(&c.node, &id_map));
    }
  }
  Ok(ram::Fact {
    prob: f.node.prob,
    predicate: f.node.head.node.predicate.clone(),
    args: args,
  })
}

pub fn ast_to_ram_variables(ast: &ast::Program) -> Vec<ram::Variable> {
  let mut variables = vec![];
  for decl in &ast.decls {
    variables.push(ram::Variable {
      is_temporary: false,
      name: decl.node.predicate.clone(),
      arg_types: ast_arg_types_to_var_type(&decl.node.arg_types),
    });
  }
  variables
}

pub fn ast_to_ram_facts(
  ast: &ast::Program,
  id_map: &SymbolIdMap,
) -> Result<Vec<ram::Fact>, CompileError> {
  let mut facts = vec![];
  for fact in &ast.facts {
    let ram_fact = ast_fact_to_ram_fact(fact, &id_map)?;
    facts.push(ram_fact);
  }
  Ok(facts)
}

pub fn ast_to_ram_disjunctions(
  ast: &ast::Program,
  id_map: &SymbolIdMap,
) -> Result<Vec<ram::Disjunction>, CompileError> {
  let mut disjunctions = vec![];
  for disjunction in &ast.disjunctions {
    let mut ram_disj_facts = vec![];
    for fact in &disjunction.node.facts {
      let ram_fact = ast_fact_to_ram_fact(fact, &id_map)?;
      ram_disj_facts.push(ram_fact);
    }
    disjunctions.push(ram::Disjunction {
      id: disjunction.location.id,
      facts: ram_disj_facts,
    });
  }
  Ok(disjunctions)
}

pub fn ast_arg_to_ram_arg(
  arg: &ast::Argument,
  vars: &HashMap<String, Vec<usize>>,
  id_map: &SymbolIdMap,
) -> Result<ram::Argument, CompileError> {
  match arg {
    ast::Argument::Variable(v) => {
      let index = vars[&v.node.name].clone();
      Ok(ram::Argument::Element(index))
    }
    ast::Argument::Constant(c) => {
      let c = ast_const_to_ram_const(&c.node, id_map);
      Ok(ram::Argument::Constant(c))
    }
    ast::Argument::Binary(bin) => {
      let op1 = ast_arg_to_ram_arg(&bin.node.op1, vars, id_map)?;
      let op2 = ast_arg_to_ram_arg(&bin.node.op2, vars, id_map)?;
      Ok(ram::Argument::Binary(
        bin.node.op.clone(),
        Box::new(op1),
        Box::new(op2),
      ))
    }
    ast::Argument::Unary(una) => {
      let op1 = ast_arg_to_ram_arg(&una.node.op1, vars, id_map)?;
      Ok(ram::Argument::Unary(una.node.op.clone(), Box::new(op1)))
    }
    ast::Argument::Wildcard(w) => Err(CompileError::InvalidWildcard {
      loc: w.location.clone(),
    }),
  }
}

pub fn tmp_variable_name(tmp_counter: &mut usize) -> String {
  let counter = tmp_counter.clone();
  *tmp_counter += 1;
  format!("_tmp_{}", counter)
}

pub fn get_elem_type_of_index(var_type: &ram::VarType, indices: &[usize]) -> Type {
  match indices.first() {
    Some(i) => match var_type {
      ram::VarType::Tuple(elem_types) => get_elem_type_of_index(&elem_types[*i], &indices[1..]),
      _ => panic!("Bad"),
    },
    None => match var_type {
      ram::VarType::Base(b) => b.clone(),
      _ => panic!("Bad"),
    },
  }
}

pub fn type_of_ram_arg(arg: &ram::Argument, var_type: &ram::VarType) -> ram::VarType {
  match arg {
    ram::Argument::Constant(c) => ram::VarType::Base(type_of_ram_const(c)),
    ram::Argument::Element(e) => ram::VarType::Base(get_elem_type_of_index(var_type, &e)),
    ram::Argument::Tuple(t) => ram::VarType::Tuple(
      t.iter()
        .map(|e| type_of_ram_arg(e, var_type))
        .collect::<Vec<_>>(),
    ),
    _ => panic!("Not supported"),
  }
}

pub fn variable_type_of_flow(flow: &ram::Flow, vars: &Vec<ram::Variable>) -> ram::VarType {
  match flow {
    ram::Flow::ContainsChain(_, _, source) => variable_type_of_flow(source, vars),
    ram::Flow::Filter(v, _) => variable_type_of_flow(v, vars),
    ram::Flow::Find(v, _) => variable_type_of_flow(v, vars),
    ram::Flow::Intersect(a, _) => variable_type_of_flow(a, vars),
    ram::Flow::Join(a, b) => {
      let a_ty = variable_type_of_flow(a, vars);
      let b_ty = variable_type_of_flow(b, vars);
      match (a_ty, b_ty) {
        (ram::VarType::Tuple(a_elems), ram::VarType::Tuple(b_elems)) => {
          assert_eq!(a_elems.len(), 2);
          assert_eq!(b_elems.len(), 2);
          // assert_eq!(a_elems[0], b_elems[0]);
          let k = a_elems[0].clone();
          let t1 = a_elems[1].clone();
          let t2 = b_elems[1].clone();
          ram::VarType::Tuple(vec![k, t1, t2])
        }
        _ => panic!("Cannot join"),
      }
    }
    ram::Flow::Product(a, b) => {
      let a_ty = variable_type_of_flow(a, vars);
      let b_ty = variable_type_of_flow(b, vars);
      ram::VarType::Tuple(vec![a_ty, b_ty])
    }
    ram::Flow::Project(v, p) => {
      let v_ty = variable_type_of_flow(v, vars);
      type_of_ram_arg(p, &v_ty)
    }
    ram::Flow::Variable(name) => {
      // Unwrap because there must be such name
      let var = vars.iter().find(|v| &v.name == name).unwrap();
      var.arg_types.clone()
    }
  }
}

pub fn add_temporary_variable_from_flow(
  tmp_counter: &mut usize,
  vars: &mut Vec<ram::Variable>,
  flow: &ram::Flow,
) -> String {
  let name = tmp_variable_name(tmp_counter);
  let ty = variable_type_of_flow(flow, vars);
  let var = ram::Variable {
    is_temporary: true,
    name: name.clone(),
    arg_types: ty,
  };
  vars.push(var);
  name
}

pub fn find_variable<'a>(
  ram_vars: &'a Vec<ram::Variable>,
  name: &str,
) -> Option<&'a ram::Variable> {
  for ram_var in ram_vars {
    if ram_var.name == name {
      return Some(&ram_var);
    }
  }
  None
}

pub fn projected_intersect_var(
  flow: ram::Flow,
  vars: &HashMap<String, Vec<usize>>,
  arg: ram::Argument,
  ram_vars: &mut Vec<ram::Variable>,
  updates: &mut Vec<ram::Update>,
  tmp_counter: &mut usize,
) -> String {
  match &flow {
    ram::Flow::Variable(name) => {
      let ram_var = find_variable(ram_vars, name).unwrap();
      match (&ram_var.arg_types, &arg) {
        (ram::VarType::Base(_), ram::Argument::Element(e)) => {
          if e.is_empty() {
            return name.clone();
          }
        }
        (ram::VarType::Tuple(tys), ram::Argument::Tuple(t)) => {
          if t.len() == tys.len() {
            if t.iter().enumerate().all(|(i, arg_i)| match arg_i {
              ram::Argument::Element(indices) => indices == &vec![i],
              _ => false,
            }) {
              return name.clone();
            }
          }
        }
        _ => {}
      }
    }
    _ => {}
  };

  let flow = match arg {
    ram::Argument::Element(_) if vars.len() == 1 => flow,
    _ => ram::Flow::Project(Box::new(flow), arg),
  };

  let tmp_var_name = add_temporary_variable_from_flow(tmp_counter, ram_vars, &flow);
  updates.push(ram::Update {
    into_var: tmp_var_name.clone(),
    flow,
  });

  tmp_var_name
}

pub fn projected_join_var(
  flow: ram::Flow,
  vars: &HashMap<String, Vec<usize>>,
  key: ram::Argument,
  itsct: &HashSet<&String>,
  ram_vars: &mut Vec<ram::Variable>,
  updates: &mut Vec<ram::Update>,
  tmp_counter: &mut usize,
) -> (String, HashMap<String, Vec<usize>>) {
  let t_a_all = vars
    .iter()
    .filter_map(|(name, indices)| {
      if itsct.contains(name) {
        None
      } else {
        Some((name, ram::Argument::Element(indices.clone())))
      }
    })
    .collect::<Vec<_>>();
  let (t_a, t_a_vars) = if t_a_all.len() == 1 {
    let (name, arg) = t_a_all[0].clone();
    let vars = vec![(name.clone(), vec![])]
      .into_iter()
      .collect::<HashMap<_, _>>();
    (arg, vars)
  } else {
    let arg = ram::Argument::Tuple(
      t_a_all
        .iter()
        .map(|(_, arg)| arg)
        .cloned()
        .collect::<Vec<_>>(),
    );
    let vars = t_a_all
      .iter()
      .enumerate()
      .map(|(i, (name, _))| (name.clone().clone(), vec![i]))
      .collect::<HashMap<_, _>>();
    (arg, vars)
  };
  match (&flow, &key, &t_a) {
    (
      ram::Flow::Variable(v),
      ram::Argument::Element(key_elems),
      ram::Argument::Element(t_a_elems),
    ) => {
      if key_elems == &vec![0] && t_a_elems == &vec![1] {
        return (v.clone(), t_a_vars);
      }
    }
    _ => {}
  }
  let projected_a = ram::Flow::Project(Box::new(flow), ram::Argument::Tuple(vec![key, t_a]));
  let var_a = add_temporary_variable_from_flow(tmp_counter, ram_vars, &projected_a);
  updates.push(ram::Update {
    into_var: var_a.clone(),
    flow: projected_a,
  });
  (var_a, t_a_vars)
}

pub fn body_atom_to_flow_variable(
  atom: &ast::Atom,
  id_map: &SymbolIdMap,
) -> Result<(ram::Flow, HashMap<String, Vec<usize>>), CompileError> {
  // Build set of variables and equality constraints

  let num_args = atom.node.args.len();

  let mut variables: HashMap<String, Vec<usize>> = HashMap::new();
  let mut equality_constraints: Vec<(Vec<usize>, Vec<usize>)> = vec![];
  let mut filter_constants: Vec<(Vec<usize>, ast::ConstantNode)> = vec![];

  for (i, arg) in atom.node.args.iter().enumerate() {
    let indicies = if num_args == 1 { vec![] } else { vec![i] };
    match arg {
      ast::Argument::Variable(v) => {
        if variables.contains_key(&v.node.name) {
          equality_constraints.push((variables[&v.node.name].clone(), indicies));
        } else {
          variables.insert(v.node.name.clone(), indicies);
        }
      }
      ast::Argument::Wildcard(_) => {}
      ast::Argument::Constant(c) => {
        filter_constants.push((indicies, c.node.clone()));
      }
      _ => return Err(CompileError::ShouldNotHappen),
    }
  }

  // Add constants constraint
  let source_var_flow = ram::Flow::Variable(atom.node.predicate.clone());
  let const_filtered_flow = if filter_constants.len() > 0 {
    let const_filters = filter_constants
      .iter()
      .map(|(indices, constant)| {
        ram::Argument::Binary(
          BinaryOp::Eq,
          Box::new(ram::Argument::Element(indices.clone())),
          Box::new(ram::Argument::Constant(ast_const_to_ram_const(
            constant, id_map,
          ))),
        )
      })
      .collect::<Vec<_>>();
    let filter_arg = const_filters
      .iter()
      .skip(1)
      .fold(const_filters[0].clone(), |agg, curr| {
        ram::Argument::Binary(BinaryOp::And, Box::new(agg), Box::new(curr.clone()))
      });
    ram::Flow::Filter(Box::new(source_var_flow), filter_arg)
  } else {
    source_var_flow
  };

  // Check if we need to create a flow
  let equality_filtered_flow = if equality_constraints.len() == 0 {
    // In this case, we don't need to do anything but directly use the
    // variable
    const_filtered_flow
  } else {
    // In this case, some of the arguments are omitted. We formulate this
    // as a potential filter and projection
    if equality_constraints.is_empty() {
      const_filtered_flow
    } else {
      // Create equals from equality constraints
      let equals = equality_constraints
        .iter()
        .map(|(i, j)| {
          ram::Argument::Binary(
            BinaryOp::Eq,
            Box::new(ram::Argument::Element(i.clone())),
            Box::new(ram::Argument::Element(j.clone())),
          )
        })
        .collect::<Vec<_>>();

      // Combine them with And operation
      let filter_arg = equals.iter().skip(1).fold(equals[0].clone(), |agg, curr| {
        ram::Argument::Binary(BinaryOp::And, Box::new(agg), Box::new(curr.clone()))
      });

      ram::Flow::Filter(Box::new(const_filtered_flow), filter_arg)
    }
  };

  if variables.len() == atom.node.args.len() && atom.node.args.len() <= MAX_TUPLE_SIZE {
    return Ok((equality_filtered_flow, variables));
  } else {
    use std::iter::FromIterator;
    let (project_arg, vars) = if variables.len() == 1 {
      let arg = ram::Argument::Element(variables.iter().next().unwrap().1.clone());
      let vars = HashMap::from_iter(variables.iter().map(|(name, _)| (name.clone(), vec![])));
      (arg, vars)
    } else {
      if atom.node.args.len() <= MAX_TUPLE_SIZE {
        let mut args = variables.iter().collect::<Vec<_>>();
        args.sort_by(|(_, i1), (_, i2)| i1.partial_cmp(i2).unwrap());
        let arg = ram::Argument::Tuple(
          args
            .iter()
            .map(|(_, indices)| ram::Argument::Element((*indices).clone()))
            .collect::<Vec<_>>(),
        );
        let vars = HashMap::from_iter(
          args
            .iter()
            .enumerate()
            .map(|(i, (name, _))| ((*name).clone(), vec![i])),
        );
        (arg, vars)
      } else {
        return Err(CompileError::NotImplemented);
      }
    };
    let flow = ram::Flow::Project(Box::new(equality_filtered_flow), project_arg);
    Ok((flow, vars))
  }
}

pub fn ast_atom_is_fact(atom: &ast::Atom) -> bool {
  for arg in atom.node.args.iter() {
    match arg {
      ast::Argument::Constant(_) => {}
      _ => return false,
    }
  }
  return true;
}

type VarLocMap = HashMap<String, Vec<usize>>;

pub fn product_flow(
  f1: (ram::Flow, VarLocMap),
  f2: (ram::Flow, VarLocMap),
) -> Result<(ram::Flow, VarLocMap), CompileError> {
  let (agg_flow, agg_vars) = f1;
  let (curr_flow, curr_vars) = f2;

  let num_agg_vars = agg_vars.len();
  let num_curr_vars = curr_vars.len();

  let result_flow = ram::Flow::Product(Box::new(agg_flow), Box::new(curr_flow));
  let result_vars = agg_vars
    .into_iter()
    .map(|(name, indices)| {
      (
        name,
        if num_agg_vars == 1 {
          vec![0]
        } else {
          std::iter::once(0)
            .chain(indices.iter().cloned())
            .collect::<Vec<_>>()
        },
      )
    })
    .chain(curr_vars.into_iter().map(|(name, indices)| {
      (
        name,
        if num_curr_vars == 1 {
          vec![1]
        } else {
          std::iter::once(1)
            .chain(indices.iter().cloned())
            .collect::<Vec<_>>()
        },
      )
    }))
    .collect::<HashMap<_, _>>();
  Ok((result_flow, result_vars))
}

pub fn combine_flows(
  f1: (ram::Flow, VarLocMap),
  f2: (ram::Flow, VarLocMap),
  vars: &mut Vec<ram::Variable>,
  updates: &mut Vec<ram::Update>,
  tmp_counter: &mut usize,
) -> Result<(ram::Flow, VarLocMap), CompileError> {
  let (agg_flow, agg_vars) = f1;
  let (curr_flow, curr_vars) = f2;

  // Sets check
  let agg_vars_set = agg_vars.iter().map(|(var, _)| var).collect::<HashSet<_>>();
  let curr_vars_set = curr_vars.iter().map(|(var, _)| var).collect::<HashSet<_>>();
  let itsct = &agg_vars_set & &curr_vars_set;
  if itsct.is_empty() {
    // Product
    product_flow((agg_flow, agg_vars), (curr_flow, curr_vars))
  } else {
    // First get the variables and arguments inside itsct
    let (k_a, k_b, k_vars) = match itsct.len() {
      0 => panic!("Not possible"),
      1 => {
        let name = *itsct.iter().next().unwrap();
        let k_a = ram::Argument::Element(agg_vars[name].clone());
        let k_b = ram::Argument::Element(curr_vars[name].clone());
        let vars = vec![(name.clone(), vec![])]
          .into_iter()
          .collect::<HashMap<_, _>>();
        (k_a, k_b, vars)
      }
      _ => {
        let mut itsct_vec = itsct.iter().cloned().collect::<Vec<_>>();
        itsct_vec.sort_by(|a, b| {
          (&agg_vars[*a], &curr_vars[*a])
            .partial_cmp(&(&agg_vars[*b], &curr_vars[*b]))
            .unwrap()
        });

        let k_a = ram::Argument::Tuple(
          itsct_vec
            .iter()
            .map(|name| {
              let indices = agg_vars[*name].clone();
              ram::Argument::Element(indices)
            })
            .collect::<Vec<_>>(),
        );
        let k_b = ram::Argument::Tuple(
          itsct_vec
            .iter()
            .map(|name| {
              let indices = curr_vars[*name].clone();
              ram::Argument::Element(indices)
            })
            .collect::<Vec<_>>(),
        );
        let vars = itsct_vec
          .into_iter()
          .enumerate()
          .map(|(i, name)| (name.clone(), vec![i]))
          .collect::<HashMap<_, _>>();
        (k_a, k_b, vars)
      }
    };

    // Check the relationship between the two
    let is_agg = itsct == agg_vars_set;
    let is_curr = itsct == curr_vars_set;

    if is_agg && is_curr {
      let var_a = projected_intersect_var(agg_flow, &agg_vars, k_a, vars, updates, tmp_counter);
      let var_b = projected_intersect_var(curr_flow, &curr_vars, k_b, vars, updates, tmp_counter);
      let intersect_flow = ram::Flow::Intersect(
        Box::new(ram::Flow::Variable(var_a)),
        Box::new(ram::Flow::Variable(var_b)),
      );
      Ok((intersect_flow, k_vars))
    } else if is_agg || is_curr {
      let tmp_a_var_name = if is_agg {
        projected_intersect_var(agg_flow.clone(), &agg_vars, k_a.clone(), vars, updates, tmp_counter)
      } else {
        projected_intersect_var(curr_flow.clone(), &curr_vars, k_b.clone(), vars, updates, tmp_counter)
      };

      let (var_b, t_b_vars) = if is_agg {
        projected_join_var(curr_flow, &curr_vars, k_b, &itsct, vars, updates, tmp_counter)
      } else {
        projected_join_var(agg_flow, &agg_vars, k_a, &itsct, vars, updates, tmp_counter)
      };
      let flow_a = ram::Flow::Project(
        Box::new(ram::Flow::Variable(tmp_a_var_name)),
        ram::Argument::Tuple(vec![
          ram::Argument::Element(vec![]),
          ram::Argument::Tuple(vec![]),
        ]),
      );
      let joined_flow = ram::Flow::Join(Box::new(flow_a), Box::new(ram::Flow::Variable(var_b)));
      let joint_vars = k_vars
        .into_iter()
        .map(|(name, indices)| (name, std::iter::once(0).chain(indices).collect::<Vec<_>>()))
        .chain(
          t_b_vars
            .into_iter()
            .map(|(name, indices)| (name, std::iter::once(2).chain(indices).collect::<Vec<_>>())),
        )
        .collect::<HashMap<_, _>>();
      Ok((joined_flow, joint_vars))
    } else {
      let (var_a, t_a_vars) =
        projected_join_var(agg_flow, &agg_vars, k_a, &itsct, vars, updates, tmp_counter);
      let (var_b, t_b_vars) = projected_join_var(
        curr_flow,
        &curr_vars,
        k_b,
        &itsct,
        vars,
        updates,
        tmp_counter,
      );

      let joint_flow = ram::Flow::Join(
        Box::new(ram::Flow::Variable(var_a.clone())),
        Box::new(ram::Flow::Variable(var_b.clone())),
      );

      let joint_vars = k_vars
        .into_iter()
        .map(|(name, indices)| (name, std::iter::once(0).chain(indices).collect::<Vec<_>>()))
        .chain(
          t_a_vars
            .into_iter()
            .map(|(name, indices)| (name, std::iter::once(1).chain(indices).collect::<Vec<_>>())),
        )
        .chain(
          t_b_vars
            .into_iter()
            .map(|(name, indices)| (name, std::iter::once(2).chain(indices).collect::<Vec<_>>())),
        )
        .collect::<HashMap<_, _>>();

      Ok((joint_flow, joint_vars))
    }
  }
}

pub fn create_project_arg(args: &[ram::Argument]) -> ram::Argument {
  match args.len() {
    0 => ram::Argument::Tuple(vec![]),
    1 => args[0].clone(),
    length => {
      if length > MAX_TUPLE_SIZE {
        let first_part = create_project_arg(&args[0..length / 2]);
        let second_part = create_project_arg(&args[length / 2..]);
        ram::Argument::Tuple(vec![first_part, second_part])
      } else {
        ram::Argument::Tuple(args.iter().cloned().collect::<Vec<_>>())
      }
    }
  }
}

pub fn ast_rule_to_ram_updates(
  rule: &ast::Rule,
  vars: &mut Vec<ram::Variable>,
  facts: &mut Vec<ram::Fact>,
  id_map: &SymbolIdMap,
  tmp_counter: &mut usize,
) -> Result<Vec<ram::Update>, CompileError> {
  let mut updates = vec![];

  let mut pos_flows = vec![];
  let mut pos_facts = vec![];
  // let mut neg_flows = vec![];
  // let mut neg_facts = vec![];

  let mut constraints = vec![];
  for body_literal in &rule.node.body {
    match &body_literal.node {
      ast::LiteralNode::Pos(atom) => {
        if ast_atom_is_fact(atom) {
          pos_facts.push(atom);
        } else {
          pos_flows.push(body_atom_to_flow_variable(atom, id_map)?);
        }
      }
      ast::LiteralNode::Neg(_) => {
        return Err(CompileError::NegationNotImplemented);
        // if ast_atom_is_fact(atom) {
        //   neg_facts.push(atom);
        // } else {
        //   neg_flows.push(body_atom_to_flow_variable(atom, id_map)?);
        // }
      }
      ast::LiteralNode::Constraint(cons) => constraints.push(cons.clone()),
    }
  }

  let (joint_pos_flow, joint_pos_variables) = if pos_flows.is_empty() {
    // Head has to be an atom
    let head_args = rule
      .node
      .head
      .node
      .args
      .iter()
      .map(|arg| match arg {
        ast::Argument::Constant(c) => Ok(ast_const_to_ram_const(&c.node, id_map)),
        _ => Err(CompileError::ShouldNotHappen),
      })
      .collect::<Result<Vec<_>, _>>()?;
    let tmp_name = tmp_variable_name(tmp_counter);
    let head_var = find_variable(vars, &rule.node.head.node.predicate).unwrap();
    let var = ram::Variable {
      is_temporary: true,
      name: tmp_name.clone(),
      arg_types: head_var.arg_types.clone(),
    };
    vars.push(var);
    let fact = ram::Fact {
      prob: None,
      predicate: tmp_name.clone(),
      args: head_args,
    };
    facts.push(fact);
    let flow = ram::Flow::Variable(tmp_name);
    let variables = HashMap::new();
    (flow, variables)
  } else {
    let first_pos_flow = pos_flows[0].clone();
    pos_flows
      .into_iter()
      .skip(1)
      .try_fold(first_pos_flow, |agg, curr| {
        combine_flows(agg, curr, vars, &mut updates, tmp_counter)
      })?
  };

  let joint_pos_flow_with_facts = pos_facts
    .iter()
    .fold(joint_pos_flow, |agg, curr_fact_atom| {
      let var_to_find = ram::Flow::Variable(curr_fact_atom.node.predicate.clone());
      let key = curr_fact_atom
        .node
        .args
        .iter()
        .map(|arg| match arg {
          ast::Argument::Constant(c) => {
            ast_const_to_ram_const(&c.node, id_map)
          }
          _ => panic!("Should not happen"),
        })
        .collect::<Vec<_>>();
      ram::Flow::ContainsChain(Box::new(var_to_find), key, Box::new(agg))
    });

  // Add constraints
  let pos_flow_with_constraints = if constraints.is_empty() {
    joint_pos_flow_with_facts
  } else {
    let constraint_args = constraints
      .iter()
      .map(|constraint| match constraint {
        ast::Constraint::Unary(_) => Err(CompileError::NotImplemented),
        ast::Constraint::Binary(b) => Ok(ram::Argument::Binary(
          b.node.op.clone(),
          Box::new(ast_arg_to_ram_arg(
            &b.node.op1,
            &joint_pos_variables,
            id_map,
          )?),
          Box::new(ast_arg_to_ram_arg(
            &b.node.op2,
            &joint_pos_variables,
            id_map,
          )?),
        )),
      })
      .collect::<Result<Vec<_>, CompileError>>()?;
    let agg =
      constraint_args
        .iter()
        .skip(1)
        .fold(constraint_args[0].clone(), |agg, curr_constraint| {
          ram::Argument::Binary(
            BinaryOp::And,
            Box::new(agg),
            Box::new(curr_constraint.clone()),
          )
        });
    ram::Flow::Filter(Box::new(joint_pos_flow_with_facts), agg)
  };

  let var_name = rule.node.head.node.predicate.clone();
  let head_arity = rule.node.head.node.args.len();
  let head_variables = rule
    .node
    .head
    .node
    .args
    .iter()
    .enumerate()
    .filter_map(|(i, arg)| match arg {
      ast::Argument::Variable(v) => Some((v.node.name.clone(), vec![i])),
      _ => None,
    })
    .collect::<HashMap<_, _>>();

  // Check head variables and pos_flow variables
  let pos_flow = if joint_pos_variables == head_variables && joint_pos_variables.len() == head_arity
  {
    pos_flow_with_constraints
  } else {
    let args = rule
      .node
      .head
      .node
      .args
      .iter()
      .map(|arg| ast_arg_to_ram_arg(arg, &joint_pos_variables, id_map))
      .collect::<Result<Vec<_>, CompileError>>()?;

    let project_arg = create_project_arg(&args);

    ram::Flow::Project(Box::new(pos_flow_with_constraints), project_arg)
  };

  // Add the last flow
  updates.push(ram::Update {
    flow: pos_flow,
    into_var: var_name,
  });

  Ok(updates)
}

pub fn ast2ram(ast: &ast::Program) -> Result<ram::Program, CompileError> {
  // Precomputed analysis
  let id_map = identifier_map(ast);
  let mut tmp_var_id = 0;

  // Compute program components
  let mut variables = ast_to_ram_variables(ast);
  let mut facts = ast_to_ram_facts(ast, &id_map)?;
  let disjunctions = ast_to_ram_disjunctions(ast, &id_map)?;

  // Populate updates from rules
  let mut updates = vec![];
  for rule in &ast.rules {
    let rule_updates =
      ast_rule_to_ram_updates(&rule, &mut variables, &mut facts, &id_map, &mut tmp_var_id)?;
    updates.extend(rule_updates);
  }

  // Generate program
  Ok(ram::Program {
    variables,
    facts,
    disjunctions,
    updates,
  })
}
