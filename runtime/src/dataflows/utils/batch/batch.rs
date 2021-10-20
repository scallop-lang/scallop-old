use crate::*;

pub trait Batch<Tup, Tag>: Iterator<Item = Element<Tup, Tag>> + Clone
where
  Tup: Tuple,
  Tag: Semiring,
{
  /// Step u steps
  fn step(&mut self, u: usize) {
    for _ in 0..u {
      self.next();
    }
  }

  /// Search until the given comparison function returns true on a given element
  fn search_ahead<F>(&mut self, _: F) -> Option<Element<Tup, Tag>>
  where
    F: FnMut(&Tup) -> bool,
  {
    self.next()
  }
}

impl<Tup, Tag> Batch<Tup, Tag> for std::iter::Empty<Element<Tup, Tag>>
where
  Tup: Tuple,
  Tag: Semiring,
{
}

impl<Tup, Tag> Batch<Tup, Tag> for std::vec::IntoIter<Element<Tup, Tag>>
where
  Tup: Tuple,
  Tag: Semiring,
{
}
