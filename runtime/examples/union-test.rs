use scallop_runtime::tags::*;
use scallop_runtime::*;

struct Student<Tag: Semiring> {
  iter: Iteration<Tag>,
  third_grade: Variable<(usize,), Tag>,
  forth_grade: Variable<(usize,), Tag>,
  is_male: Variable<(usize,), Tag>,
  is_female: Variable<(usize,), Tag>,
  query: Variable<(usize,), Tag>,
}

impl<Tag: Semiring> Program<Tag> for Student<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let third_grade = iter.variable::<(usize,)>();
    let forth_grade = iter.variable::<(usize,)>();
    let is_male = iter.variable::<(usize,)>();
    let is_female = iter.variable::<(usize,)>();
    let query = iter.variable::<(usize,)>();
    Self {
      iter,
      third_grade,
      forth_grade,
      is_male,
      is_female,
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
    self.iter.insert_dataflow(
      &self.query,
      self.iter.union(&self.third_grade, &self.is_female),
    );
  }
}

fn main() {
  let mut prog = Student::<TopKProbProofs<3>>::new();

  // Initialize
  prog.iter.insert_with_tag_info(
    &prog.third_grade,
    vec![(0.5, (1,)), (0.2, (2,)), (0.1, (3,))],
  );
  prog.iter.insert_with_tag_info(
    &prog.forth_grade,
    vec![(0.2, (4,)), (0.1, (5,)), (0.5, (6,))],
  );
  prog
    .iter
    .insert_with_tag_info(&prog.is_male, vec![(0.3, (1,)), (0.8, (4,))]);
  prog.iter.insert_with_tag_info(
    &prog.is_female,
    vec![(0.01, (2,)), (0.8, (3,)), (0.3, (6,))],
  );

  // Execute the program
  prog.run();

  // Investigate the results
  let result = prog.iter.complete(&prog.query);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
