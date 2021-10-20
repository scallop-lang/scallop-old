pub trait Probability {
  fn prob(&self) -> f32;
}

impl Probability for f32 {
  fn prob(&self) -> f32 {
    self.clone()
  }
}
