use scallop_runtime::tags::*;
use scallop_runtime::*;

struct Student<Tag: Semiring> {
  iter: Iteration<Tag>,
  color: Variable<(usize, &'static str), Tag>,
  shape: Variable<(usize, &'static str), Tag>,
  query: Variable<(usize, &'static str, &'static str), Tag>,
}

impl<Tag: Semiring> Program<Tag> for Student<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let color = iter.variable::<(usize, &'static str)>();
    let shape = iter.variable::<(usize, &'static str)>();
    let query = iter.variable::<(usize, &'static str, &'static str)>();
    Self {
      iter,
      color,
      shape,
      query,
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
      .insert_dataflow(&self.query, self.iter.join(&self.color, &self.shape));
  }
}

fn main() {
  let mut prog = Student::<TopKProbProofs<3>>::new();

  // Initialize
  prog.iter.insert_with_tag_info(
    &prog.color,
    vec![(0.5, (1, "yellow")), (0.1, (1, "blue")), (0.6, (1, "red"))],
  );
  prog.iter.insert_with_tag_info(
    &prog.shape,
    vec![
      (0.3, (0, "cube")),
      (0.2, (1, "sphere")),
      (0.8, (2, "sphere")),
    ],
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let result = prog.iter.complete(&prog.query);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
