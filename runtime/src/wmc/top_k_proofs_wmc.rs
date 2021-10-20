use std::collections::*;

use sdd::{BooleanFormula, SDDBuilder, SDDBuilderConfig};

use super::prob_semiring::*;
use super::WeightedModelCounter;
use crate::semiring::*;
use crate::tags::{TopKProbProof, TopKProbProofs};

#[derive(Debug, Clone)]
pub struct TopKProbProofsWMC<const K: usize>;

impl<const K: usize> WeightedModelCounter for TopKProbProofsWMC<K> {
  type Tag = TopKProbProofs<K>;

  type Output = f32;

  fn wmc(
    &self,
    ctx: &<Self::Tag as Semiring>::Context,
    tag: &Self::Tag,
  ) -> Self::Output {
    let form = top_k_prob_proofs_to_boolean_formula(tag);
    let vars = form.collect_vars();
    let config = SDDBuilderConfig::with_formula(&form);
    let sdd = SDDBuilder::with_config(config).build(&form);
    let prob_semiring = ProbabilitySemiring;
    let var_assign = vars
      .iter()
      .map(|var_id| (var_id.clone(), ctx.prob_table[var_id]))
      .collect::<HashMap<usize, f32>>();
    sdd.eval_t(&var_assign, &prob_semiring)
  }
}

fn top_k_prob_proofs_to_boolean_formula<const K: usize>(
  proofs: &TopKProbProofs<K>,
) -> BooleanFormula {
  let mut iter = proofs.proofs.iter();
  let mut acc = top_k_prob_proof_to_boolean_formula(iter.next().unwrap());
  while let Some(proof) = iter.next() {
    acc = acc | top_k_prob_proof_to_boolean_formula(proof);
  }
  acc
}

fn top_k_prob_proof_to_boolean_formula(proof: &TopKProbProof) -> BooleanFormula {
  let mut iter = proof.facts.iter();
  let mut acc = BooleanFormula::Pos {
    var_id: iter.next().unwrap().clone(),
  };
  while let Some(var_id) = iter.next() {
    acc = acc
      & BooleanFormula::Pos {
        var_id: var_id.clone(),
      };
  }
  acc
}
