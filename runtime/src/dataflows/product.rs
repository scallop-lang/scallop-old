use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn product<'b, D1, D2, T1, T2, Tag>(
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
) -> Product<'b, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  Product {
    d1,
    d2,
    semiring_ctx,
    phantom: PhantomData,
  }
}

pub struct Product<'b, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<'b, D1, D2, T1, T2, Tag> Clone for Product<'b, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
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

impl<'b, D1, D2, T1, T2, Tag> Dataflow<(T1, T2), Tag> for Product<'b, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  type Stable = BatchesJoin<
    D1::Stable,
    D2::Stable,
    StableStableOp<'b, D1, D2, T1, T2, Tag>,
    T1,
    T2,
    (T1, T2),
    Tag,
  >;

  type Recent = BatchesChain3<
    BatchesJoin<
      D1::Recent,
      D2::Stable,
      RecentStableOp<'b, D1, D2, T1, T2, Tag>,
      T1,
      T2,
      (T1, T2),
      Tag,
    >,
    BatchesJoin<
      D1::Stable,
      D2::Recent,
      StableRecentOp<'b, D1, D2, T1, T2, Tag>,
      T1,
      T2,
      (T1, T2),
      Tag,
    >,
    BatchesJoin<
      D1::Recent,
      D2::Recent,
      RecentRecentOp<'b, D1, D2, T1, T2, Tag>,
      T1,
      T2,
      (T1, T2),
      Tag,
    >,
    (T1, T2),
    Tag,
  >;

  fn iter_stable(&self) -> Self::Stable {
    let op = ProductOp::new(self.semiring_ctx);
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
        ProductOp::new(self.semiring_ctx),
      ),
      BatchesJoin::join(
        d1_stable,
        d2_recent.clone(),
        ProductOp::new(self.semiring_ctx),
      ),
      BatchesJoin::join(d1_recent, d2_recent, ProductOp::new(self.semiring_ctx)),
    )
  }
}

type StableStableOp<'b, D1, D2, T1, T2, Tag> = ProductOp<
  'b,
  <<D1 as Dataflow<T1, Tag>>::Stable as Batches<T1, Tag>>::Batch,
  <<D2 as Dataflow<T2, Tag>>::Stable as Batches<T2, Tag>>::Batch,
  T1,
  T2,
  Tag,
>;

type RecentStableOp<'b, D1, D2, T1, T2, Tag> = ProductOp<
  'b,
  <<D1 as Dataflow<T1, Tag>>::Recent as Batches<T1, Tag>>::Batch,
  <<D2 as Dataflow<T2, Tag>>::Stable as Batches<T2, Tag>>::Batch,
  T1,
  T2,
  Tag,
>;

type StableRecentOp<'b, D1, D2, T1, T2, Tag> = ProductOp<
  'b,
  <<D1 as Dataflow<T1, Tag>>::Stable as Batches<T1, Tag>>::Batch,
  <<D2 as Dataflow<T2, Tag>>::Recent as Batches<T2, Tag>>::Batch,
  T1,
  T2,
  Tag,
>;

type RecentRecentOp<'b, D1, D2, T1, T2, Tag> = ProductOp<
  'b,
  <<D1 as Dataflow<T1, Tag>>::Recent as Batches<T1, Tag>>::Batch,
  <<D2 as Dataflow<T2, Tag>>::Recent as Batches<T2, Tag>>::Batch,
  T1,
  T2,
  Tag,
>;

pub struct ProductOp<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(I1, I2, T1, T2, Tag)>,
}

impl<'b, I1, I2, T1, T2, Tag> Clone for ProductOp<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, T1, T2, Tag> ProductOp<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  pub fn new(semiring_ctx: &'b Tag::Context) -> Self {
    Self {
      semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, T1, T2, Tag> BatchBinaryOp<I1, I2> for ProductOp<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  type IOut = ProductIterator<'b, I1, I2, T1, T2, Tag>;

  fn apply(&self, mut i1: I1, i2: I2) -> Self::IOut {
    let i1_curr = i1.next();
    Self::IOut {
      i1,
      i1_curr,
      i2_source: i2.clone(),
      i2_clone: i2,
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

pub struct ProductIterator<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  i1: I1,
  i1_curr: Option<Element<T1, Tag>>,
  i2_source: I2,
  i2_clone: I2,
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(I1, I2, T1, T2)>,
}

impl<'b, I1, I2, T1, T2, Tag> Clone for ProductIterator<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      i1: self.i1.clone(),
      i1_curr: self.i1_curr.clone(),
      i2_source: self.i2_source.clone(),
      i2_clone: self.i2_clone.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, T1, T2, Tag> Iterator for ProductIterator<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
  type Item = Element<(T1, T2), Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match &self.i1_curr {
        Some(i1_curr) => match self.i2_clone.next() {
          Some(i2_curr) => {
            let tup = (i1_curr.tup.clone(), i2_curr.tup);
            let tag = Tag::mult(&self.semiring_ctx, &i1_curr.tag, &i2_curr.tag);
            let elem = Element { tup, tag };
            return Some(elem);
          }
          None => {
            self.i1_curr = self.i1.next();
            self.i2_clone = self.i2_source.clone();
          }
        },
        None => return None,
      }
    }
  }
}

impl<'b, I1, I2, T1, T2, Tag> Batch<(T1, T2), Tag> for ProductIterator<'b, I1, I2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  I2: Batch<T2, Tag>,
{
}
