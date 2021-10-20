use scallop_codegen::scallop;

scallop! {
  Sum2 {
    decl digit(Symbol, Int).
    decl sum(Symbol, Symbol, Int).
    sum(A, B, DA + DB) :- digit(A, DA), digit(B, DB).
  }
}

fn main() {
  let mut prog = Sum2::<()>::new();

  // Initialize data
  prog.digit().insert(vec![(0, 0), (0, 1), (1, 2), (1, 3)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.sum().complete().into_iter() {
    println!("{:?}", elem);
  }
}
