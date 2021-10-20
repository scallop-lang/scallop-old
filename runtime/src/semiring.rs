use std::fmt::Debug;

pub trait Semiring: Clone + Sized + Debug + Send + Sync + 'static {
  type Context: SemiringContext<Self>;

  fn zero(ctx: &Self::Context) -> Self;

  fn one(ctx: &Self::Context) -> Self;

  fn add(ctx: &Self::Context, t1: &Self, t2: &Self) -> Self;

  fn mult(ctx: &Self::Context, t1: &Self, t2: &Self) -> Self;

  fn is_valid(&self, ctx: &Self::Context) -> bool;
}

pub trait SemiringWithDifference: Semiring {
  fn minus(ctx: &Self::Context, t1: &Self, t2: &Self) -> Self;
}

pub trait SemiringContext<Tag>: Default + Sync {
  type Info;

  fn base_tag(&mut self, info: Self::Info) -> Tag;
}
