use std::marker::PhantomData;

use super::*;
use crate::*;

#[derive(Clone)]
pub struct BatchesChain<B1, B2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B1: Batches<Tup, Tag>,
  B2: Batches<Tup, Tag>,
{
  b1: B1,
  b2: B2,
  use_b1: bool,
  phantom: PhantomData<(Tup, Tag)>,
}

impl<B1, B2, Tup, Tag> BatchesChain<B1, B2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B1: Batches<Tup, Tag>,
  B2: Batches<Tup, Tag>,
{
  pub fn chain(b1: B1, b2: B2) -> Self {
    Self {
      b1: b1,
      b2: b2,
      use_b1: true,
      phantom: PhantomData,
    }
  }
}

impl<B1, B2, Tup, Tag> Iterator for BatchesChain<B1, B2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B1: Batches<Tup, Tag>,
  B2: Batches<Tup, Tag>,
{
  type Item = EitherBatch<B1::Batch, B2::Batch, Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.use_b1 {
      if let Some(b1_curr) = self.b1.next() {
        return Some(EitherBatch::first(b1_curr));
      } else {
        self.use_b1 = false;
      }
    }
    self.b2.next().map(EitherBatch::second)
  }
}

impl<Tup, Tag, B1, B2> Batches<Tup, Tag> for BatchesChain<B1, B2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B1: Batches<Tup, Tag>,
  B2: Batches<Tup, Tag>,
{
  type Batch = EitherBatch<B1::Batch, B2::Batch, Tup, Tag>;
}

pub type BatchesChain3<B1, B2, B3, Tup, Tag> =
  BatchesChain<BatchesChain<B1, B2, Tup, Tag>, B3, Tup, Tag>;

impl<B1, B2, B3, Tup, Tag> BatchesChain3<B1, B2, B3, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  B1: Batches<Tup, Tag>,
  B2: Batches<Tup, Tag>,
  B3: Batches<Tup, Tag>,
{
  pub fn chain_3(b1: B1, b2: B2, b3: B3) -> Self {
    BatchesChain::chain(BatchesChain::chain(b1, b2), b3)
  }
}
