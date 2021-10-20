use crate::semiring::*;

#[derive(Clone, Default)]
pub struct UnitSemiringContext;

impl Semiring for () {
  type Context = UnitSemiringContext;

  fn zero(_: &Self::Context) -> Self {
    ()
  }

  fn one(_: &Self::Context) -> Self {
    ()
  }

  fn add(_: &Self::Context, _: &Self, _: &Self) -> Self {
    ()
  }

  fn mult(_: &Self::Context, _: &Self, _: &Self) -> Self {
    ()
  }

  fn is_valid(&self, _: &Self::Context) -> bool {
    true
  }
}

impl SemiringWithDifference for () {
  fn minus(_: &Self::Context, _: &Self, _: &Self) -> Self {
    ()
  }
}

impl SemiringContext<()> for UnitSemiringContext {
  type Info = ();

  fn base_tag(&mut self, _: Self::Info) -> () {
    ()
  }
}
