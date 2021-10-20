use scallop_runtime::dataflows::*;
use scallop_runtime::tags::*;
use scallop_runtime::*;

struct Sum2<Tag: Semiring> {
  iter: Iteration<Tag>,
  digit: Variable<(usize, u8), Tag>,
  sum: Variable<((usize, usize), u8), Tag>,
}

impl<Tag: Semiring> Program<Tag> for Sum2<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let digit = iter.variable::<(usize, u8)>();
    let sum = iter.variable::<((usize, usize), u8)>();
    Self { iter, digit, sum }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    self.iter.insert_dataflow(
      &self.sum,
      self
        .iter
        .product(self.digit.find(0), self.digit.find(1))
        .project(|((o1, d1), (o2, d2))| ((o1, o2), d1 + d2)),
    );
  }
}

fn main() {
  let mut prog = Sum2::<ProbProofs>::new();

  // First digit
  prog.iter.insert_disjunction(
    &mut prog.digit,
    vec![(0.1, (0, 0)), (0.6, (0, 1)), (0.2, (0, 2)), (0.1, (0, 3))],
  );

  // Second digit
  prog.iter.insert_disjunction(
    &mut prog.digit,
    vec![(0.6, (1, 0)), (0.1, (1, 1)), (0.2, (1, 2)), (0.1, (1, 3))],
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let result = prog.iter.complete(&prog.sum);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
