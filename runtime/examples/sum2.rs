use scallop_runtime::dataflows::*;
use scallop_runtime::*;

struct Sum2 {
  iter: Iteration<()>,
  digit: Variable<(usize, u8), ()>,
  sum: Variable<((usize, usize), u8), ()>,
}

impl Program<()> for Sum2 {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let digit = iter.variable::<(usize, u8)>();
    let sum = iter.variable::<((usize, usize), u8)>();
    Self { iter, digit, sum }
  }

  fn iteration(&self) -> &Iteration<()> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<()> {
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
  let mut prog = Sum2::new();

  // Initialize
  prog.iter.insert(
    &mut prog.digit,
    vec![
      // First digit
      (0, 0),
      (0, 1),
      (0, 2),
      (0, 3),
      // Second digit
      (1, 0),
      (1, 1),
      (1, 2),
      (1, 3),
    ],
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let result = prog.iter.complete(&prog.sum);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
