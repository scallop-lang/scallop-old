use std::collections::*;

use super::*;

#[derive(Debug, Clone)]
pub struct DiffProbProofContext {
  pub id_counter: usize,
  pub disjunctions: Disjunctions,
  pub diff_prob_table: HashMap<usize, DualNumber>,
}

impl Default for DiffProbProofContext {
  fn default() -> Self {
    Self {
      id_counter: 0,
      disjunctions: Vec::new(),
      diff_prob_table: HashMap::new(),
    }
  }
}
