use scallop_runtime::dataflows::*;
use scallop_runtime::tags::*;
use scallop_runtime::wmc::*;
use scallop_runtime::*;

/// ``` datalog
/// digit(ImageID, Int).
/// sum(ImageID, ImageID, Int).
///
/// 0.1::digit(0, 0);
/// 0.6::digit(0, 1);
/// 0.2::digit(0, 2);
/// 0.1::digit(0, 3).
///
/// 0.6::digit(1, 0);
/// 0.1::digit(1, 1);
/// 0.2::digit(1, 2);
/// 0.1::digit(1, 3).
///
/// sum(A, B, C, D1 + D2 + D3) :- digit(A, D1), digit(B, D2), digit(C, D3).
/// ```
struct Sum3<Tag: Semiring<Context = ProbProofContext>> {
  iter: Iteration<Tag>,
  digit: Variable<(usize, u8), Tag>,
  sum: Variable<((usize, usize, usize), u8), Tag>,
}

impl<Tag: Semiring<Context = ProbProofContext>> Program<Tag> for Sum3<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let digit = iter.variable::<(usize, u8)>();
    let sum = iter.variable::<((usize, usize, usize), u8)>();
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
        .product(
          self
            .iter
            .product(self.digit.find(0), self.digit.find(1))
            .project(|((o1, d1), (o2, d2))| ((o1, o2), d1 + d2)),
          self.digit.find(2),
        )
        .project(|(((o1, o2), d12), (o3, d3))| ((o1, o2, o3), d12 + d3)),
    );
  }
}

fn ten_normalized_numbers(seed: u64) -> Vec<f32> {
  use rand::prelude::*;
  let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
  let mut numbers = (0..9).map(|_| rng.gen::<f32>()).collect::<Vec<_>>();
  let sum = numbers.iter().fold(0.0, |a, p| a + p);
  for i in 0..9 {
    numbers[i] /= sum;
  }
  numbers
}

fn main() {
  let mut prog = Sum3::<TopKProbProofs<2>>::new();

  // First digit
  prog.iter.insert_disjunction(
    &mut prog.digit,
    ten_normalized_numbers(12345)
      .into_iter()
      .enumerate()
      .map(|(i, p)| (p, (0, i as u8)))
      .collect(),
  );

  // Second digit
  prog.iter.insert_disjunction(
    &mut prog.digit,
    ten_normalized_numbers(23456)
      .into_iter()
      .enumerate()
      .map(|(i, p)| (p, (1, i as u8)))
      .collect(),
  );

  // Second digit
  prog.iter.insert_disjunction(
    &mut prog.digit,
    ten_normalized_numbers(34567)
      .into_iter()
      .enumerate()
      .map(|(i, p)| (p, (2, i as u8)))
      .collect(),
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let wmc = TopKProbProofsWMC::<2>;
  let result = prog.iter.complete(&prog.sum);
  for elem in result.iter() {
    println!(
      "{:?}, Prob: {}",
      elem,
      wmc.wmc(&prog.iter.semiring_ctx, &elem.tag)
    );
  }
}
