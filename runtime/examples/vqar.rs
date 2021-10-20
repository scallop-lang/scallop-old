use std::iter::FromIterator;

use scallop_runtime::tags::*;
use scallop_runtime::*;

struct VQAReasoning<Tag: Semiring> {
  iter: Iteration<Tag>,
  name: Variable<(usize, &'static str), Tag>,
  is_a: Variable<(&'static str, &'static str), Tag>,
  result: Variable<((usize, &'static str), (&'static str, &'static str)), Tag>,
}

impl<Tag: Semiring> Program<Tag> for VQAReasoning<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let name = iter.variable::<(usize, &str)>();
    let is_a = iter.variable::<(&str, &str)>();
    let result = iter.variable::<((usize, &str), (&str, &str))>();
    Self {
      iter,
      name,
      is_a,
      result,
    }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    let prod = self.iter.product(&self.name, &self.is_a);
    self.iter.insert_dataflow(&self.result, prod);
  }
}

fn main() {
  // Setup top 3 provenance tracking
  let mut prog = VQAReasoning::<TopKProbProofs<3>>::new();

  // Setup disjunction
  prog
    .iter
    .semiring_ctx
    .disjunctions
    .push(Disjunction::from_iter(vec![0, 1, 2]));

  // Initialization
  prog.iter.insert_with_tag_info(
    &prog.name,
    vec![
      (0.8, (10503, "giraffe")),
      (0.1, (10503, "tiger")),
      (0.1, (10503, "plant")),
    ],
  );
  prog.iter.insert_with_tag_info(
    &prog.is_a,
    vec![(1.0, ("giraffe", "animal")), (1.0, ("tiger", "animal"))],
  );

  // Main loop
  prog.run();

  // Result inspection
  let result = prog.iter.complete(&prog.result);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
