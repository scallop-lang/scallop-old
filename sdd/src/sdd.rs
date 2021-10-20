use std::collections::*;

use super::*;

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct SDDElement {
  prime: SDDNodeIndex,
  sub: SDDNodeIndex,
}

impl std::fmt::Debug for SDDElement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "Elem {{ p: {:?}, s: {:?} }}",
      self.prime, self.sub
    ))
  }
}

#[derive(Clone)]
pub enum SDDLiteral {
  PosVar { var_id: usize },
  NegVar { var_id: usize },
  True,
  False,
}

impl std::fmt::Debug for SDDLiteral {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::PosVar { var_id } => f.write_fmt(format_args!("Pos({})", var_id)),
      Self::NegVar { var_id } => f.write_fmt(format_args!("Neg({})", var_id)),
      Self::True => f.write_str("True"),
      Self::False => f.write_str("False"),
    }
  }
}

#[derive(Clone)]
pub enum SDDNode {
  Or { children: Vec<SDDElement> },
  Literal { literal: SDDLiteral },
}

impl std::fmt::Debug for SDDNode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Literal { literal } => f.write_fmt(format_args!("{:?}", literal)),
      Self::Or { children } => f.write_fmt(format_args!("Or {{ {:?} }}", children)),
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SDDNodeIndex(usize);

impl std::fmt::Debug for SDDNodeIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("SDD({})", self.0))
  }
}

#[derive(Clone)]
pub struct SDDNodes {
  nodes: HashMap<usize, SDDNode>,
  max_id: usize,
  remove_count: usize,
  shrink_when_remove_count: usize,
}

impl SDDNodes {
  pub fn new() -> Self {
    Self {
      nodes: HashMap::new(),
      max_id: 0,
      remove_count: 0,
      shrink_when_remove_count: 32,
    }
  }

  pub fn add_node(&mut self, node: SDDNode) -> SDDNodeIndex {
    self.nodes.insert(self.max_id, node);
    let id = SDDNodeIndex(self.max_id);
    self.max_id += 1;
    id
  }

  pub fn remove_node(&mut self, id: SDDNodeIndex) {
    self.nodes.remove(&id.0);
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn iter(&self) -> SDDNodesIter {
    SDDNodesIter {
      indices: self
        .nodes
        .iter()
        .map(|(id, _)| SDDNodeIndex(id.clone()))
        .collect::<Vec<_>>()
        .into_iter(),
    }
  }
}

pub struct SDDNodesIter {
  indices: std::vec::IntoIter<SDDNodeIndex>,
}

impl Iterator for SDDNodesIter {
  type Item = SDDNodeIndex;

  fn next(&mut self) -> Option<Self::Item> {
    self.indices.next()
  }
}

impl std::ops::Index<SDDNodeIndex> for SDDNodes {
  type Output = SDDNode;

  fn index(&self, id: SDDNodeIndex) -> &Self::Output {
    &self.nodes[&id.0]
  }
}

impl std::fmt::Debug for SDDNodes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "SDDNodes {{ num_nodes: {}, nodes: {:?} }}",
      self.nodes.len(),
      self.nodes
    ))
  }
}

#[derive(Clone, Debug)]
pub struct SDD {
  sdd_nodes: SDDNodes,
  roots: Vec<SDDNodeIndex>,
}

impl SDD {
  pub fn eval(&self, var_assign: &HashMap<usize, bool>) -> bool {
    self.eval_node(self.roots[0], var_assign)
  }

  pub fn eval_i<T: IntoIterator<Item = (usize, bool)>>(&self, var_assign: T) -> bool {
    self.eval_node(self.roots[0], &var_assign.into_iter().collect())
  }

  fn eval_node(&self, node_id: SDDNodeIndex, var_assign: &HashMap<usize, bool>) -> bool {
    match &self.sdd_nodes[node_id] {
      SDDNode::Or { children } => {
        for child in children {
          // First evaluate the prime
          let result = self.eval_node(child.prime, var_assign);

          // If prime holds, return the evaluated value of the sub
          if result {
            return self.eval_node(child.sub, var_assign);
          }
        }
        panic!("Mutual exclusion violated")
      }
      SDDNode::Literal { literal } => match literal {
        SDDLiteral::PosVar { var_id } => var_assign[var_id],
        SDDLiteral::NegVar { var_id } => !var_assign[var_id],
        SDDLiteral::True => true,
        SDDLiteral::False => false,
      },
    }
  }

  pub fn eval_t<T: Semiring>(
    &self,
    var_assign: &HashMap<usize, <T as Semiring>::Element>,
    semiring: &T,
  ) -> <T as Semiring>::Element {
    self.eval_node_t(self.roots[0], var_assign, semiring)
  }

  fn eval_node_t<T: Semiring>(
    &self,
    node_id: SDDNodeIndex,
    var_assign: &HashMap<usize, <T as Semiring>::Element>,
    semiring: &T,
  ) -> <T as Semiring>::Element {
    match &self.sdd_nodes[node_id] {
      SDDNode::Or { children } => {
        children
          .iter()
          .fold(semiring.zero(), |acc, SDDElement { prime, sub }| {
            let prime_res = self.eval_node_t(prime.clone(), var_assign, semiring);
            let sub_res = self.eval_node_t(sub.clone(), var_assign, semiring);
            semiring.add(acc, semiring.mult(prime_res, sub_res))
          })
      }
      SDDNode::Literal { literal } => match literal {
        SDDLiteral::PosVar { var_id } => var_assign[var_id].clone(),
        SDDLiteral::NegVar { var_id } => semiring.negate(var_assign[var_id].clone()),
        SDDLiteral::True => semiring.one(),
        SDDLiteral::False => semiring.zero(),
      },
    }
  }

  pub fn dot(&self) -> String {
    fn literal_label(literal: &SDDLiteral) -> String {
      match literal {
        SDDLiteral::True => format!("⊤"),
        SDDLiteral::False => format!("⊥"),
        SDDLiteral::PosVar { var_id } => format!("V{}", var_id),
        SDDLiteral::NegVar { var_id } => format!("¬V{}", var_id),
      }
    }

    fn node_identifier(node_id: SDDNodeIndex) -> String {
      format!("N{}", node_id.0)
    }

    fn literal_label_or_node_identifier(nodes: &SDDNodes, node_id: SDDNodeIndex) -> String {
      match &nodes[node_id] {
        SDDNode::Or { .. } => node_identifier(node_id),
        SDDNode::Literal { literal } => literal_label(literal),
      }
    }

    fn traverse(
      nodes: &SDDNodes,
      curr_node: SDDNodeIndex,
      node_strs: &mut Vec<String>,
      edge_strs: &mut Vec<String>,
      elem_id: &mut usize,
    ) {
      let curr_label = node_identifier(curr_node);
      match &nodes[curr_node.clone()] {
        SDDNode::Or { children } => {
          node_strs.push(format!("{} [label=\"OR\", shape=circle];", curr_label));
          for SDDElement { prime, sub } in children {
            // Get element label
            let elem_label = format!("E{}", elem_id);
            *elem_id += 1;

            // Get child label
            let prime_label = literal_label_or_node_identifier(nodes, prime.clone());
            let sub_label = literal_label_or_node_identifier(nodes, sub.clone());

            // Add nodes
            node_strs.push(format!(
              "{} [label=\"<prime>{} | <sub>{}\"];",
              elem_label, prime_label, sub_label
            ));

            // Add Or to Elem edge
            edge_strs.push(format!("{} -> {};", curr_label, elem_label));

            // Add Elem to Child edge and continue traverse
            match &nodes[prime.clone()] {
              SDDNode::Or { .. } => {
                edge_strs.push(format!("{}:prime -> {};", elem_label, prime_label));
                traverse(nodes, prime.clone(), node_strs, edge_strs, elem_id);
              }
              _ => {}
            }
            match &nodes[sub.clone()] {
              SDDNode::Or { .. } => {
                edge_strs.push(format!("{}:sub -> {};", elem_label, sub_label));
                traverse(nodes, sub.clone(), node_strs, edge_strs, elem_id);
              }
              _ => {}
            }
          }
        }
        SDDNode::Literal { literal } => {
          let node_str = format!("{} [label=\"{}\"]", curr_label, literal_label(literal));
          node_strs.push(node_str);
        }
      }
    }

    let mut node_strs = vec![];
    let mut edge_strs = vec![];
    let mut elem_id = 0;

    for root in &self.roots {
      traverse(
        &self.sdd_nodes,
        root.clone(),
        &mut node_strs,
        &mut edge_strs,
        &mut elem_id,
      );
    }

    format!(
      "digraph sdd {{ node [shape=record margin=0.03 width=0 height=0]; {} {} }}",
      node_strs.join(" "),
      edge_strs.join(" ")
    )
  }

  pub fn save_dot(&self, file_name: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create(file_name)?;
    file.write_all(self.dot().as_bytes())?;
    Ok(())
  }
}

pub struct SDDBuilderConfig {
  vtree: VTree,
  garbage_collect: bool,
}

impl SDDBuilderConfig {
  pub fn new(vars: Vec<usize>, vtree_type: VTreeType, garbage_collect: bool) -> Self {
    let vtree = VTree::new_with_type(vars, vtree_type);
    Self {
      vtree,
      garbage_collect,
    }
  }

  pub fn with_formula(form: &BooleanFormula) -> Self {
    let vars = form.collect_vars();
    let vtree = VTree::new_with_type(vars, VTreeType::default());
    Self {
      vtree,
      garbage_collect: true,
    }
  }

  pub fn disable_garbage_collect(mut self) -> Self {
    self.garbage_collect = false;
    self
  }

  pub fn enable_garbage_collect(mut self) -> Self {
    self.garbage_collect = true;
    self
  }
}

pub struct SDDBuilder {
  config: SDDBuilderConfig,

  // Core
  sdd_nodes: SDDNodes,
  roots: Vec<SDDNodeIndex>,

  // Helper caches
  false_node: SDDNodeIndex,
  true_node: SDDNodeIndex,
  pos_var_nodes: HashMap<usize, SDDNodeIndex>,
  neg_var_nodes: HashMap<usize, SDDNodeIndex>,
  negation_map: HashMap<SDDNodeIndex, SDDNodeIndex>,
  sdd_node_to_vtree_node_map: HashMap<SDDNodeIndex, VTreeNodeIndex>,
  apply_cache: HashMap<(SDDNodeIndex, SDDNodeIndex, ApplyOp), SDDNodeIndex>,

  // Builder states
  apply_depth: usize,

  // Statistics
  apply_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ApplyOp {
  Conjoin,
  Disjoin,
}

impl SDDBuilder {
  pub fn with_config(config: SDDBuilderConfig) -> Self {
    // Generate new graph
    let mut sdd_nodes = SDDNodes::new();

    // False and True Nodes
    let false_node = sdd_nodes.add_node(SDDNode::Literal {
      literal: SDDLiteral::False,
    });
    let true_node = sdd_nodes.add_node(SDDNode::Literal {
      literal: SDDLiteral::True,
    });

    // Positive and Negative variable Nodes
    let pos_var_nodes = config
      .vtree
      .vars
      .iter()
      .map(|var_id| {
        (
          var_id.clone(),
          sdd_nodes.add_node(SDDNode::Literal {
            literal: SDDLiteral::PosVar {
              var_id: var_id.clone(),
            },
          }),
        )
      })
      .collect::<HashMap<_, _>>();
    let neg_var_nodes = config
      .vtree
      .vars
      .iter()
      .map(|var_id| {
        (
          var_id.clone(),
          sdd_nodes.add_node(SDDNode::Literal {
            literal: SDDLiteral::NegVar {
              var_id: var_id.clone(),
            },
          }),
        )
      })
      .collect::<HashMap<_, _>>();

    // Negation map:
    // - pos variables are negation of neg variables
    // - neg variables are negation of pos variables
    // - true is negation of false
    // - false is negation of true
    let negation_map = pos_var_nodes
      .iter()
      .map(|(var_id, pos_node_id)| (pos_node_id.clone(), neg_var_nodes[var_id]))
      .chain(
        neg_var_nodes
          .iter()
          .map(|(var_id, neg_node_id)| (neg_node_id.clone(), pos_var_nodes[var_id])),
      )
      .chain(vec![(false_node, true_node), (true_node, false_node)])
      .collect::<HashMap<_, _>>();

    // Mapping from SDD node to VTree node
    // - All the Pos/Neg var nodes are ampped to their VTree leaf nodes
    let sdd_node_to_vtree_node_map = pos_var_nodes
      .iter()
      .map(|(var_id, pos_node_id)| (pos_node_id.clone(), config.vtree.var_to_node_id_map[var_id]))
      .chain(neg_var_nodes.iter().map(|(var_id, neg_node_id)| {
        (neg_node_id.clone(), config.vtree.var_to_node_id_map[var_id])
      }))
      .collect::<HashMap<_, _>>();
    let apply_cache = HashMap::new();

    // Roots; initialized to empty
    let roots = Vec::new();

    // Construct the builder
    Self {
      config,
      sdd_nodes,
      roots,

      // Helper nodes
      false_node,
      true_node,
      pos_var_nodes,
      neg_var_nodes,
      negation_map,
      sdd_node_to_vtree_node_map,
      apply_cache,

      // States
      apply_depth: 0,

      // Statistics
      apply_count: 0,
    }
  }

  pub fn build(mut self, formula: &BooleanFormula) -> SDD {
    let root = self.build_sdd(formula);
    self.roots.push(root);

    // Do garbage collection if presented
    if self.config.garbage_collect {
      self.garbage_collect();
    }

    // Create an SDD
    SDD {
      sdd_nodes: self.sdd_nodes,
      roots: self.roots,
    }
  }

  pub fn add_formula(&mut self, formula: &BooleanFormula) -> usize {
    let num_roots = self.roots.len();
    let new_root = self.build_sdd(formula);
    self.roots.push(new_root);
    num_roots
  }

  pub fn build_arena(mut self) -> SDD {
    if self.config.garbage_collect {
      self.garbage_collect();
    }

    SDD {
      sdd_nodes: self.sdd_nodes,
      roots: self.roots,
    }
  }

  fn mark_visited(sdd_nodes: &SDDNodes, node: SDDNodeIndex, visited: &mut HashSet<SDDNodeIndex>) {
    visited.insert(node);
    match &sdd_nodes[node] {
      SDDNode::Literal { .. } => {}
      SDDNode::Or { children } => {
        for SDDElement { prime, sub } in children {
          Self::mark_visited(sdd_nodes, prime.clone(), visited);
          Self::mark_visited(sdd_nodes, sub.clone(), visited);
        }
      }
    }
  }

  fn remove_not_visited(sdd_nodes: &mut SDDNodes, visited: &HashSet<SDDNodeIndex>) {
    for node_id in sdd_nodes.iter() {
      if !visited.contains(&node_id) {
        sdd_nodes.remove_node(node_id);
      }
    }
  }

  pub fn garbage_collect(&mut self) {
    let mut visited = HashSet::new();
    if self.roots.len() > 0 {
      for root in &self.roots {
        Self::mark_visited(&self.sdd_nodes, root.clone(), &mut visited);
      }
      Self::remove_not_visited(&mut self.sdd_nodes, &visited);
    }
  }

  pub fn build_sdd(&mut self, formula: &BooleanFormula) -> SDDNodeIndex {
    match formula {
      BooleanFormula::True => self.true_node,
      BooleanFormula::False => self.false_node,
      BooleanFormula::Pos { var_id } => self.pos_var_nodes[var_id],
      BooleanFormula::Neg { var_id } => self.neg_var_nodes[var_id],
      BooleanFormula::Not { form } => {
        let form_id = self.build_sdd(form);
        self.negate_node(form_id)
      }
      BooleanFormula::And { left, right } => {
        let left_id = self.build_sdd(left);
        let right_id = self.build_sdd(right);
        self.apply(left_id, right_id, ApplyOp::Conjoin)
      }
      BooleanFormula::Or { left, right } => {
        let left_id = self.build_sdd(left);
        let right_id = self.build_sdd(right);
        self.apply(left_id, right_id, ApplyOp::Disjoin)
      }
    }
  }

  fn negation_of(&mut self, node: SDDNodeIndex) -> Option<SDDNodeIndex> {
    self.negation_map.get(&node).map(SDDNodeIndex::clone)
  }

  fn zero(&self, op: ApplyOp) -> SDDNodeIndex {
    match op {
      ApplyOp::Conjoin => self.false_node,
      ApplyOp::Disjoin => self.true_node,
    }
  }

  #[allow(dead_code)]
  fn one(&self, op: ApplyOp) -> SDDNodeIndex {
    match op {
      ApplyOp::Conjoin => self.true_node,
      ApplyOp::Disjoin => self.false_node,
    }
  }

  fn is_zero(&self, node: SDDNodeIndex, op: ApplyOp) -> bool {
    match op {
      ApplyOp::Conjoin => node == self.false_node,
      ApplyOp::Disjoin => node == self.true_node,
    }
  }

  fn is_false(&self, node: SDDNodeIndex) -> bool {
    self.false_node == node
  }

  #[allow(dead_code)]
  fn is_true(&self, node: SDDNodeIndex) -> bool {
    self.true_node == node
  }

  fn is_one(&self, node: SDDNodeIndex, op: ApplyOp) -> bool {
    match op {
      ApplyOp::Conjoin => node == self.true_node,
      ApplyOp::Disjoin => node == self.false_node,
    }
  }

  fn vtree_node(&self, sdd_node: SDDNodeIndex) -> VTreeNodeIndex {
    self.sdd_node_to_vtree_node_map[&sdd_node]
  }

  fn add_or_node(&mut self, children: Vec<SDDElement>, vtree_node: VTreeNodeIndex) -> SDDNodeIndex {
    // Apply shortcuts
    if children.len() == 2 {
      if Some(children[0].prime) == self.negation_of(children[1].prime) {
        if children[0].sub == self.false_node && children[1].sub == self.true_node {
          return children[1].prime;
        } else if children[0].sub == self.true_node && children[1].sub == self.false_node {
          return children[0].prime;
        } else if children[0].sub == children[1].sub {
          return children[0].sub;
        }
      }
    }

    // Create node
    let node = SDDNode::Or { children };
    let node_id = self.sdd_nodes.add_node(node);

    // Update vtree link
    self.sdd_node_to_vtree_node_map.insert(node_id, vtree_node);

    // Return node id
    return node_id;
  }

  fn cache_apply_result(
    &mut self,
    lhs: SDDNodeIndex,
    rhs: SDDNodeIndex,
    op: ApplyOp,
    result_node: SDDNodeIndex,
  ) {
    self.apply_cache.insert((lhs, rhs, op), result_node);
  }

  fn lookup_apply_cache(
    &self,
    lhs: SDDNodeIndex,
    rhs: SDDNodeIndex,
    op: ApplyOp,
  ) -> Option<SDDNodeIndex> {
    self
      .apply_cache
      .get(&(lhs, rhs, op))
      .map(SDDNodeIndex::clone)
  }

  fn negate_node(&mut self, n: SDDNodeIndex) -> SDDNodeIndex {
    // Check if there is
    if let Some(neg) = self.negation_of(n) {
      return neg;
    }

    // Prime-sub stack
    let mut neg_children = Vec::new();
    if let SDDNode::Or { children } = self.sdd_nodes[n].clone() {
      for SDDElement { prime, sub } in children {
        let sub_neg = self.negate_node(sub.clone());
        neg_children.push(SDDElement {
          prime,
          sub: sub_neg,
        });
      }
    }

    // Insert negated node
    let neg = self.add_or_node(neg_children, self.sdd_node_to_vtree_node_map[&n]);

    // Update negation
    self.negation_map.insert(n, neg);
    self.negation_map.insert(neg, n);

    neg
  }

  fn apply_equal(
    &mut self,
    n1: SDDNodeIndex,
    n2: SDDNodeIndex,
    op: ApplyOp,
    lca: VTreeNodeIndex,
  ) -> SDDNodeIndex {
    let mut new_children = Vec::new();

    // Get the children; they should both have children
    let n1_sdd = self.sdd_nodes[n1].clone();
    let n2_sdd = self.sdd_nodes[n2].clone();
    let (c1, c2) = match (n1_sdd, n2_sdd) {
      (SDDNode::Or { children: c1 }, SDDNode::Or { children: c2 }) => (c1, c2),
      _ => panic!("Should not happen"),
    };

    // Do cartesian product
    for SDDElement { prime: p1, sub: s1 } in &c1 {
      for SDDElement { prime: p2, sub: s2 } in &c2 {
        // Generate prime
        let new_prime = self.apply(p1.clone(), p2.clone(), ApplyOp::Conjoin);

        // Shortcut for prime
        if self.is_false(new_prime) {
          continue;
        }

        // Generate sub
        let new_sub = self.apply(s1.clone(), s2.clone(), op);
        new_children.push(SDDElement {
          prime: new_prime,
          sub: new_sub,
        });
      }
    }

    // Add the node
    self.add_or_node(new_children, lca)
  }

  fn apply_left(
    &mut self,
    n1: SDDNodeIndex,
    n2: SDDNodeIndex,
    op: ApplyOp,
    lca: VTreeNodeIndex,
  ) -> SDDNodeIndex {
    let n1_neg = self.negate_node(n1);
    let n = match op {
      ApplyOp::Conjoin => n1,
      ApplyOp::Disjoin => n1_neg,
    };

    // Create the set of new elements
    let mut new_children = Vec::new();
    new_children.push(SDDElement {
      prime: self.negation_of(n).unwrap(), // Unwrap as we just created negated node of n1
      sub: self.zero(op),
    });

    // n2 has to be an OR node as n1 vtree is a subtree of n2 vtree
    match self.sdd_nodes[n2].clone() {
      SDDNode::Or { children } => {
        for SDDElement { prime, sub } in children {
          let new_prime = self.apply(prime, n, ApplyOp::Conjoin);
          if !self.is_false(new_prime) {
            new_children.push(SDDElement {
              prime: new_prime,
              sub: sub,
            });
          }
        }
      }
      _ => panic!("Should not happen"),
    }

    // Construct new or node
    self.add_or_node(new_children, lca)
  }

  fn apply_right(
    &mut self,
    n1: SDDNodeIndex,
    n2: SDDNodeIndex,
    op: ApplyOp,
    lca: VTreeNodeIndex,
  ) -> SDDNodeIndex {
    // n1 has to be an OR node as n2 tree is a subtree of n1 tree
    match self.sdd_nodes[n1].clone() {
      SDDNode::Or { children } => {
        let mut new_children = Vec::new();
        for SDDElement { prime, sub } in children {
          let new_sub = self.apply(sub.clone(), n2, op);
          new_children.push(SDDElement {
            prime: prime.clone(),
            sub: new_sub,
          });
        }

        // Construct new or node
        self.add_or_node(new_children, lca)
      }
      _ => panic!("Should not happen"),
    }
  }

  fn apply_disjoint(
    &mut self,
    n1: SDDNodeIndex,
    n2: SDDNodeIndex,
    op: ApplyOp,
    lca: VTreeNodeIndex,
  ) -> SDDNodeIndex {
    let n1_neg = self.negate_node(n1);
    let n1_sub = self.apply(n2, self.true_node, op);
    let n1_neg_sub = self.apply(n2, self.false_node, op);

    // Construct the new OR node
    let e1 = SDDElement {
      prime: n1,
      sub: n1_sub,
    };
    let e2 = SDDElement {
      prime: n1_neg,
      sub: n1_neg_sub,
    };

    // Add new node
    self.add_or_node(vec![e1, e2], lca)
  }

  fn apply(&mut self, lhs: SDDNodeIndex, rhs: SDDNodeIndex, op: ApplyOp) -> SDDNodeIndex {
    // If they are the same node, return the node itself
    if lhs == rhs {
      return lhs;
    }

    // If A == ~B, simplify A & B to false or A | B to true
    if Some(lhs) == self.negation_of(rhs) {
      return self.zero(op);
    }

    // If A or B is false, then A & B is false
    // If A or B is true, then A | B is true
    if self.is_zero(lhs, op) || self.is_zero(rhs, op) {
      return self.zero(op);
    }

    // If A is true, then A & B is B
    // If A is false, then A | B is B
    if self.is_one(lhs, op) {
      return rhs;
    }

    // The same applies for B
    if self.is_one(rhs, op) {
      return lhs;
    }

    // Check if there is cached computation result
    if let Some(cached_node_id) = self.lookup_apply_cache(lhs, rhs, op) {
      return cached_node_id;
    }

    // Increment depth
    self.apply_depth += 1;

    // Statistics
    self.apply_count += 1;

    // Swap the two nodes if their respective position is out of order
    let lhs_v = self.vtree_node(lhs);
    let rhs_v = self.vtree_node(rhs);
    let lhs_vpos = self.config.vtree.position(lhs_v);
    let rhs_vpos = self.config.vtree.position(rhs_v);
    let ((lhs, lhs_v), (rhs, rhs_v)) = if lhs_vpos > rhs_vpos {
      ((rhs, rhs_v), (lhs, lhs_v))
    } else {
      ((lhs, lhs_v), (rhs, rhs_v))
    };

    // Get the lowest common ancestor
    let (anc_type, lca) = self.config.vtree.lowest_common_ancestor(lhs_v, rhs_v);

    // Real apply
    let result_node = match anc_type {
      AncestorType::Equal => self.apply_equal(lhs, rhs, op, lca),
      AncestorType::Left => self.apply_left(lhs, rhs, op, lca),
      AncestorType::Right => self.apply_right(lhs, rhs, op, lca),
      AncestorType::Disjoint => self.apply_disjoint(lhs, rhs, op, lca),
    };

    // Cache
    self.cache_apply_result(lhs, rhs, op, result_node);

    // Decrement depth
    self.apply_depth -= 1;

    // Return the node
    result_node
  }
}
