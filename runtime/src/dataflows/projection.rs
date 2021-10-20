use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn project<S, F, T1, T2, Tag>(source: S, map_fn: F) -> Projection<S, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  S: Dataflow<T1, Tag>,
  F: Fn(T1) -> T2,
{
  Projection {
    source,
    map_fn,
    phantom: PhantomData,
  }
}

pub trait ProjectionOnDataflow<S, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  S: Dataflow<T1, Tag>,
  F: Fn(T1) -> T2,
{
  fn project(self, map_fn: F) -> Projection<S, F, T1, T2, Tag>;
}

impl<S, F, T1, T2, Tag> ProjectionOnDataflow<S, F, T1, T2, Tag> for S
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  S: Dataflow<T1, Tag>,
  F: Fn(T1) -> T2,
{
  fn project(self, map_fn: F) -> Projection<S, F, T1, T2, Tag> {
    project(self, map_fn)
  }
}

#[derive(Clone)]
pub struct Projection<S, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  S: Dataflow<T1, Tag>,
  F: Fn(T1) -> T2,
{
  source: S,
  map_fn: F,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<S, F, T1, T2, Tag> Dataflow<T2, Tag> for Projection<S, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  S: Dataflow<T1, Tag>,
  F: Fn(T1) -> T2 + Clone,
{
  type Stable = BatchesMap<S::Stable, ProjectOp<F, T1, T2, Tag>, T1, T2, Tag>;

  type Recent = BatchesMap<S::Recent, ProjectOp<F, T1, T2, Tag>, T1, T2, Tag>;

  fn iter_stable(&self) -> Self::Stable {
    let op = ProjectOp::new(self.map_fn.clone());
    Self::Stable::new(self.source.iter_stable(), op)
  }

  fn iter_recent(self) -> Self::Recent {
    let op = ProjectOp::new(self.map_fn.clone());
    Self::Recent::new(self.source.iter_recent(), op)
  }
}

#[derive(Clone)]
pub struct ProjectOp<F, T1, T2, Tag>
where
  F: Fn(T1) -> T2 + Clone,
{
  map_fn: F,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<F, T1, T2, Tag> ProjectOp<F, T1, T2, Tag>
where
  F: Fn(T1) -> T2 + Clone,
{
  pub fn new(map_fn: F) -> Self {
    Self {
      map_fn,
      phantom: PhantomData,
    }
  }
}

impl<I1, F, T1, T2, Tag> BatchUnaryOp<I1> for ProjectOp<F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I1: Batch<T1, Tag>,
  F: Fn(T1) -> T2 + Clone,
{
  type I2 = ProjectionIterator<I1, F, T1, T2, Tag>;

  fn apply(&self, i1: I1) -> Self::I2 {
    Self::I2 {
      source_iter: i1,
      map_fn: self.map_fn.clone(),
      phantom: PhantomData,
    }
  }
}

#[derive(Clone)]
pub struct ProjectionIterator<I, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<T1, Tag>,
  F: Fn(T1) -> T2 + Clone,
{
  source_iter: I,
  map_fn: F,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<I, F, T1, T2, Tag> Iterator for ProjectionIterator<I, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<T1, Tag>,
  F: Fn(T1) -> T2 + Clone,
{
  type Item = Element<T2, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.source_iter.next() {
      Some(item) => Some(Element {
        tup: (self.map_fn)(item.tup),
        tag: item.tag,
      }),
      None => None,
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.source_iter.size_hint()
  }
}

impl<I, F, T1, T2, Tag> Batch<T2, Tag> for ProjectionIterator<I, F, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<T1, Tag>,
  F: Fn(T1) -> T2 + Clone,
{
}
