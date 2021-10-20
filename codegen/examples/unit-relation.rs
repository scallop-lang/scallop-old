use scallop_codegen::scallop;

scallop! {
  UnitRelation {
    decl prob_of_rule.
    decl input_1(Symbol, Symbol).
    decl input_2(Symbol, Symbol).
    decl target(Symbol, Symbol).

    target(A, B) :- input_1(A, _), input_2(B, _), prob_of_rule.
  }
}

fn main() {
  let mut prog = UnitRelation::<()>::new();

  // Initialize data
  prog.prob_of_rule().insert(vec![()]);

  prog.input_1().insert(vec![(1, 2)]);

  prog.input_2().insert(vec![(10, 12)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.target().complete().into_iter() {
    println!("{:?}", elem);
  }
}
