use scallop_codegen::scallop;

scallop! {
  ConstInBody {
    decl rela_a(Int, Int, Int).
    decl result(Int, Int).

    result(A, B) :- rela_a(A, B, 1).
  }
}

fn main() {
  let mut prog = ConstInBody::<()>::new();

  // Initialize data
  prog.rela_a().insert(vec![(0, 1, 2), (1, 2, 3), (2, 3, 1)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.result().complete().into_iter() {
    println!("{:?}", elem);
  }
}
