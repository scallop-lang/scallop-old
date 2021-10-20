use std::cmp::Ordering;
use std::marker::PhantomData;

use super::*;
use crate::*;

pub fn find<D, T1, T2, Tag>(source: D, key: T1) -> Find<D, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D: Dataflow<(T1, T2), Tag>,
{
  Find {
    source,
    key,
    phantom: PhantomData,
  }
}

pub trait FindOnDataflow<D, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D: Dataflow<(T1, T2), Tag>,
{
  fn find(self, key: T1) -> Find<D, T1, T2, Tag>;
}

impl<D, T1, T2, Tag> FindOnDataflow<D, T1, T2, Tag> for D
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D: Dataflow<(T1, T2), Tag>,
{
  fn find(self, key: T1) -> Find<D, T1, T2, Tag> {
    find(self, key)
  }
}

#[derive(Clone)]
pub struct Find<D, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D: Dataflow<(T1, T2), Tag>,
{
  source: D,
  key: T1,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<D, T1, T2, Tag> Dataflow<(T1, T2), Tag> for Find<D, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  D: Dataflow<(T1, T2), Tag>,
{
  type Stable = BatchesMap<D::Stable, FindOp<T1, T2, Tag>, (T1, T2), (T1, T2), Tag>;

  type Recent = BatchesMap<D::Recent, FindOp<T1, T2, Tag>, (T1, T2), (T1, T2), Tag>;

  fn iter_stable(&self) -> Self::Stable {
    let op = FindOp::new(self.key.clone());
    Self::Stable::new(self.source.iter_stable(), op)
  }

  fn iter_recent(self) -> Self::Recent {
    let op = FindOp::new(self.key.clone());
    Self::Recent::new(self.source.iter_recent(), op)
  }
}

#[derive(Clone)]
pub struct FindOp<T1, T2, Tag> {
  key: T1,
  phantom: PhantomData<(T2, Tag)>,
}

impl<T1, T2, Tag> FindOp<T1, T2, Tag> {
  pub fn new(key: T1) -> Self {
    Self {
      key,
      phantom: PhantomData,
    }
  }
}

impl<I, T1, T2, Tag> BatchUnaryOp<I> for FindOp<T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<(T1, T2), Tag>,
{
  type I2 = FindIterator<I, T1, T2, Tag>;

  fn apply(&self, mut i1: I) -> Self::I2 {
    let curr_elem = i1.next();
    Self::I2 {
      source_iter: i1,
      curr_elem,
      key: self.key.clone(),
      phantom: PhantomData,
    }
  }
}

#[derive(Clone)]
pub struct FindIterator<I, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<(T1, T2), Tag>,
{
  source_iter: I,
  curr_elem: Option<Element<(T1, T2), Tag>>,
  key: T1,
  phantom: PhantomData<(T2, Tag)>,
}

impl<I, T1, T2, Tag> Iterator for FindIterator<I, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<(T1, T2), Tag>,
{
  type Item = Element<(T1, T2), Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    let key = self.key.clone();
    loop {
      match &self.curr_elem {
        Some(curr_elem) => match curr_elem.tup.0.cmp(&self.key) {
          Ordering::Less => {
            self.curr_elem = self.source_iter.search_ahead(|x| x.0 < key);
          }
          Ordering::Equal => {
            let result = curr_elem.clone();
            self.curr_elem = self.source_iter.next();
            return Some(result);
          }
          Ordering::Greater => return None,
        },
        None => return None,
      }
    }
  }
}

impl<I, T1, T2, Tag> Batch<(T1, T2), Tag> for FindIterator<I, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  I: Batch<(T1, T2), Tag>,
{
}
