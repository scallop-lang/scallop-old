use std::cmp::Ordering;
use std::collections::*;

use crate::semiring::*;
use super::utils::*;

/// Differentiable Top-K Probabilistic Proof
#[derive(Clone, Debug)]
pub struct DiffTopKProbProof {
  pub prob: f64,
  pub facts: BTreeSet<usize>,
}

/// Differentiable Top-K Probabilistic Proof are equal if all facts are equal
impl PartialEq for DiffTopKProbProof {
  fn eq(&self, other: &Self) -> bool {
    self.facts == other.facts
  }
}

/// Differentiable Top-K Probabilistic Proof also implements Equal
impl Eq for DiffTopKProbProof {}

/// Differentiable Top-K Probabilistic Proof's compare is its probability's partial order
impl PartialOrd for DiffTopKProbProof {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    // Reversed ordering for MIN-HEAP
    other.prob.partial_cmp(&self.prob)
  }
}

/// If the probability does not have order, we get order from the facts
impl Ord for DiffTopKProbProof {
  fn cmp(&self, other: &Self) -> Ordering {
    // Reversed ordering for MIN-HEAP
    match other.prob.partial_cmp(&self.prob) {
      Some(ord) => ord,
      _ => other.facts.cmp(&self.facts),
    }
  }
}

/// Differentiable Beam of Proofs
///
/// A set of proofs ordered by their probabilities.
/// This set only contain K proofs
///
/// Internally, this set of proofs is managed by a Binary Heap
#[derive(Clone, Debug)]
pub struct DiffTopKProbProofs<const K: usize> {
  pub proofs: BinaryHeap<DiffTopKProbProof>,
}

impl<const K: usize> DiffTopKProbProofs<K> {
  /// Create a new beam of proofs
  pub fn new() -> Self {
    Self {
      proofs: BinaryHeap::new(),
    }
  }

  /// Create a new beam of proofs that only contains one proof with one fact
  ///
  /// \tilde{S}_f = { {f} },
  /// with \Pr(f) = p
  pub fn singleton(fact_id: usize, prob: f64) -> Self {
    let mut facts = BTreeSet::new();
    facts.insert(fact_id);
    let mut proofs = BinaryHeap::new();
    proofs.push(DiffTopKProbProof { prob, facts });
    DiffTopKProbProofs { proofs }
  }

  /// Insert a new proof into the beam of proofs.
  /// If there are already K proofs, we will remove the one with the lowest probability
  pub fn insert(&mut self, proof: DiffTopKProbProof) {
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

impl<const K: usize> Semiring for DiffTopKProbProofs<K> {
  type Context = DiffProbProofContext;

  /// Zero is an empty set ({})
  fn zero(_: &Self::Context) -> Self {
    Self::new()
  }

  /// One is a singleton set with one empty set proof ({{}})
  fn one(_: &Self::Context) -> Self {
    let mut proofs = BinaryHeap::new();
    proofs.push(DiffTopKProbProof {
      prob: 1.0,
      facts: BTreeSet::new(),
    });
    Self { proofs }
  }

  /// Add two beams of proofs: doing set union
  fn add(_: &Self::Context, s1: &Self, s2: &Self) -> Self {
    let mut proofs = s1.clone();
    for p2 in &s2.proofs {
      proofs.insert(p2.clone());
    }
    proofs
  }

  /// Multiply two beams of proofs
  ///
  /// S1 X S2 = {F1 U F2 | F1 in S1, F2 in S2, F1 U F2 has no disjunction conflict}
  ///
  /// Will keep only the top-k proofs at the end
  fn mult(ctx: &Self::Context, s1: &Self, s2: &Self) -> Self {
    let mut proofs = Self::new();
    for p1 in &s1.proofs {
      for p2 in &s2.proofs {
        let facts = p1.facts.union(&p2.facts).cloned().collect::<BTreeSet<_>>();
        if !has_conflict_in_disjunctions(&ctx.disjunctions, &facts) {
          let prob = facts
            .iter()
            .fold(1.0, |p, fact_id| {
              p * ctx.diff_prob_table[fact_id].real()
            });
          proofs.insert(DiffTopKProbProof {
            prob: prob,
            facts: facts,
          });
        }
      }
    }
    proofs
  }

  /// A beam of proofs is invalid if there is no proof (of the tagged fact)
  fn is_valid(&self, _: &Self::Context) -> bool {
    !self.proofs.is_empty()
  }
}

impl<const K: usize> SemiringContext<DiffTopKProbProofs<K>> for DiffProbProofContext {
  type Info = DualNumber;

  fn base_tag(&mut self, dual_number: Self::Info) -> DiffTopKProbProofs<K> {
    let id = self.id_counter;
    let result = DiffTopKProbProofs::singleton(id, dual_number.real());
    self.diff_prob_table.insert(id, dual_number);
    self.id_counter += 1;
    result
  }
}
