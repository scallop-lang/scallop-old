use scallop_codegen::*;

scallop! {
  HeadIsFact {
    decl head.
    decl tail.
    decl result(Symbol).

    head.

    result(1) :- head.
    result(0) :- tail.
  }
}

fn main() {
  let mut prog = HeadIsFact::<()>::new();

  // Execute the program
  prog.run();

  // Investigate the results
  for elem in prog.result().complete().into_iter() {
    println!("{:?}", elem);
  }
}
