use scallop_runtime::dataflows::*;
use scallop_runtime::*;

/// ``` datalog
/// decl edge(Symbol, Symbol).
/// decl path(Symbol, Symbol).
///
/// path(A, B) :- edge(A, B).
/// path(A, C) :- edge(A, B), path(B, C).
/// ```
struct PathEdgeTest<Tag: Semiring> {
  iter: Iteration<Tag>,
  edge: Variable<(usize, usize), Tag>,     // B, C
  path: Variable<(usize, usize), Tag>,     // A, B
  path_inv: Variable<(usize, usize), Tag>, // B, A
}

impl<Tag: Semiring> Program<Tag> for PathEdgeTest<Tag> {
  fn new() -> Self {
    let mut iter = Iteration::new();
    let edge = iter.variable::<(usize, usize)>();
    let path = iter.variable::<(usize, usize)>();
    let path_inv = iter.variable::<(usize, usize)>();
    Self {
      iter,
      edge,
      path,
      path_inv,
    }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    // Iniitalize edge to path
    self.iter.insert_dataflow(&self.path, &self.edge);

    // Initialize edge to path_inv
    self
      .iter
      .insert_dataflow(&self.path_inv, self.edge.project(|(a, b)| (b, a)));

    // Join path_inv and edge to get new path_inv's
    self.iter.insert_dataflow(
      &self.path_inv,
      self
        .iter
        .join(&self.path_inv, &self.edge)
        .project(|(_, a, c)| (c, a)),
    );

    // Finally insert path_inv to path
    self
      .iter
      .insert_dataflow(&self.path, self.path_inv.project(|(b, a)| (a, b)));
  }
}

fn main() {
  // Setup top 3 provenance tracking
  let mut prog = PathEdgeTest::<()>::new();

  // Initialization
  prog
    .iter
    .insert(&mut prog.edge, vec![(1, 2), (2, 3), (3, 4), (4, 5)]);

  // Main loop
  prog.run();

  // Result inspection
  let result = prog.iter.complete(&prog.path);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
