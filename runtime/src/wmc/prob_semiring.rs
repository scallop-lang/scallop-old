use sdd::Semiring as SDDSemiring;

pub struct ProbabilitySemiring;

impl SDDSemiring for ProbabilitySemiring {
  type Element = f32;

  fn zero(&self) -> Self::Element {
    0.0
  }

  fn one(&self) -> Self::Element {
    1.0
  }

  fn add(&self, a: Self::Element, b: Self::Element) -> Self::Element {
    a + b
  }

  fn mult(&self, a: Self::Element, b: Self::Element) -> Self::Element {
    a * b
  }

  fn negate(&self, a: Self::Element) -> Self::Element {
    1.0 - a
  }
}
