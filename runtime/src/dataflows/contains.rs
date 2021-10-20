use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn contains_chain<'a, D1, D2, T1, T2, Tag>(
  d1: D1,
  key: T1,
  d2: D2,
  semiring_ctx: &'a Tag::Context,
) -> ContainsChain<'a, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  ContainsChain {
    d1,
    key,
    d2,
    semiring_ctx,
    phantom: PhantomData,
  }
}

pub struct ContainsChain<'a, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  d1: D1,
  key: T1,
  d2: D2,
  semiring_ctx: &'a Tag::Context,
  phantom: PhantomData<(T2, Tag)>,
}

impl<'a, D1, D2, T1, T2, Tag> Clone for ContainsChain<'a, D1, D2, T1, T2, Tag>
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
      key: self.key.clone(),
      d2: self.d2.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, D1, D2, T1, T2, Tag> Dataflow<T2, Tag> for ContainsChain<'a, D1, D2, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D1: Dataflow<T1, Tag>,
  D2: Dataflow<T2, Tag>,
{
  type Stable =
    OptionalBatches<BatchesMap<D2::Stable, MergeTagOp<'a, T2, Tag>, T2, T2, Tag>, T2, Tag>;

  type Recent = BatchesChain<
    OptionalBatches<BatchesMap<D2::Stable, MergeTagOp<'a, T2, Tag>, T2, T2, Tag>, T2, Tag>,
    OptionalBatches<BatchesMap<D2::Recent, MergeTagOp<'a, T2, Tag>, T2, T2, Tag>, T2, Tag>,
    T2,
    Tag,
  >;

  fn iter_stable(&self) -> Self::Stable {
    for batch in self.d1.iter_stable() {
      if let Some(tag) = search(batch, &self.key) {
        let op = MergeTagOp::new(tag.clone(), self.semiring_ctx);
        return Self::Stable::some(BatchesMap::new(self.d2.iter_stable(), op));
      }
    }
    Self::Stable::none()
  }

  fn iter_recent(self) -> Self::Recent {
    for batch in self.d1.clone().iter_recent() {
      if let Some(tag) = search(batch, &self.key) {
        let op = MergeTagOp::new(tag.clone(), self.semiring_ctx);
        let d2_stable = self.d2.iter_stable();
        let stable = OptionalBatches::some(BatchesMap::new(d2_stable, op.clone()));
        let recent = OptionalBatches::some(BatchesMap::new(self.d2.iter_recent(), op));
        return Self::Recent::chain(stable, recent);
      }
    }
    for batch in self.d1.iter_stable() {
      if let Some(tag) = search(batch, &self.key) {
        let op = MergeTagOp::new(tag.clone(), self.semiring_ctx);
        return Self::Recent::chain(
          OptionalBatches::none(),
          OptionalBatches::some(BatchesMap::new(self.d2.iter_recent(), op)),
        );
      }
    }
    return Self::Recent::chain(OptionalBatches::none(), OptionalBatches::none());
  }
}

fn search<B, T, Tag>(mut batch: B, key: &T) -> Option<Tag>
where
  T: Tuple,
  Tag: Semiring,
  B: Batch<T, Tag>,
{
  if let Some(curr) = batch.next() {
    if &curr.tup == key {
      return Some(curr.tag.clone());
    }
  } else {
    return None;
  }
  while let Some(curr) = batch.search_ahead(|i| i < key) {
    if &curr.tup == key {
      return Some(curr.tag.clone());
    }
  }
  return None;
}

pub struct MergeTagOp<'a, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
{
  tag: Tag,
  semiring_ctx: &'a Tag::Context,
  phantom: PhantomData<T>,
}

impl<'a, T, Tag> Clone for MergeTagOp<'a, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
{
  fn clone(&self) -> Self {
    Self {
      tag: self.tag.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, T, Tag> MergeTagOp<'a, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
{
  pub fn new(tag: Tag, semiring_ctx: &'a Tag::Context) -> Self {
    Self {
      tag,
      semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, I, T, Tag> BatchUnaryOp<I> for MergeTagOp<'a, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
  I: Batch<T, Tag>,
{
  type I2 = MergeTagIterator<'a, I, T, Tag>;

  fn apply(&self, i1: I) -> Self::I2 {
    Self::I2 {
      source_iter: i1,
      tag: self.tag.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

pub struct MergeTagIterator<'a, I, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
  I: Batch<T, Tag>,
{
  source_iter: I,
  tag: Tag,
  semiring_ctx: &'a Tag::Context,
  phantom: PhantomData<T>,
}

impl<'a, I, T, Tag> Clone for MergeTagIterator<'a, I, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
  I: Batch<T, Tag>,
{
  fn clone(&self) -> Self {
    Self {
      source_iter: self.source_iter.clone(),
      tag: self.tag.clone(),
      semiring_ctx: self.semiring_ctx,
      phantom: PhantomData,
    }
  }
}

impl<'a, I, T, Tag> Iterator for MergeTagIterator<'a, I, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
  I: Batch<T, Tag>,
{
  type Item = Element<T, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.source_iter.next() {
      Some(item) => {
        let merged = Element {
          tup: item.tup,
          tag: Tag::mult(self.semiring_ctx, &self.tag, &item.tag),
        };
        Some(merged)
      }
      None => None,
    }
  }
}

impl<'a, I, T, Tag> Batch<T, Tag> for MergeTagIterator<'a, I, T, Tag>
where
  T: Tuple,
  Tag: Semiring,
  I: Batch<T, Tag>,
{
}
