use std::iter::FromIterator;

use super::*;
use super::super::batch::*;
use crate::*;

#[derive(Clone)]
pub struct SingletonBatch<I>
where
  I: Iterator + Clone,
{
  batch: Option<I>,
}

impl<I> SingletonBatch<I>
where
  I: Iterator + Clone,
{
  pub fn singleton(i: I) -> Self {
    Self { batch: Some(i) }
  }

  pub fn empty() -> Self {
    Self { batch: None }
  }
}

impl<I> Iterator for SingletonBatch<I>
where
  I: Iterator + Clone,
{
  type Item = I;

  fn next(&mut self) -> Option<I> {
    if self.batch.is_some() {
      let mut result = None;
      std::mem::swap(&mut result, &mut self.batch);
      result
    } else {
      None
    }
  }
}

impl<I> FromIterator<I> for SingletonBatch<I>
where
  I: Iterator + Clone,
{
  fn from_iter<Iter: IntoIterator<Item = I>>(i: Iter) -> Self {
    match i.into_iter().next() {
      Some(batch) => Self::singleton(batch),
      None => Self::empty(),
    }
  }
}

impl<I, Tup, Tag> Batches<Tup, Tag> for SingletonBatch<I>
where
  I: Batch<Tup, Tag>,
  Tup: Tuple,
  Tag: Semiring,
{
  type Batch = I;
}
