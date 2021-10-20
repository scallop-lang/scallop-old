use scallop_codegen::scallop;

scallop! {
  FactInBody {
    decl a(Int).
    decl b(Int).
    decl c(Int, Int).
    decl d(Int, Int).
    decl result(Int, Int).

    c(A, B) :- d(A, B).

    result(A, B) :- a(A), b(B), c(1, 1).
  }
}

fn main() {
  let mut prog = FactInBody::<()>::new();

  // Initialize data
  prog.a().insert(vec![1, 2]);
  prog.b().insert(vec![4, 5, 6]);
  prog.c().insert(vec![(0, 0)]);
  prog.d().insert(vec![(1, 1)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.result().complete().into_iter() {
    println!("{:?}", elem);
  }
}
