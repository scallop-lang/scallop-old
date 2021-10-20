use scallop_codegen::scallop;

scallop! {
  VQARSmallExample {
    decl name(Symbol, String).
    decl attr(Symbol, String).
    decl target(Symbol).

    0.8::name(1, "giraffe").
    0.3::attr(1, "tall").

    target(A) :- name(A, "giraffe"), attr(A, "tall").
  }
}

fn main() {
  let mut prog = VQARSmallExample::<TopKProbProofs<3>>::new();

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.target().complete().into_iter() {
    println!("{:?}", elem);
  }
}
