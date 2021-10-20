use scallop_codegen::scallop;

scallop! {
  Constraint1 {
    decl height(Symbol, Int).
    decl taller(Symbol, Symbol).
    taller(A, B) :- height(A, TA), height(B, TB), TA > TB.
  }
}

fn main() {
  let mut prog = Constraint1::<()>::new();

  // Initialize data
  prog
    .height()
    .insert(vec![(0, 155), (1, 160), (2, 168), (3, 176)]);

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.taller().complete().into_iter() {
    println!("{:?}", elem);
  }
}
