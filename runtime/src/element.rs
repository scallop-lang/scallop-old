use std::cmp::Ordering;

use super::semiring::*;
use super::*;

/// A static element
///
/// `Tup` is a tuple type storing inside the element;
/// `Tag` is the tag associated with this tuple.
#[derive(Clone, Debug)]
pub struct Element<Tup: Tuple, Tag: Semiring = ()> {
  pub tup: Tup,
  pub tag: Tag,
}

/// Element implements PartialOrd; only the tuple will be used
/// to perform the comparing
impl<Tup, Tag> PartialOrd for Element<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.tup.partial_cmp(&other.tup)
  }
}

/// Element implements Ord; only the tuple will be used to perform
/// the comparing
impl<Tup, Tag> Ord for Element<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn cmp(&self, other: &Self) -> Ordering {
    self.tup.cmp(&other.tup)
  }
}

/// Element implements PartialEq; it will only compare the two tuples.
impl<Tup, Tag> PartialEq for Element<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn eq(&self, other: &Self) -> bool {
    self.tup == other.tup
  }
}

/// Element implmeents Eq.
impl<Tup, Tag> Eq for Element<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
}
