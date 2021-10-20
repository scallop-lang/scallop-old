use scallop_runtime::dataflows::*;
use scallop_runtime::*;

type Num = u8;

struct Sudoku4x4<Tag: Semiring> {
  iter: Iteration<Tag>,
  number: Variable<Num, Tag>,
  two_nums: Variable<(Num, Num), Tag>,
  row: Variable<(Num, Num, Num, Num), Tag>,
  two_rows: Variable<((Num, Num, Num, Num), (Num, Num, Num, Num)), Tag>,
  board: Variable<
    (
      ((Num, Num, Num, Num), (Num, Num, Num, Num)),
      ((Num, Num, Num, Num), (Num, Num, Num, Num)),
    ),
    Tag,
  >,
}

impl<Tag: Semiring> Program<Tag> for Sudoku4x4<Tag> {
  fn new() -> Self {
    // Setup variables
    let mut iter = Iteration::new();
    let number = iter.variable::<Num>();
    let two_nums = iter.variable::<(Num, Num)>();
    let row = iter.variable::<(Num, Num, Num, Num)>();
    let two_rows = iter.variable::<((Num, Num, Num, Num), (Num, Num, Num, Num))>();
    let board = iter.variable::<(
      ((Num, Num, Num, Num), (Num, Num, Num, Num)),
      ((Num, Num, Num, Num), (Num, Num, Num, Num)),
    )>();

    // Insert the ground facts that numbers are 1, 2, 3, and 4
    iter.insert_ground(&number, vec![1, 2, 3, 4]);

    Self {
      iter,
      number,
      two_nums,
      row,
      two_rows,
      board,
    }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }

  fn update(&self) {
    self.iter.insert_dataflow(
      &self.two_nums,
      self
        .iter
        .product(&self.number, &self.number)
        .filter(|(a, b)| a != b),
    );

    self.iter.insert_dataflow(
      &self.row,
      self
        .iter
        .product(&self.two_nums, &self.two_nums)
        .filter(|((a, b), (c, d))| a != c && b != c && a != d && b != d)
        .project(|((a, b), (c, d))| (a, b, c, d)),
    );

    self.iter.insert_dataflow(
      &self.two_rows,
      self
        .iter
        .product(&self.row, &self.row)
        .filter(|((a1, b1, c1, d1), (a2, b2, c2, d2))| {
          a1 != a2
            && a1 != b2
            && b1 != a2
            && b1 != b2
            && c1 != c2
            && c1 != d2
            && d1 != c2
            && d1 != d2
        }),
    );

    self.iter.insert_dataflow(
      &self.board,
      self.iter.product(&self.two_rows, &self.two_rows).filter(
        |(((a1, b1, c1, d1), (a2, b2, c2, d2)), ((a3, b3, c3, d3), (a4, b4, c4, d4)))| {
          a1 != a3
            && a1 != a4
            && a2 != a3
            && a2 != a4
            && b1 != b3
            && b1 != b4
            && b2 != b3
            && b2 != b4
            && c1 != c3
            && c1 != c4
            && c2 != c3
            && c2 != c4
            && d1 != d3
            && d1 != d4
            && d2 != d3
            && d2 != d4
        },
      ),
    )
  }
}

fn main() {
  let mut prog = Sudoku4x4::<()>::new();

  prog.run();

  let result = prog.iter.complete(&prog.board);
  for elem in result.iter() {
    println!("{:?}", elem);
  }
}
