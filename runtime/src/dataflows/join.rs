use std::cmp::Ordering;
use std::marker::PhantomData;

use super::utils::*;
use super::*;
use crate::*;

pub fn join<'b, D1, D2, K, T1, T2, Tag>(
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
) -> Join<'b, D1, D2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<(K, T1), Tag>,
  D2: Dataflow<(K, T2), Tag>,
{
  Join {
    d1,
    d2,
    semiring_ctx,
    phantom: PhantomData,
  }
}

pub struct Join<'b, D1, D2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<(K, T1), Tag>,
  D2: Dataflow<(K, T2), Tag>,
{
  d1: D1,
  d2: D2,
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(K, T1, T2, Tag)>,
}

impl<'b, D1, D2, K, T1, T2, Tag> Clone for Join<'b, D1, D2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<(K, T1), Tag>,
  D2: Dataflow<(K, T2), Tag>,
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

impl<'b, D1, D2, K, T1, T2, Tag> Dataflow<(K, T1, T2), Tag> for Join<'b, D1, D2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<(K, T1), Tag>,
  D2: Dataflow<(K, T2), Tag>,
{
  type Stable = BatchesJoin<
    D1::Stable,
    D2::Stable,
    StableStableOp<'b, D1, D2, K, T1, T2, Tag>,
    (K, T1),
    (K, T2),
    (K, T1, T2),
    Tag,
  >;

  type Recent = BatchesChain3<
    BatchesJoin<
      D1::Recent,
      D2::Stable,
      RecentStableOp<'b, D1, D2, K, T1, T2, Tag>,
      (K, T1),
      (K, T2),
      (K, T1, T2),
      Tag,
    >,
    BatchesJoin<
      D1::Stable,
      D2::Recent,
      StableRecentOp<'b, D1, D2, K, T1, T2, Tag>,
      (K, T1),
      (K, T2),
      (K, T1, T2),
      Tag,
    >,
    BatchesJoin<
      D1::Recent,
      D2::Recent,
      RecentRecentOp<'b, D1, D2, K, T1, T2, Tag>,
      (K, T1),
      (K, T2),
      (K, T1, T2),
      Tag,
    >,
    (K, T1, T2),
    Tag,
  >;

  fn iter_stable(&self) -> Self::Stable {
    let op = JoinOp::new(self.semiring_ctx);
    Self::Stable::join(self.d1.iter_stable(), self.d2.iter_stable(), op)
  }

  fn iter_recent(self) -> Self::Recent {
    let d1_stable = self.d1.iter_stable();
    let d2_stable = self.d2.iter_stable();
    let d1_recent = self.d1.iter_recent();
    let d2_recent = self.d2.iter_recent();
    Self::Recent::chain_3(
      BatchesJoin::join(d1_recent.clone(), d2_stable, JoinOp::new(self.semiring_ctx)),
      BatchesJoin::join(d1_stable, d2_recent.clone(), JoinOp::new(self.semiring_ctx)),
      BatchesJoin::join(d1_recent, d2_recent, JoinOp::new(self.semiring_ctx)),
    )
  }
}

type StableStableOp<'b, D1, D2, K, T1, T2, Tag> = JoinOp<
  'b,
  <<D1 as Dataflow<(K, T1), Tag>>::Stable as Batches<(K, T1), Tag>>::Batch,
  <<D2 as Dataflow<(K, T2), Tag>>::Stable as Batches<(K, T2), Tag>>::Batch,
  K,
  T1,
  T2,
  Tag,
>;

type RecentStableOp<'b, D1, D2, K, T1, T2, Tag> = JoinOp<
  'b,
  <<D1 as Dataflow<(K, T1), Tag>>::Recent as Batches<(K, T1), Tag>>::Batch,
  <<D2 as Dataflow<(K, T2), Tag>>::Stable as Batches<(K, T2), Tag>>::Batch,
  K,
  T1,
  T2,
  Tag,
>;

type StableRecentOp<'b, D1, D2, K, T1, T2, Tag> = JoinOp<
  'b,
  <<D1 as Dataflow<(K, T1), Tag>>::Stable as Batches<(K, T1), Tag>>::Batch,
  <<D2 as Dataflow<(K, T2), Tag>>::Recent as Batches<(K, T2), Tag>>::Batch,
  K,
  T1,
  T2,
  Tag,
>;

type RecentRecentOp<'b, D1, D2, K, T1, T2, Tag> = JoinOp<
  'b,
  <<D1 as Dataflow<(K, T1), Tag>>::Recent as Batches<(K, T1), Tag>>::Batch,
  <<D2 as Dataflow<(K, T2), Tag>>::Recent as Batches<(K, T2), Tag>>::Batch,
  K,
  T1,
  T2,
  Tag,
>;

pub struct JoinOp<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  semiring_ctx: &'b Tag::Context,
  phantom: PhantomData<(I1, I2, K, T1, T2, Tag)>,
}

impl<'b, I1, I2, K, T1, T2, Tag> Clone for JoinOp<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  fn clone(&self) -> Self {
    Self {
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, K, T1, T2, Tag> JoinOp<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  pub fn new(semiring_ctx: &'b Tag::Context) -> Self {
    Self {
      semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'b, I1, I2, K, T1, T2, Tag> BatchBinaryOp<I1, I2> for JoinOp<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  type IOut = JoinIterator<'b, I1, I2, K, T1, T2, Tag>;

  fn apply(&self, mut i1: I1, mut i2: I2) -> Self::IOut {
    let i1_curr = i1.next();
    let i2_curr = i2.next();
    Self::IOut {
      i1,
      i2,
      i1_curr,
      i2_curr,
      curr_iter: None,
      semiring_ctx: self.semiring_ctx,
    }
  }
}

pub struct JoinIterator<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  i1: I1,
  i2: I2,
  i1_curr: Option<Element<(K, T1), Tag>>,
  i2_curr: Option<Element<(K, T2), Tag>>,
  curr_iter: Option<JoinProductIterator<K, T1, T2, Tag>>,
  semiring_ctx: &'b Tag::Context,
}

impl<'b, I1, I2, K, T1, T2, Tag> Clone for JoinIterator<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  fn clone(&self) -> Self {
    Self {
      i1: self.i1.clone(),
      i2: self.i2.clone(),
      i1_curr: self.i1_curr.clone(),
      i2_curr: self.i2_curr.clone(),
      curr_iter: self.curr_iter.clone(),
      semiring_ctx: self.semiring_ctx,
    }
  }
}

impl<'b, I1, I2, K, T1, T2, Tag> Iterator for JoinIterator<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
  type Item = Element<(K, T1, T2), Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      // First go through curr joint product iterator
      if let Some(curr_iter) = &mut self.curr_iter {
        if let Some((e1, e2)) = curr_iter.next() {
          let (k, t1) = e1.tup;
          let (_, t2) = e2.tup;
          let tup = (k, t1, t2);
          let tag = Tag::mult(self.semiring_ctx, &e1.tag, &e2.tag);
          let result = Element { tup, tag };
          return Some(result);
        } else {
          // Skip ahead
          self.i1.step(curr_iter.v1.len() - 1);
          self.i1_curr = self.i1.next();
          self.i2.step(curr_iter.v2.len() - 1);
          self.i2_curr = self.i2.next();

          // Remove current iterator
          self.curr_iter = None;
        }
      }

      // Then continue
      match (&self.i1_curr, &self.i2_curr) {
        (Some(i1_curr), Some(i2_curr)) => match i1_curr.tup.0.cmp(&i2_curr.tup.0) {
          Ordering::Less => {
            self.i1_curr = self.i1.search_ahead(|i1_next| i1_next.0 < i2_curr.tup.0)
          }
          Ordering::Equal => {
            let key = &i1_curr.tup.0;
            let v1 = std::iter::once(i1_curr.clone())
              .chain(self.i1.clone().take_while(|x| &x.tup.0 == key))
              .collect::<Vec<_>>();
            let v2 = std::iter::once(i2_curr.clone())
              .chain(self.i2.clone().take_while(|x| &x.tup.0 == key))
              .collect::<Vec<_>>();
            let iter = JoinProductIterator::new(v1, v2);
            self.curr_iter = Some(iter);
          }
          Ordering::Greater => {
            self.i2_curr = self.i2.search_ahead(|i2_next| i2_next.0 < i1_curr.tup.0)
          }
        },
        _ => break None,
      }
    }
  }
}

impl<'b, I1, I2, K, T1, T2, Tag> Batch<(K, T1, T2), Tag>
  for JoinIterator<'b, I1, I2, K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<(K, T1), Tag>,
  I2: Batch<(K, T2), Tag>,
{
}
