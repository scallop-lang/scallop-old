use std::collections::*;

use super::disjunction::*;

#[derive(Clone, Debug)]
pub struct ProbProofContext {
  pub id_counter: usize,
  pub disjunctions: Disjunctions,
  pub prob_table: HashMap<usize, f32>,
}

impl Default for ProbProofContext {
  fn default() -> Self {
    Self {
      id_counter: 0,
      disjunctions: Vec::new(),
      prob_table: HashMap::new(),
    }
  }
}
