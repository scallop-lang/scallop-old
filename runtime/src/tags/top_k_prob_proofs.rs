use std::cmp::Ordering;
use std::collections::*;

use crate::semiring::*;
use super::*;

#[derive(Clone, Debug)]
pub struct TopKProbProof {
  pub prob: f32,
  pub facts: BTreeSet<usize>,
}

impl PartialEq for TopKProbProof {
  fn eq(&self, other: &Self) -> bool {
    self.facts == other.facts
  }
}

impl Eq for TopKProbProof {}

impl PartialOrd for TopKProbProof {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    // Reversed ordering for MIN-HEAP
    other.prob.partial_cmp(&self.prob)
  }
}

impl Ord for TopKProbProof {
  fn cmp(&self, other: &Self) -> Ordering {
    // Reversed ordering for MIN-HEAP
    match other.prob.partial_cmp(&self.prob) {
      Some(ord) => ord,
      _ => other.facts.cmp(&self.facts),
    }
  }
}

#[derive(Clone, Debug)]
pub struct TopKProbProofs<const K: usize> {
  pub proofs: BinaryHeap<TopKProbProof>,
}

impl<const K: usize> TopKProbProofs<K> {
  pub fn new() -> Self {
    Self {
      proofs: BinaryHeap::new(),
    }
  }

  pub fn singleton(fact_id: usize, prob: f32) -> Self {
    let mut facts = BTreeSet::new();
    facts.insert(fact_id);
    let mut proofs = BinaryHeap::new();
    proofs.push(TopKProbProof { prob, facts });
    TopKProbProofs { proofs }
  }

  pub fn insert(&mut self, proof: TopKProbProof) {
    if self.proofs.len() < K {
      self.proofs.push(proof);
    } else {
      let p = self.proofs.peek().unwrap(); // len >= K, K > 0
      if p.prob < proof.prob {
        self.proofs.pop();
        self.proofs.push(proof);
      }
    }
  }
}

impl<const K: usize> Semiring for TopKProbProofs<K> {
  type Context = ProbProofContext;

  fn zero(_: &Self::Context) -> Self {
    Self::new()
  }

  fn one(_: &Self::Context) -> Self {
    let mut proofs = BinaryHeap::new();
    proofs.push(TopKProbProof {
      prob: 1.0,
      facts: BTreeSet::new(),
    });
    Self { proofs }
  }

  fn add(_: &Self::Context, s1: &Self, s2: &Self) -> Self {
    let mut proofs = s1.clone();
    for p2 in &s2.proofs {
      proofs.insert(p2.clone());
    }
    proofs
  }

  fn mult(ctx: &Self::Context, s1: &Self, s2: &Self) -> Self {
    let mut proofs = Self::new();
    for p1 in &s1.proofs {
      for p2 in &s2.proofs {
        let facts = p1.facts.union(&p2.facts).cloned().collect::<BTreeSet<_>>();
        if !has_conflict_in_disjunctions(&ctx.disjunctions, &facts) {
          let prob = facts
            .iter()
            .fold(1.0, |p, fact_id| p * ctx.prob_table[fact_id]);
          proofs.insert(TopKProbProof {
            prob: prob,
            facts: facts,
          });
        }
      }
    }
    proofs
  }

  fn is_valid(&self, _: &Self::Context) -> bool {
    !self.proofs.is_empty()
  }
}

impl<const K: usize> SemiringContext<TopKProbProofs<K>> for ProbProofContext {
  type Info = f32;

  fn base_tag(&mut self, prob: Self::Info) -> TopKProbProofs<K> {
    let id = self.id_counter;
    self.id_counter += 1;
    self.prob_table.insert(id, prob);
    TopKProbProofs::singleton(id, prob)
  }
}
