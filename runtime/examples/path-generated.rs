mod scallop_path {
  use scallop_runtime::dataflows::*;
  use scallop_runtime::*;
  pub struct Path<Tag: Semiring> {
    pub iter: Iteration<Tag>,
    pub edge: Variable<(usize, usize), Tag>,
    pub path: Variable<(usize, usize), Tag>,
    _tmp_0: Variable<(usize, usize), Tag>,
  }
  impl<Tag: Semiring> Program<Tag> for Path<Tag> {
    fn new() -> Self {
      let mut iter = Iteration::new();
      let edge = iter.variable::<(usize, usize)>();
      let path = iter.variable::<(usize, usize)>();
      let _tmp_0 = iter.variable::<(usize, usize)>();
      Self {
        iter,
        edge,
        path,
        _tmp_0,
      }
    }
    fn iteration(&self) -> &Iteration<Tag> {
      &self.iter
    }
    fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
      &mut self.iter
    }
    fn update(&self) {
      self.iter.insert_dataflow(&self.path, &self.edge);
      self
        .iter
        .insert_dataflow(&self._tmp_0, self.edge.project(|arg| (arg.1, arg.0)));
      self.iter.insert_dataflow(
        &self.path,
        self
          .iter
          .join(&self._tmp_0, &self.path)
          .project(|arg| (arg.1, arg.2)),
      );
    }
  }
}

use scallop_path::*;
use scallop_runtime::*;

fn main() {
  let mut prog = Path::<()>::new();
  prog
    .iter
    .insert(&prog.edge, vec![(1, 2), (2, 3), (3, 4), (3, 5)]);
  prog.run();
  for elem in prog.iter.complete(&prog.path).iter() {
    println!("{:?}", elem.tup);
  }
}
