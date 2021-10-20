use tch::{nn, Tensor};

#[derive(Debug)]
pub struct DualNumber(f64, Tensor);

unsafe impl Send for DualNumber {}

unsafe impl Sync for DualNumber {}

impl Clone for DualNumber {
  fn clone(&self) -> Self {
    Self(self.0, self.1.copy())
  }
}

impl DualNumber {
  pub fn new(t: Tensor) -> Self {
    Self(t.double_value(&[]), t)
  }

  pub fn real(&self) -> f64 {
    self.0.clone()
  }

  pub fn tensor(&self) -> Tensor {
    self.1.copy()
  }

  pub fn zero(vs: &nn::Path, shape: &[i64]) -> Self {
    Self(0.0, vs.zeros("z", shape))
  }

  pub fn one(vs: &nn::Path, shape: &[i64]) -> Self {
    Self(1.0, vs.ones("o", shape))
  }

  pub fn add(&self, other: &Self) -> Self {
    Self(self.0 + other.0, &self.1 + &other.1)
  }

  pub fn mult(&self, other: &Self) -> Self {
    Self(self.0 * other.0, &self.1 * &other.1)
  }

  pub fn negate(&self) -> Self {
    Self(1.0 - self.0, 1.0 - &self.1)
  }
}
