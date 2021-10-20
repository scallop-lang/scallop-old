use scallop_codegen::scallop;

scallop! {
  JoinSubset {
    decl rela_a(Symbol, Symbol, Symbol).
    decl rela_b(Symbol, Symbol).
    decl result(Symbol, Symbol).

    result(A, B) :- rela_b(B, C), rela_a(A, B, C).
  }
}

fn main() {
  let mut prog = JoinSubset::<()>::new();

  // Initialize data
  prog.rela_a().insert(vec![(0, 1, 2), (1, 2, 3)]);

  prog.rela_b().insert(vec![(1, 2), (10, 13)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.result().complete().into_iter() {
    println!("{:?}", elem);
  }
}
