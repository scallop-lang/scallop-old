use std::marker::PhantomData;

use crate::*;
use super::*;

#[derive(Clone)]
pub enum EitherBatch<I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  First(I1, PhantomData<(Tup, Tag)>),
  Second(I2, PhantomData<(Tup, Tag)>),
}

impl<I1, I2, Tup, Tag> EitherBatch<I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  pub fn first(i1: I1) -> Self {
    Self::First(i1, PhantomData)
  }

  pub fn second(i2: I2) -> Self {
    Self::Second(i2, PhantomData)
  }
}

impl<I1, I2, Tup, Tag> Iterator for EitherBatch<I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  type Item = Element<Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::First(i1, _) => i1.next(),
      Self::Second(i2, _) => i2.next(),
    }
  }
}

impl<I1, I2, Tup, Tag> Batch<Tup, Tag> for EitherBatch<I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
}
