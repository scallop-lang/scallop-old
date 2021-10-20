use std::marker::PhantomData;

use super::*;
use crate::*;

#[derive(Clone)]
pub enum OptionalBatches<B, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B: Batches<Tup, Tag>,
{
  Some(B),
  None(PhantomData<(Tup, Tag)>),
}

impl<B, Tup, Tag> OptionalBatches<B, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B: Batches<Tup, Tag>,
{
  pub fn some(b: B) -> Self {
    Self::Some(b)
  }

  pub fn none() -> Self {
    Self::None(PhantomData)
  }
}

impl<B, Tup, Tag> Iterator for OptionalBatches<B, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B: Batches<Tup, Tag>,
{
  type Item = B::Batch;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Some(b) => b.next(),
      _ => None,
    }
  }
}

impl<B, Tup, Tag> Batches<Tup, Tag> for OptionalBatches<B, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B: Batches<Tup, Tag>,
{
  type Batch = B::Batch;
}
