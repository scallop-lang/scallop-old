use std::cmp::Ordering;
use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn intersect<'b, D1, D2, Tup, Tag>(
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
) -> Intersection<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  Intersection {
    d1,
    d2,
    semiring_ctx,
    phantom: PhantomData,
  }
}

pub struct Intersection<'b, D1, D2, Tup, Tag>
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

impl<'b, D1, D2, Tup, Tag> Clone for Intersection<'b, D1, D2, Tup, Tag>
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

impl<'b, D1, D2, Tup, Tag> Dataflow<Tup, Tag> for Intersection<'b, D1, D2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D1: Dataflow<Tup, Tag>,
  D2: Dataflow<Tup, Tag>,
{
  type Stable =
    BatchesJoin<D1::Stable, D2::Stable, StableStableOp<'b, D1, D2, Tup, Tag>, Tup, Tup, Tup, Tag>;

  type Recent = BatchesChain3<
    BatchesJoin<D1::Recent, D2::Stable, RecentStableOp<'b, D1, D2, Tup, Tag>, Tup, Tup, Tup, Tag>,
    BatchesJoin<D1::Stable, D2::Recent, StableRecentOp<'b, D1, D2, Tup, Tag>, Tup, Tup, Tup, Tag>,
    BatchesJoin<D1::Recent, D2::Recent, RecentRecentOp<'b, D1, D2, Tup, Tag>, Tup, Tup, Tup, Tag>,
    Tup,
    Tag,
  >;

  fn iter_stable(&self) -> Self::Stable {
    let op = IntersectionOp::new(self.semiring_ctx);
    Self::Stable::join(self.d1.iter_stable(), self.d2.iter_stable(), op)
  }

  fn iter_recent(self) -> Self::Recent {
    let d1_stable = self.d1.iter_stable();
    let d2_stable = self.d2.iter_stable();
    let d1_recent = self.d1.iter_recent();
    let d2_recent = self.d2.iter_recent();
    Self::Recent::chain_3(
      BatchesJoin::join(
        d1_recent.clone(),
        d2_stable,
        IntersectionOp::new(self.semiring_ctx),
      ),
      BatchesJoin::join(
        d1_stable,
        d2_recent.clone(),
        IntersectionOp::new(self.semiring_ctx),
      ),
      BatchesJoin::join(d1_recent, d2_recent, IntersectionOp::new(self.semiring_ctx)),
    )
  }
}

type StableStableOp<'b, D1, D2, Tup, Tag> = IntersectionOp<
  'b,
  <<D1 as Dataflow<Tup, Tag>>::Stable as Batches<Tup, Tag>>::Batch,
  <<D2 as Dataflow<Tup, Tag>>::Stable as Batches<Tup, Tag>>::Batch,
  Tup,
  Tag,
>;

type RecentStableOp<'b, D1, D2, Tup, Tag> = IntersectionOp<
  'b,
  <<D1 as Dataflow<Tup, Tag>>::Recent as Batches<Tup, Tag>>::Batch,
  <<D2 as Dataflow<Tup, Tag>>::Stable as Batches<Tup, Tag>>::Batch,
  Tup,
  Tag,
>;

type StableRecentOp<'b, D1, D2, Tup, Tag> = IntersectionOp<
  'b,
  <<D1 as Dataflow<Tup, Tag>>::Stable as Batches<Tup, Tag>>::Batch,
  <<D2 as Dataflow<Tup, Tag>>::Recent as Batches<Tup, Tag>>::Batch,
  Tup,
  Tag,
>;

type RecentRecentOp<'b, D1, D2, Tup, Tag> = IntersectionOp<
  'b,
  <<D1 as Dataflow<Tup, Tag>>::Recent as Batches<Tup, Tag>>::Batch,
  <<D2 as Dataflow<Tup, Tag>>::Recent as Batches<Tup, Tag>>::Batch,
  Tup,
  Tag,
>;

pub struct IntersectionOp<'a, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  semiring_ctx: &'a Tag::Context,
  phantom: PhantomData<(I1, I2, Tup, Tag)>,
}

impl<'a, I1, I2, Tup, Tag> Clone for IntersectionOp<'a, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, I1, I2, Tup, Tag> IntersectionOp<'a, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  pub fn new(semiring_ctx: &'a Tag::Context) -> Self {
    Self {
      semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, I1, I2, Tup, Tag> BatchBinaryOp<I1, I2> for IntersectionOp<'a, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  type IOut = IntersectionIterator<'a, I1, I2, Tup, Tag>;

  fn apply(&self, mut i1: I1, mut i2: I2) -> Self::IOut {
    let i1_curr = i1.next();
    let i2_curr = i2.next();
    IntersectionIterator {
      i1,
      i2,
      i1_curr,
      i2_curr,
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

pub struct IntersectionIterator<'b, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  i1: I1,
  i2: I2,
  i1_curr: Option<Element<Tup, Tag>>,
  i2_curr: Option<Element<Tup, Tag>>,
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(I1, I2, Tup)>,
}

impl<'b, I1, I2, Tup, Tag> Clone for IntersectionIterator<'b, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      i1: self.i1.clone(),
      i2: self.i2.clone(),
      i1_curr: self.i1_curr.clone(),
      i2_curr: self.i2_curr.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, Tup, Tag> Iterator for IntersectionIterator<'b, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
  type Item = Element<Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match (&self.i1_curr, &self.i2_curr) {
        (Some(i1_curr), Some(i2_curr)) => match i1_curr.tup.cmp(&i2_curr.tup) {
          Ordering::Less => {
            self.i1_curr = self.i1.search_ahead(|i1_next| i1_next < &i2_curr.tup);
          }
          Ordering::Equal => {
            let tag = Tag::mult(self.semiring_ctx, &i1_curr.tag, &i2_curr.tag);
            let result = Element {
              tup: i1_curr.tup.clone(),
              tag,
            };
            self.i1_curr = self.i1.next();
            self.i2_curr = self.i2.next();
            return Some(result);
          }
          Ordering::Greater => {
            self.i2_curr = self.i2.search_ahead(|i2_next| i2_next < &i1_curr.tup);
          }
        },
        _ => return None,
      }
    }
  }
}

impl<'b, I1, I2, Tup, Tag> Batch<Tup, Tag> for IntersectionIterator<'b, I1, I2, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  I2: Batch<Tup, Tag>,
{
}
