pub use scallop_runtime::*;
use scallop_codegen::scallop;
use tch::{nn, Tensor, index::*, Kind};

#[derive(Debug)]
pub struct MnistNet {
  conv1: nn::Conv2D,
  conv2: nn::Conv2D,
  fc1: nn::Linear,
  fc2: nn::Linear,
}

impl MnistNet {
  pub fn new(vs: &nn::Path) -> Self {
    let conv1 = nn::conv2d(vs, 1, 32, 5, Default::default());
    let conv2 = nn::conv2d(vs, 32, 64, 5, Default::default());
    let fc1 = nn::linear(vs, 1024, 1024, Default::default());
    let fc2 = nn::linear(vs, 1024, 10, Default::default());
    Self {
      conv1,
      conv2,
      fc1,
      fc2,
    }
  }
}

impl nn::ModuleT for MnistNet {
  fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
    xs.view([-1, 1, 28, 28])
      .apply(&self.conv1)
      .max_pool2d_default(2)
      .apply(&self.conv2)
      .max_pool2d_default(2)
      .view([-1, 1024])
      .apply(&self.fc1)
      .relu()
      .dropout_(0.5, train)
      .apply(&self.fc2)
      .softmax(1, Kind::Float)
  }
}

pub fn to_digit_disjunction(
  tensor: &Tensor,
  symbol: usize,
  option_num: i64,
) -> std::vec::Vec<(scallop_runtime::DualNumber, (usize, i64))> {
  (0..option_num).map(|i| {
    let ith = tensor.i(i);
    let dn = DualNumber::new(ith);
    let tup = (symbol, i as i64);
    (dn, tup)
  }).collect::<Vec<_>>()
}

scallop! {
  Sum2 {
    decl digit(Symbol, Int).
    decl sum_2(Int).

    sum_2(DA + DB) :- digit(0, DA), digit(1, DB).
  }
}

pub use scallop_sum2::Sum2;

scallop! {
  Sum3 {
    decl digit(Symbol, Int).
    decl sum_3(Int).

    sum_3(DA + DB + DC) :- digit(0, DA), digit(1, DB), digit(2, DC).
  }
}

pub use scallop_sum3::Sum3;

scallop! {
  Sum4 {
    decl digit(Symbol, Int).
    decl sum_4(Int).

    sum_4(DA + DB + DC + DD) :- digit(0, DA), digit(1, DB), digit(2, DC), digit(3, DD).
  }
}

pub use scallop_sum4::Sum4;

scallop! {
  Sort2 {
    decl digit(Symbol, Int).
    decl sort_2(Int).

    sort_2(0) :- digit(0, DA), digit(1, DB), DA <= DB.
    sort_2(1) :- digit(0, DA), digit(1, DB), DA > DB.
  }
}

pub use scallop_sort2::Sort2;

scallop! {
  CountCorrect2 {
    decl digit(Symbol, Int).
    decl gt(Int, Int).
    decl cc_2(Symbol).

    cc_2(0) :- digit(0, DA), digit(1, DB), gt(GA, GB), GA != DA, GB != DB.
    cc_2(1) :- digit(0, DA), digit(1, DB), gt(GA, GB), GA == DA, GB != DB.
    cc_2(1) :- digit(0, DA), digit(1, DB), gt(GA, GB), GA != DA, GB == DB.
    cc_2(2) :- digit(0, DA), digit(1, DB), gt(GA, GB), GA == DA, GB == DB.
  }
}

pub use scallop_countcorrect2::CountCorrect2;

scallop! {
  CountCorrect3 {
    decl digit(Symbol, Int).
    decl gt(Int, Int, Int).
    decl cc_3(Symbol).
    decl intermediate(Int, Int, Int, Int, Int, Int).

    intermediate(DA, DB, DC, GA, GB, GC) :- digit(0, DA), digit(1, DB), digit(2, DC), gt(GA, GB, GC).

    cc_3(0) :- intermediate(DA, DB, DC, GA, GB, GC), GA != DA, GB != DB, GC != DC.
    cc_3(1) :- intermediate(DA, DB, DC, GA, GB, GC), GA == DA, GB != DB, GC != DC.
    cc_3(1) :- intermediate(DA, DB, DC, GA, GB, GC), GA != DA, GB == DB, GC != DC.
    cc_3(1) :- intermediate(DA, DB, DC, GA, GB, GC), GA != DA, GB != DB, GC == DC.
    cc_3(2) :- intermediate(DA, DB, DC, GA, GB, GC), GA != DA, GB == DB, GC == DC.
    cc_3(2) :- intermediate(DA, DB, DC, GA, GB, GC), GA == DA, GB != DB, GC == DC.
    cc_3(2) :- intermediate(DA, DB, DC, GA, GB, GC), GA == DA, GB == DB, GC != DC.
    cc_3(3) :- intermediate(DA, DB, DC, GA, GB, GC), GA == DA, GB == DB, GC == DC.
  }
}

pub use scallop_countcorrect3::CountCorrect3;

scallop! {
  Sort3 {
    decl digit(Symbol, Int).
    decl sort_3(Int).
    decl digit_abc(Int, Int, Int).

    digit_abc(DA, DB, DC) :- digit(0, DA), digit(1, DB), digit(2, DC).

    sort_3(0) :- digit_abc(DA, DB, DC), DA <= DB, DB <= DC.
    sort_3(1) :- digit_abc(DA, DB, DC), DA <= DC, DC < DB.
    sort_3(2) :- digit_abc(DA, DB, DC), DB < DA, DA <= DC.
    sort_3(3) :- digit_abc(DA, DB, DC), DB <= DC, DC < DA.
    sort_3(4) :- digit_abc(DA, DB, DC), DC < DA, DA <= DB.
    sort_3(5) :- digit_abc(DA, DB, DC), DC < DB, DB < DA.
  }
}

pub use scallop_sort3::Sort3;

scallop! {
  Sort4 {
    decl digit(Symbol, Int).
    decl sort_4(Int).
    decl digits(Int, Int, Int, Int).

    digits(D0, D1, D2, D3) :- digit(0, D0), digit(1, D1), digit(2, D2), digit(3, D3).

    sort_4(0) :- digits(D0, D1, D2, D3), D0 <= D1, D1 <= D2, D2 <= D3. // 0, 1, 2, 3
    sort_4(1) :- digits(D0, D1, D2, D3), D0 <= D1, D1 <= D3, D3 < D2. // 0, 1, 3, 2
    sort_4(2) :- digits(D0, D1, D2, D3), D0 <= D2, D2 < D1, D1 <= D3. // 0, 2, 1, 3
    sort_4(3) :- digits(D0, D1, D2, D3), D0 <= D2, D2 <= D3, D3 < D1. // 0, 2, 3, 1
    sort_4(4) :- digits(D0, D1, D2, D3), D0 <= D3, D3 < D1, D1 <= D2. // 0, 3, 1, 2
    sort_4(5) :- digits(D0, D1, D2, D3), D0 <= D3, D3 < D2, D2 < D1. // 0, 3, 2, 1

    sort_4(6) :- digits(D0, D1, D2, D3), D1 < D0, D0 <= D2, D2 <= D3. // 1, 0, 2, 3
    sort_4(7) :- digits(D0, D1, D2, D3), D1 < D0, D0 <= D3, D3 < D2. // 1, 0, 3, 2
    sort_4(8) :- digits(D0, D1, D2, D3), D1 <= D2, D2 < D0, D0 <= D3. // 1, 2, 0, 3
    sort_4(9) :- digits(D0, D1, D2, D3), D1 <= D2, D2 <= D3, D3 < D0. // 1, 2, 3, 0
    sort_4(10) :- digits(D0, D1, D2, D3), D1 <= D3, D3 < D0, D0 <= D2. // 1, 3, 0, 2
    sort_4(11) :- digits(D0, D1, D2, D3), D1 <= D3, D3 < D2, D2 < D0. // 1, 3, 2, 0

    sort_4(12) :- digits(D0, D1, D2, D3), D2 < D0, D0 <= D1, D1 <= D3. // 2, 0, 1, 3
    sort_4(13) :- digits(D0, D1, D2, D3), D2 < D0, D0 <= D3, D3 < D1. // 2, 0, 3, 1
    sort_4(14) :- digits(D0, D1, D2, D3), D2 < D1, D1 < D0, D0 <= D3. // 2, 1, 0, 3
    sort_4(15) :- digits(D0, D1, D2, D3), D2 < D1, D1 <= D3, D3 < D0. // 2, 1, 3, 0
    sort_4(16) :- digits(D0, D1, D2, D3), D2 <= D3, D3 < D0, D0 <= D1. // 2, 3, 0, 1
    sort_4(17) :- digits(D0, D1, D2, D3), D2 <= D3, D3 < D1, D1 < D0. // 2, 3, 1, 0

    sort_4(18) :- digits(D0, D1, D2, D3), D3 < D0, D0 <= D1, D1 <= D2. // 3, 0, 1, 2
    sort_4(19) :- digits(D0, D1, D2, D3), D3 < D0, D0 <= D2, D2 < D1. // 3, 0, 2, 1
    sort_4(20) :- digits(D0, D1, D2, D3), D3 < D1, D1 < D0, D0 <= D2. // 3, 1, 0, 2
    sort_4(21) :- digits(D0, D1, D2, D3), D3 < D1, D1 <= D2, D2 < D0. // 3, 1, 2, 0
    sort_4(22) :- digits(D0, D1, D2, D3), D3 < D2, D2 < D0, D0 <= D1. // 3, 2, 0, 1
    sort_4(23) :- digits(D0, D1, D2, D3), D3 < D2, D2 < D1, D1 < D0. // 3, 2, 1, 0
  }
}

pub use scallop_sort4::Sort4;
