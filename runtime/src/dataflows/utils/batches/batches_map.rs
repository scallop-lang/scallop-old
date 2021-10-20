use std::marker::PhantomData;

use super::*;
use crate::*;

#[derive(Clone)]
pub struct BatchesMap<B, Op, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  Op: BatchUnaryOp<B::Batch>,
  Op::I2: Batch<T2, Tag>,
  B: Batches<T1, Tag>,
{
  source: B,
  op: Op,
  phantom: PhantomData<(T1, T2, Tag)>,
}

impl<B, Op, T1, T2, Tag> BatchesMap<B, Op, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  Op: BatchUnaryOp<B::Batch>,
  Op::I2: Batch<T2, Tag>,
  B: Batches<T1, Tag>,
{
  pub fn new(source: B, op: Op) -> Self {
    Self {
      source,
      op,
      phantom: PhantomData,
    }
  }
}

impl<B, Op, T1, T2, Tag> Iterator for BatchesMap<B, Op, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  Op: BatchUnaryOp<B::Batch>,
  B: Batches<T1, Tag>,
  Op::I2: Batch<T2, Tag>,
{
  type Item = Op::I2;

  fn next(&mut self) -> Option<Self::Item> {
    self.source.next().map(|batch| self.op.apply(batch))
  }
}

impl<B, Op, T1, T2, Tag> Batches<T2, Tag> for BatchesMap<B, Op, T1, T2, Tag>
where
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
  Op: BatchUnaryOp<B::Batch>,
  B: Batches<T1, Tag>,
  Op::I2: Batch<T2, Tag>,
{
  type Batch = Op::I2;
}
