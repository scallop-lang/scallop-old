use tch::nn;
use sdd::Semiring as SDDSemiring;

use crate::tags::SparseDualNumber;

pub struct DifferentiableProbabilitySemiring<'a> {
  pub vs: &'a nn::Path<'a>,
  pub shape: Vec<i64>,
}

impl<'a> SDDSemiring for DifferentiableProbabilitySemiring<'a> {
  type Element = SparseDualNumber;

  fn zero(&self) -> Self::Element {
    Self::Element::zero(self.vs, &self.shape)
  }

  fn one(&self) -> Self::Element {
    Self::Element::one(self.vs, &self.shape)
  }

  fn add(&self, a: Self::Element, b: Self::Element) -> Self::Element {
    a.add(&b)
  }

  fn mult(&self, a: Self::Element, b: Self::Element) -> Self::Element {
    a.mult(&b)
  }

  fn negate(&self, a: Self::Element) -> Self::Element {
    a.negate()
  }
}
