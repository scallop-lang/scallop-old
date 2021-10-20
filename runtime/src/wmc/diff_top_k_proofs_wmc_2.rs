use std::collections::*;
use sdd::{bf, BooleanFormula, SDDBuilder, SDDBuilderConfig};

use crate::semiring::*;
use crate::tags::*;
use super::{WeightedModelCounter, DiffProbabilitySemiring2};

#[derive(Debug, Clone)]
pub struct DiffTopKProbProofsWMC2<const K: usize>;

impl<const K: usize> WeightedModelCounter for DiffTopKProbProofsWMC2<K> {
  type Tag = DiffTopKProbProofs<K>;

  type Output = (f64, HashMap<usize, f64>);

  fn wmc(
    &self,
    ctx: &<Self::Tag as Semiring>::Context,
    tag: &Self::Tag,
  ) -> Self::Output {
    // Shortcut for false tag
    if tag.proofs.is_empty() {
      return (0.0, HashMap::new());
    }

    // Convert the proof to a boolean formula
    let form = top_k_prob_proofs_to_boolean_formula(tag);

    // Construct an SDD from the boolean formula
    let vars = form.collect_vars();
    let config = SDDBuilderConfig::with_formula(&form);
    let sdd = SDDBuilder::with_config(config).build(&form);
    let prob_semiring = DiffProbabilitySemiring2;

    let mut index = 0;
    let mut var_id_to_sparse_id_map = BTreeMap::new();
    let mut sparse_id_to_var_id_map = BTreeMap::new();
    for var_id in &vars {
      if !var_id_to_sparse_id_map.contains_key(var_id) {
        let sparse_id = index;
        index += 1;
        var_id_to_sparse_id_map.insert(var_id.clone(), sparse_id);
        sparse_id_to_var_id_map.insert(sparse_id, var_id.clone());
      }
    }
    let num_sparse_indices = var_id_to_sparse_id_map.len();

    let var_assign = vars
      .iter()
      .map(|var_id| {
        let source_tensor = &ctx.diff_prob_table[var_id];
        let prob = source_tensor.real();
        let sparse_id = var_id_to_sparse_id_map[var_id];
        let sparse_dn = DualNumber2::new(prob, sparse_id, num_sparse_indices);
        (var_id.clone(), sparse_dn)
      })
      .collect::<HashMap<usize, DualNumber2>>();

    let result_sparse_vec = sdd.eval_t(&var_assign, &prob_semiring);

    let y_pred = result_sparse_vec.prob();

    let var_id_to_deriv_map = (0..num_sparse_indices).map(|sparse_id| {
      let var_id = sparse_id_to_var_id_map[&sparse_id];
      let deriv = result_sparse_vec.ith_grad(&sparse_id);
      (var_id, deriv)
    }).collect::<HashMap<_, _>>();
    (y_pred, var_id_to_deriv_map)
  }
}

fn top_k_prob_proof_to_boolean_formula(proof: &DiffTopKProbProof) -> BooleanFormula {
  // Shortcut for empty proof: it is an always true proof
  if proof.facts.is_empty() {
    return BooleanFormula::True;
  }

  // Otherwise, iterate through each fact to construct the proof
  let mut iter = proof.facts.iter();
  let mut acc = bf(iter.next().unwrap().clone());
  while let Some(var_id) = iter.next() {
    acc = acc & bf(var_id.clone());
  }
  acc
}

fn top_k_prob_proofs_to_boolean_formula<const K: usize>(
  proofs: &DiffTopKProbProofs<K>,
) -> BooleanFormula {
  // First get the CNF formula
  let mut iter = proofs.proofs.iter();
  let mut acc = top_k_prob_proof_to_boolean_formula(iter.next().unwrap());
  while let Some(proof) = iter.next() {
    acc = acc | top_k_prob_proof_to_boolean_formula(proof);
  }
  acc
}
