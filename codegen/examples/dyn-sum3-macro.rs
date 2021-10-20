use scallop_codegen::scallop;
use scallop_runtime::error::RuntimeError;

scallop! {
  Digit {
    decl digit(Symbol, Int).
  }
}

fn run_sum_3<Tag: Semiring>(prog: &mut Digit<Tag>) -> Result<(), RuntimeError> {
  prog.add_variable("sum_3", <TupleType as FromType<(usize, usize, usize, i64)>>::from_type())?;
  let rule_sum_3 = prog.add_rule("sum_3(A, B, C, DA + DB + DC) :- digit(A, DA), digit(B, DB), digit(C, DC).")?;

  // Execute the program
  prog.run();

  // Investigate result
  for elem in prog.get_variable("sum_3")?.complete().iter() {
    println!("{:?}", elem);
  }

  // Remove the rules
  prog.remove_rule(rule_sum_3)?;
  prog.remove_variable("sum_3")
}

fn run_sum_2<Tag: Semiring>(prog: &mut Digit<Tag>) -> Result<(), RuntimeError> {
  prog.add_variable("sum_2", <TupleType as FromType<(usize, usize, i64)>>::from_type())?;
  let rule_sum_2 = prog.add_rule("sum_2(A, B, DA + DB) :- digit(A, DA), digit(B, DB).")?;

  // Execute the program
  prog.run();

  // Investigate result
  for elem in prog.get_variable("sum_2")?.complete().iter() {
    println!("{:?}", elem);
  }

  // Remove the rules
  prog.remove_rule(rule_sum_2)?;
  prog.remove_variable("sum_2")
}

fn main() -> Result<(), RuntimeError> {
  let mut prog = Digit::<()>::new();

  prog.digit().insert(vec![(0, 0), (0, 1), (1, 2), (1, 3)]);

  // Run the tasks
  run_sum_2(&mut prog)?;
  run_sum_3(&mut prog)?;

  Ok(())
}
