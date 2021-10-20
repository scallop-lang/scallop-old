use std::collections::*;
use tch::{nn, Tensor};
use sdd::{bf, BooleanFormula, SDDBuilder, SDDBuilderConfig};

use crate::semiring::*;
use crate::tags::*;
use super::{WeightedModelCounter, DifferentiableProbabilitySemiring};

#[derive(Debug, Clone)]
pub struct DiffTopKProbProofsWMC<'a, const K: usize> {
  pub vs: &'a nn::Path<'a>,
  pub shape: Vec<i64>,
}

impl<'a, const K: usize> WeightedModelCounter for DiffTopKProbProofsWMC<'a, K> {
  type Tag = DiffTopKProbProofs<K>;

  type Output = (f64, HashMap<usize, f64>);

  fn wmc(
    &self,
    ctx: &<Self::Tag as Semiring>::Context,
    tag: &Self::Tag,
  ) -> Self::Output {
    let form = top_k_prob_proofs_to_boolean_formula(tag, ctx);
    let vars = form.collect_vars();
    let config = SDDBuilderConfig::with_formula(&form);
    let sdd = SDDBuilder::with_config(config).build(&form);
    let prob_semiring = DifferentiableProbabilitySemiring {
      vs: self.vs,
      shape: self.shape.clone(),
    };

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
        let one_hot = (0..num_sparse_indices).map(|i| {
          if var_id_to_sparse_id_map[var_id] == i { 1 } else { 0 }
        }).collect::<Vec<_>>();
        let one_hot_tensor = Tensor::of_slice(&one_hot);
        let one_hot_dev = self.vs.var_copy("i", &one_hot_tensor);

        let source_tensor = &ctx.diff_prob_table[var_id];
        let sparse_dn = SparseDualNumber::new(source_tensor.real(), one_hot_dev);

        (var_id.clone(), sparse_dn)
      })
      .collect::<HashMap<usize, SparseDualNumber>>();

    let result_sparse_vec = sdd.eval_t(&var_assign, &prob_semiring);

    let y_pred = result_sparse_vec.prob();
    let y_pred_sparse_grad = result_sparse_vec.grad();

    let var_id_to_deriv_map = (0..num_sparse_indices).map(|sparse_id| {
      let var_id = sparse_id_to_var_id_map[&sparse_id];
      let deriv = y_pred_sparse_grad.double_value(&[sparse_id as i64]);
      (var_id, deriv)
    }).collect::<HashMap<_, _>>();
    (y_pred, var_id_to_deriv_map)
  }
}

fn top_k_prob_proof_to_boolean_formula(proof: &DiffTopKProbProof) -> BooleanFormula {
  let mut iter = proof.facts.iter();
  let mut acc = bf(iter.next().unwrap().clone());
  while let Some(var_id) = iter.next() {
    acc = acc & bf(var_id.clone());
  }
  acc
}

/// Returns a conjunction over variables where all the others are negative
/// except the ith variable being positive
#[allow(dead_code)]
fn one_pos_clause(vars: &Vec<usize>, i: usize) -> BooleanFormula {
  let mut acc = bf(vars[i]);
  for j in 0..vars.len() {
    if j != i {
      acc = acc & !bf(vars[j]);
    }
  }
  acc
}

#[allow(dead_code)]
fn prob_proofs_disjunction_formula<const K: usize>(
  proofs: &DiffTopKProbProofs<K>,
  disj: &Disjunction,
) -> BooleanFormula {
  // Collect all the conflicting facts
  let facts_in_disj = proofs.proofs.iter().map(|proof| {
    proof.facts.intersection(disj)
  }).fold(BTreeSet::new(), |acc, itsct| {
    acc.union(&itsct.cloned().collect::<BTreeSet<_>>()).cloned().collect::<BTreeSet<_>>()
  }).into_iter().collect::<Vec<_>>();

  // Shortcut
  if facts_in_disj.len() < 2 {
    return BooleanFormula::True;
  }

  // Get all negative case
  let all_neg = {
    let mut acc = !bf(facts_in_disj[0]);
    for i in 1..facts_in_disj.len() {
      acc = acc & !bf(facts_in_disj[i]);
    }
    acc
  };

  // Get all clauses where only one of them is positive
  let one_pos_clauses = {
    let mut acc = one_pos_clause(&facts_in_disj, 0);
    for i in 1..facts_in_disj.len() {
      acc = acc | one_pos_clause(&facts_in_disj, i);
    }
    acc
  };

  // Use and to join all negative and one positive clauses
  all_neg | one_pos_clauses
}

#[allow(unused_variables)]
fn top_k_prob_proofs_to_boolean_formula<const K: usize>(
  proofs: &DiffTopKProbProofs<K>,
  ctx: &DiffProbProofContext,
) -> BooleanFormula {
  // First get the CNF formula
  let mut iter = proofs.proofs.iter();
  let mut acc = top_k_prob_proof_to_boolean_formula(iter.next().unwrap());
  while let Some(proof) = iter.next() {
    acc = acc | top_k_prob_proof_to_boolean_formula(proof);
  }

  // // Then get the disjunction conflicts
  // let mut disj_iter = ctx.disjunctions.iter();
  // let acc = if let Some(first_disj) = disj_iter.next() {
  //   let mut disj_fs = prob_proofs_disjunction_formula(proofs, first_disj);
  //   while let Some(disj) = disj_iter.next() {
  //     let disj_f = prob_proofs_disjunction_formula(proofs, disj);
  //     disj_fs = disj_fs & disj_f;
  //   }
  //   disj_fs & acc
  // } else {
  //   acc
  // };

  acc
}
