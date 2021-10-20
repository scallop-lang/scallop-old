use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn filter<S, F, Tup, Tag>(source: S, filter_fn: F) -> Filter<S, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  S: Dataflow<Tup, Tag>,
  F: Fn(&Tup) -> bool,
{
  Filter {
    source,
    filter_fn,
    phantom: PhantomData,
  }
}

pub trait FilterOnDataflow<S, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  S: Dataflow<Tup, Tag>,
  F: Fn(&Tup) -> bool,
{
  fn filter(self, filter_fn: F) -> Filter<S, F, Tup, Tag>;
}

impl<S, F, Tup, Tag> FilterOnDataflow<S, F, Tup, Tag> for S
where
  Tup: Tuple,
  Tag: Semiring,
  S: Dataflow<Tup, Tag>,
  F: Fn(&Tup) -> bool,
{
  fn filter(self, filter_fn: F) -> Filter<S, F, Tup, Tag> {
    filter(self, filter_fn)
  }
}

#[derive(Clone)]
pub struct Filter<S, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  S: Dataflow<Tup, Tag>,
  F: Fn(&Tup) -> bool,
{
  source: S,
  filter_fn: F,
  phantom: PhantomData<(Tup, Tag)>,
}

impl<S, F, Tup, Tag> Dataflow<Tup, Tag> for Filter<S, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  S: Dataflow<Tup, Tag>,
  F: Fn(&Tup) -> bool + Clone,
{
  type Stable = BatchesMap<S::Stable, FilterOp<F, Tup, Tag>, Tup, Tup, Tag>;

  type Recent = BatchesMap<S::Recent, FilterOp<F, Tup, Tag>, Tup, Tup, Tag>;

  fn iter_stable(&self) -> Self::Stable {
    let op = FilterOp::new(self.filter_fn.clone());
    Self::Stable::new(self.source.iter_stable(), op)
  }

  fn iter_recent(self) -> Self::Recent {
    let op = FilterOp::new(self.filter_fn);
    Self::Recent::new(self.source.iter_recent(), op)
  }
}

#[derive(Clone)]
pub struct FilterOp<F, Tup, Tag>
where
  F: Fn(&Tup) -> bool + Clone,
{
  filter_fn: F,
  phantom: PhantomData<(Tup, Tag)>,
}

impl<F, Tup, Tag> FilterOp<F, Tup, Tag>
where
  F: Fn(&Tup) -> bool + Clone,
{
  pub fn new(filter_fn: F) -> Self {
    Self {
      filter_fn,
      phantom: PhantomData,
    }
  }
}

impl<I1, F, Tup, Tag> BatchUnaryOp<I1> for FilterOp<F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I1: Batch<Tup, Tag>,
  F: Fn(&Tup) -> bool + Clone,
{
  type I2 = FilterIterator<I1, F, Tup, Tag>;

  fn apply(&self, i1: I1) -> Self::I2 {
    Self::I2 {
      source_iter: i1,
      filter_fn: self.filter_fn.clone(),
      phantom: PhantomData,
    }
  }
}

#[derive(Clone)]
pub struct FilterIterator<I, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I: Batch<Tup, Tag>,
  F: Fn(&Tup) -> bool + Clone,
{
  source_iter: I,
  filter_fn: F,
  phantom: PhantomData<(Tup, Tag)>,
}

impl<I, F, Tup, Tag> Iterator for FilterIterator<I, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I: Batch<Tup, Tag>,
  F: Fn(&Tup) -> bool + Clone,
{
  type Item = Element<Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.source_iter.next() {
        Some(item) => {
          if (self.filter_fn)(&item.tup) {
            return Some(item);
          }
        }
        None => return None,
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.source_iter.size_hint()
  }
}

impl<I, F, Tup, Tag> Batch<Tup, Tag> for FilterIterator<I, F, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  I: Batch<Tup, Tag>,
  F: Fn(&Tup) -> bool + Clone,
{
}
