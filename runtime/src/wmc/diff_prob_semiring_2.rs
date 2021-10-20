use sdd::Semiring as SDDSemiring;

use crate::tags::DualNumber2;

pub struct DiffProbabilitySemiring2;

impl SDDSemiring for DiffProbabilitySemiring2 {
  type Element = DualNumber2;

  fn zero(&self) -> Self::Element {
    Self::Element::zero()
  }

  fn one(&self) -> Self::Element {
    Self::Element::one()
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
