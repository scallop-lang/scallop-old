use super::*;
use crate::*;

pub trait Batches<Tup, Tag>: Iterator<Item = Self::Batch> + Clone
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Batch: Batch<Tup, Tag>;
}
