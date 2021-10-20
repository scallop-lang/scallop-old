use scallop_codegen::scallop;

scallop! {
  ConstInHead {
    decl rela_a(Int, Int, Int).
    decl result(Int, Int).

    result(A, 100) :- rela_a(_, A, 3).
  }
}

fn main() {
  let mut prog = ConstInHead::<()>::new();

  // Initialize data
  prog.rela_a().insert(vec![(2, 1, 3), (5, 1, 2), (7, 2, 3)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.result().complete().into_iter() {
    println!("{:?}", elem);
  }
}
