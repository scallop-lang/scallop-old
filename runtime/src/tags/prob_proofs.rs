use std::collections::*;

use crate::semiring::*;
use super::utils::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProbProof {
  pub facts: BTreeSet<usize>,
}

impl ProbProof {
  pub fn singleton(id: usize) -> Self {
    let mut facts = BTreeSet::new();
    facts.insert(id);
    Self { facts }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProbProofs {
  pub proofs: BTreeSet<ProbProof>,
}

impl ProbProofs {
  pub fn singleton(id: usize) -> Self {
    let proof = ProbProof::singleton(id);
    let mut proofs = BTreeSet::new();
    proofs.insert(proof);
    Self { proofs }
  }
}

impl Semiring for ProbProofs {
  type Context = ProbProofContext;

  fn zero(_: &Self::Context) -> Self {
    Self {
      proofs: BTreeSet::new(),
    }
  }

  fn one(_: &Self::Context) -> Self {
    let mut proofs = BTreeSet::new();
    proofs.insert(ProbProof {
      facts: BTreeSet::new(),
    });
    Self { proofs }
  }

  fn add(_: &Self::Context, t1: &Self, t2: &Self) -> Self {
    Self {
      proofs: t1.proofs.union(&t2.proofs).cloned().collect(),
    }
  }

  fn mult(ctx: &Self::Context, t1: &Self, t2: &Self) -> Self {
    let mut result = BTreeSet::new();
    for p1 in &t1.proofs {
      for p2 in &t2.proofs {
        let facts = p1.facts.union(&p2.facts).cloned().collect();
        if !has_conflict_in_disjunctions(&ctx.disjunctions, &facts) {
          let proof = ProbProof { facts };
          result.insert(proof);
        }
      }
    }
    Self { proofs: result }
  }

  fn is_valid(&self, _: &Self::Context) -> bool {
    !self.proofs.is_empty()
  }
}

impl SemiringContext<ProbProofs> for ProbProofContext {
  type Info = f32;

  fn base_tag(&mut self, prob: Self::Info) -> ProbProofs {
    let id = self.id_counter;
    self.id_counter += 1;
    self.prob_table.insert(id, prob);
    ProbProofs::singleton(id)
  }
}
