use tch::{nn, Tensor};

#[derive(Debug)]
pub struct SparseDualNumber(f64, Tensor);

unsafe impl Send for SparseDualNumber {}

unsafe impl Sync for SparseDualNumber {}

impl Clone for SparseDualNumber {
  fn clone(&self) -> Self {
    Self(self.0, self.1.copy())
  }
}

impl SparseDualNumber {
  pub fn new(prob: f64, grad: Tensor) -> Self {
    Self(prob, grad)
  }

  pub fn prob(&self) -> f64 {
    self.0.clone()
  }

  pub fn grad(&self) -> Tensor {
    self.1.copy()
  }

  pub fn zero(vs: &nn::Path, shape: &[i64]) -> Self {
    Self(0.0, vs.zeros("z", shape))
  }

  pub fn one(vs: &nn::Path, shape: &[i64]) -> Self {
    Self(1.0, vs.zeros("o", shape))
  }

  pub fn add(&self, other: &Self) -> Self {
    Self(self.0 + other.0, &self.1 + &other.1)
  }

  pub fn mult(&self, other: &Self) -> Self {
    Self(self.0 * other.0, self.0 * &other.1 + other.0 * &self.1)
  }

  pub fn negate(&self) -> Self {
    Self(1.0 - self.0, -&self.1)
  }
}
