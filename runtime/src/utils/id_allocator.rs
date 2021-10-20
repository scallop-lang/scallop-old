#[derive(Clone, Debug, Default)]
pub struct IdAllocator {
  pub curr_id: usize,
}

impl IdAllocator {
  pub fn new() -> Self {
    Self { curr_id: 0 }
  }

  pub fn allocate(&mut self) -> usize {
    let id = self.curr_id;
    self.curr_id += 1;
    id
  }
}
