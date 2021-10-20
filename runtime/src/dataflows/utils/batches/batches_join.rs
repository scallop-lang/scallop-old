use std::marker::PhantomData;

use super::*;
use crate::*;

#[derive(Clone)]
pub struct BatchesJoin<B1, B2, Op, T1, T2, TOut, Tag>
where
  T1: Tuple,
  T2: Tuple,
  TOut: Tuple,
  Tag: Semiring,
  Op: BatchBinaryOp<B1::Batch, B2::Batch>,
  Op::IOut: Batch<TOut, Tag>,
  B1: Batches<T1, Tag>,
  B2: Batches<T2, Tag>,
{
  b1: B1,
  b1_curr: Option<B1::Batch>,
  b2: B2,
  b2_source: B2,
  op: Op,
  phantom: PhantomData<(TOut, T2)>,
}

impl<B1, B2, Op, T1, T2, TOut, Tag> BatchesJoin<B1, B2, Op, T1, T2, TOut, Tag>
where
  T1: Tuple,
  T2: Tuple,
  TOut: Tuple,
  Tag: Semiring,
  Op: BatchBinaryOp<B1::Batch, B2::Batch>,
  Op::IOut: Batch<TOut, Tag>,
  B1: Batches<T1, Tag>,
  B2: Batches<T2, Tag>,
{
  pub fn join(mut b1: B1, b2: B2, op: Op) -> Self {
    let b1_curr = b1.next();
    let b2_source = b2.clone();
    Self {
      b1,
      b1_curr,
      b2,
      b2_source,
      op,
      phantom: PhantomData,
    }
  }
}

impl<B1, B2, Op, T1, T2, TOut, Tag> Iterator for BatchesJoin<B1, B2, Op, T1, T2, TOut, Tag>
where
  T1: Tuple,
  T2: Tuple,
  TOut: Tuple,
  Tag: Semiring,
  Op: BatchBinaryOp<B1::Batch, B2::Batch>,
  Op::IOut: Batch<TOut, Tag>,
  B1: Batches<T1, Tag>,
  B2: Batches<T2, Tag>,
{
  type Item = Op::IOut;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match &self.b1_curr {
        Some(b1_curr) => match self.b2.next() {
          Some(b2_curr) => {
            let result = self.op.apply(b1_curr.clone(), b2_curr);
            return Some(result);
          }
          None => {
            self.b1_curr = self.b1.next();
            self.b2 = self.b2_source.clone();
          }
        },
        None => return None,
      }
    }
  }
}

impl<B1, B2, Op, T1, T2, TOut, Tag> Batches<TOut, Tag>
  for BatchesJoin<B1, B2, Op, T1, T2, TOut, Tag>
where
  T1: Tuple,
  T2: Tuple,
  TOut: Tuple,
  Tag: Semiring,
  Op: BatchBinaryOp<B1::Batch, B2::Batch>,
  Op::IOut: Batch<TOut, Tag>,
  B1: Batches<T1, Tag>,
  B2: Batches<T2, Tag>,
{
  type Batch = Op::IOut;
}
