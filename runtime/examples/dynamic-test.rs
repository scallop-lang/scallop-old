use scallop_runtime::interpreter::*;
use scallop_runtime::tags::*;
use scallop_runtime::*;

struct Student<Tag: Semiring> {
  iter: Iteration<Tag>,

  // Variables
  third_grade: Variable<(usize,), Tag>,
  forth_grade: Variable<(usize,), Tag>,
  is_male: Variable<(usize,), Tag>,
  is_female: Variable<(usize,), Tag>,
  query_0: Variable<(usize,), Tag>,
}

impl<Tag: Semiring> Student<Tag> {
  pub fn dynamic_variable<'a>(&'a mut self, name: &str) -> Option<DynVariableHandle<'a, Tag>> {
    match self.iter.get_dynamic_variable(name) {
      Some(var) => {
        let var = var.clone();
        let ctx = &mut self.iter.semiring_ctx;
        Some(DynVariableHandle::new(var, ctx))
      }
      None => None
    }
  }

  pub fn third_grade<'a>(&'a mut self) -> VariableHandle<'a, (usize,), Tag> {
    self.iter.variable_handle(&self.third_grade)
  }

  pub fn forth_grade<'a>(&'a mut self) -> VariableHandle<'a, (usize,), Tag> {
    self.iter.variable_handle(&self.forth_grade)
  }

  pub fn is_male<'a>(&'a mut self) -> VariableHandle<'a, (usize,), Tag> {
    self.iter.variable_handle(&self.is_male)
  }

  pub fn is_female<'a>(&'a mut self) -> VariableHandle<'a, (usize,), Tag> {
    self.iter.variable_handle(&self.is_female)
  }

  pub fn query_0<'a>(&'a mut self) -> VariableHandle<'a, (usize,), Tag> {
    self.iter.variable_handle(&self.query_0)
  }
}

impl<Tag: Semiring> Program<Tag> for Student<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();

    // Initialize variables
    let third_grade = iter.static_variable::<(usize,)>("third_grade");
    let forth_grade = iter.static_variable::<(usize,)>("forth_grade");
    let is_male = iter.static_variable::<(usize,)>("is_male");
    let is_female = iter.static_variable::<(usize,)>("is_female");
    let query_0 = iter.variable::<(usize,)>();

    // Programs
    Self {
      iter,
      third_grade,
      forth_grade,
      is_male,
      is_female,
      query_0,
    }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    self.query_0.insert(
      &self.iter.semiring_ctx,
      self.iter.intersect(&self.third_grade, &self.is_female),
    )
  }
}

fn main() {
  let mut prog = Student::<TopKProbProofs<3>>::new();

  // Initialize
  prog
    .third_grade()
    .insert_with_tag_info(vec![(0.5, (1,)), (0.2, (2,)), (0.1, (3,))]);
  prog
    .forth_grade()
    .insert_with_tag_info(vec![(0.2, (4,)), (0.1, (5,)), (0.5, (6,))]);
  prog
    .is_male()
    .insert_with_tag_info(vec![(0.3, (1,)), (0.8, (4,))]);
  prog
    .is_female()
    .insert_with_tag_info(vec![(0.01, (2,)), (0.8, (3,)), (0.3, (6,))]);

  // !!!! Add query at runtime !!!!
  prog.iteration_mut().dynamic_variable("query_1", <TupleType as FromType<(usize,)>>::from_type());
  prog.iteration_mut().add_rule("query_1(A) :- third_grade(A), is_female(A).").unwrap();
  prog.iteration_mut().dynamic_variable("query_2", <TupleType as FromType<(usize,)>>::from_type());
  prog.iteration_mut().add_rule("query_2(A) :- forth_grade(A), is_male(A).").unwrap();

  // Execute the program
  prog.run();

  // !!!! Add query after the first runthrough !!!!
  prog.iteration_mut().dynamic_variable("query_3", <TupleType as FromType<(usize,)>>::from_type());
  prog.iteration_mut().add_rule("query_3(A) :- third_grade(A), is_male(A).").unwrap();

  // Execute the program once again
  prog.run();

  // !!!! Add query after the first runthrough !!!!
  prog.iteration_mut().dynamic_variable("query_4", <TupleType as FromType<(usize,)>>::from_type());
  prog.iteration_mut().add_rule("query_4(A) :- forth_grade(A), is_female(A).").unwrap();

  // Execute the program once again
  prog.run();

  // Investigate the results
  println!("Query 0");
  for elem in prog.query_0().complete().into_iter() {
    println!("{:?}", elem);
  }

  // Investigate the results
  println!("Query 1");
  let q1_result = prog.dynamic_variable("query_1").unwrap().complete();
  for elem in q1_result.iter() {
    println!("{:?}", elem);
  }

  println!("Query 2");
  let q2_result = prog.dynamic_variable("query_2").unwrap().complete();
  for elem in q2_result.iter() {
    println!("{:?}", elem);
  }

  println!("Query 3");
  let q3_result = prog.dynamic_variable("query_3").unwrap().complete();
  for elem in q3_result.iter() {
    println!("{:?}", elem);
  }

  println!("Query 4");
  let q4_result = prog.dynamic_variable("query_4").unwrap().complete();
  for elem in q4_result.iter() {
    println!("{:?}", elem);
  }
}
