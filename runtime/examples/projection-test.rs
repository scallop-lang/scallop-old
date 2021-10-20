use scallop_runtime::dataflows::*;
use scallop_runtime::tags::*;
use scallop_runtime::*;

struct ProjectionTest<Tag: Semiring> {
  iter: Iteration<Tag>,
  source: Variable<(usize, usize, usize), Tag>,
  target: Variable<(usize, usize), Tag>,
}

impl<Tag: Semiring> Program<Tag> for ProjectionTest<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let source = iter.variable::<(usize, usize, usize)>();
    let target = iter.variable::<(usize, usize)>();
    Self {
      iter,
      source,
      target,
    }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    self
      .iter
      .insert_dataflow(&self.target, self.source.project(|(a, _, c)| (a, c)))
  }
}

fn main() {
  let mut prog = ProjectionTest::<TopKProbProofs<3>>::new();

  // Initialize
  prog.iter.insert_with_tag_info(
    &prog.source,
    vec![(0.5, (1, 2, 3)), (0.2, (2, 5, 8)), (0.1, (3, 9, 10))],
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let result = prog.iter.complete(&prog.target);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
