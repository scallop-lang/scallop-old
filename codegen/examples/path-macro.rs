use scallop_codegen::scallop;

scallop! {
  Path {
    decl edge(Symbol, Symbol).
    decl path(Symbol, Symbol).
    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).
  }
}

fn main() {
  let mut prog = Path::<()>::new();

  // Initialize data
  prog.edge().insert(vec![(0, 0), (0, 1), (1, 3), (2, 3)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.path().complete().into_iter() {
    println!("{:?}", elem);
  }
}
