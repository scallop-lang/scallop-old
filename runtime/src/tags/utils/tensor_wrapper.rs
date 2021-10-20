use tch::Tensor;

#[derive(Debug)]
pub struct TensorWrapper(Tensor);

impl Clone for TensorWrapper {
  fn clone(&self) -> Self {
    Self(self.0.copy())
  }
}
