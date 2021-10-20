use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn union<'b, D1, D2, Tup, Tag>(
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
) -> Union<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  Union {
    d1,
    d2,
    semiring_ctx,
    phantom: PhantomData,
  }
}

pub struct Union<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(Tup, Tag)>,
}

impl<'b, D1, D2, Tup, Tag> Clone for Union<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      d1: self.d1.clone(),
      d2: self.d2.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, D1, D2, Tup, Tag> Dataflow<Tup, Tag> for Union<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  type Stable = BatchesChain<D1::Stable, D2::Stable, Tup, Tag>;

  type Recent = BatchesChain<D1::Recent, D2::Recent, Tup, Tag>;

  fn iter_stable(&self) -> Self::Stable {
    BatchesChain::chain(self.d1.iter_stable(), self.d2.iter_stable())
  }

  fn iter_recent(self) -> Self::Recent {
    BatchesChain::chain(self.d1.iter_recent(), self.d2.iter_recent())
  }
}
