use crate::semiring::*;

#[derive(Default, Clone)]
pub struct BoolSemiringContext;

impl Semiring for bool {
  type Context = BoolSemiringContext;

  #[inline(always)]
  fn zero(_: &Self::Context) -> Self {
    false
  }

  #[inline(always)]
  fn one(_: &Self::Context) -> Self {
    true
  }

  #[inline(always)]
  fn add(_: &Self::Context, b1: &Self, b2: &Self) -> Self {
    *b1 || *b2
  }

  #[inline(always)]
  fn mult(_: &Self::Context, b1: &Self, b2: &Self) -> Self {
    *b1 && *b2
  }

  #[inline(always)]
  fn is_valid(&self, _: &Self::Context) -> bool {
    *self
  }
}

impl SemiringWithDifference for bool {
  /// true - false = true
  /// true - true = false
  /// false - false = false
  ///
  /// TODO: Need to refind this
  fn minus(_: &Self::Context, _: &Self, b2: &Self) -> Self {
    !b2
  }
}

impl SemiringContext<bool> for BoolSemiringContext {
  type Info = ();

  #[inline(always)]
  fn base_tag(&mut self, (): Self::Info) -> bool {
    true
  }
}
