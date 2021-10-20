use crate::*;

pub trait WeightedModelCounter: Send + Sync {
  type Tag: Semiring;

  type Output: Send;

  fn wmc(
    &self,
    ctx: &<Self::Tag as Semiring>::Context,
    tag: &Self::Tag
  ) -> Self::Output;
}
