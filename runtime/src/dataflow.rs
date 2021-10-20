use super::*;
use super::dataflows::*;

/// A dataflow trait
///
/// A dataflow will be divided into stable parts and recent parts.
/// Each part returns a sequence of batches (as defined by `Batches`).
/// One batch will further be iterated and collected into individual
/// elements.
///
/// The sequence of recent batches represent the new elements that
/// we want to add into the system. This is as opposed to the sequence
/// of stable batches, which represents the elements that are already
/// inside of the system.
///
/// Inside a statically compiled dataflow, stable batches and recent
/// batches can have separate types. Any type that instantiates this
/// dataflow trait must provide a `Stable` type and a `Recent` type.
/// Henceforth, the `iter_stable` function will return the sequence
/// of stable batches, and the `iter_recent` function will return the
/// sequence of recent batches.
pub trait Dataflow<Tup, Tag>: Sized + Clone
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Stable: Batches<Tup, Tag>;

  type Recent: Batches<Tup, Tag>;

  fn iter_stable(&self) -> Self::Stable;

  fn iter_recent(self) -> Self::Recent;
}

/// A vector of elements can form a dataflow
///
/// It will not be producing any stable batches. It will be producing
/// one single batch which contains the elements inside the vector.
impl<Tup, Tag> Dataflow<Tup, Tag> for Vec<Element<Tup, Tag>>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Stable = EmptyBatches<std::iter::Empty<Element<Tup, Tag>>>;

  type Recent = SingletonBatch<std::vec::IntoIter<Element<Tup, Tag>>>;

  fn iter_stable(&self) -> Self::Stable {
    Self::Stable::default()
  }

  fn iter_recent(self) -> Self::Recent {
    Self::Recent::singleton(self.into_iter())
  }
}

/// A relation can form a dataflow
///
/// It will not produce any stable batch. It produces one single batch
/// which contains the elements inside the relation.
impl<Tup, Tag> Dataflow<Tup, Tag> for Relation<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Stable = EmptyBatches<std::iter::Empty<Element<Tup, Tag>>>;

  type Recent = SingletonBatch<std::vec::IntoIter<Element<Tup, Tag>>>;

  fn iter_stable(&self) -> Self::Stable {
    Self::Stable::default()
  }

  fn iter_recent(self) -> Self::Recent {
    Self::Recent::singleton(self.elements.into_iter())
  }
}
