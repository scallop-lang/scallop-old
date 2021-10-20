pub struct CompileOptions {
  pub demand_transform: bool,
}

impl Default for CompileOptions {
  fn default() -> Self {
    Self {
      demand_transform: false
    }
  }
}

impl CompileOptions {
  pub fn with_demand_transform(mut self) -> Self {
    self.demand_transform = true;
    self
  }
}
