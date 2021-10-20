use std::cell::Ref;

use super::*;
use crate::*;

impl<'a, Tup, Tag> Dataflow<Tup, Tag> for &'a Variable<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Stable = StableVariableBatches<'a, Tup, Tag>;

  type Recent = SingletonBatch<RelationIterator<'a, Tup, Tag>>;

  fn iter_stable(&self) -> Self::Stable {
    Self::Stable {
      relations: self.stable.borrow(),
      rela_id: 0,
    }
  }

  fn iter_recent(self) -> Self::Recent {
    Self::Recent::singleton(RelationIterator {
      relation: self.recent.borrow(),
      elem_id: 0,
    })
  }
}

pub struct StableVariableBatches<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  relations: Ref<'a, Vec<Relation<Tup, Tag>>>,
  rela_id: usize,
}

impl<'a, Tup, Tag> Clone for StableVariableBatches<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn clone(&self) -> Self {
    Self {
      relations: Ref::clone(&self.relations),
      rela_id: self.rela_id.clone(),
    }
  }
}

impl<'a, Tup, Tag> Iterator for StableVariableBatches<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Item = StableVariableBatch<'a, Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.rela_id < self.relations.len() {
      let result = Self::Item {
        relations: Ref::clone(&self.relations),
        rela_id: self.rela_id,
        elem_id: 0,
      };
      self.rela_id += 1;
      return Some(result);
    } else {
      return None;
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let size = self.relations.len();
    (size, Some(size))
  }
}

impl<'a, Tup, Tag> Batches<Tup, Tag> for StableVariableBatches<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Batch = StableVariableBatch<'a, Tup, Tag>;
}

pub struct StableVariableBatch<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  relations: Ref<'a, Vec<Relation<Tup, Tag>>>,
  rela_id: usize,
  elem_id: usize,
}

impl<'a, Tup, Tag> Clone for StableVariableBatch<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn clone(&self) -> Self {
    Self {
      relations: Ref::clone(&self.relations),
      rela_id: self.rela_id.clone(),
      elem_id: self.elem_id.clone(),
    }
  }
}

impl<'a, Tup, Tag> Iterator for StableVariableBatch<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Item = Element<Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    let relation = &self.relations[self.rela_id];
    if self.elem_id < relation.len() {
      let elem = &relation[self.elem_id];
      self.elem_id += 1;
      return Some(elem.clone());
    } else {
      return None;
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let size = self.relations[self.rela_id].len();
    (size, Some(size))
  }
}

impl<'a, Tup, Tag> Batch<Tup, Tag> for StableVariableBatch<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
}

pub struct RelationIterator<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  relation: Ref<'a, Relation<Tup, Tag>>,
  elem_id: usize,
}

impl<'a, Tup, Tag> Clone for RelationIterator<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn clone(&self) -> Self {
    Self {
      relation: Ref::clone(&self.relation),
      elem_id: self.elem_id,
    }
  }
}

impl<'a, Tup, Tag> Iterator for RelationIterator<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Item = Element<Tup, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.elem_id < self.relation.len() {
      let elem = &self.relation[self.elem_id];
      self.elem_id += 1;
      return Some(elem.clone());
    } else {
      return None;
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let size = self.relation.len();
    (size, Some(size))
  }
}

impl<'a, Tup, Tag> Batch<Tup, Tag> for RelationIterator<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn step(&mut self, u: usize) {
    self.elem_id += u;
  }

  fn search_ahead<F>(&mut self, mut cmp: F) -> Option<Element<Tup, Tag>>
  where
    F: FnMut(&Tup) -> bool,
  {
    assert!(self.elem_id > 0);
    let mut curr = self.elem_id - 1;
    if curr < self.relation.len() && cmp(&self.relation[curr].tup) {
      let mut step = 1;
      while curr + step < self.relation.len() && cmp(&self.relation[curr + step].tup) {
        curr += step;
        step <<= 1;
      }

      step >>= 1;
      while step > 0 {
        if curr + step < self.relation.len() && cmp(&self.relation[curr + step].tup) {
          curr += step;
        }
        step >>= 1;
      }
      self.elem_id = curr + 1;
      self.next()
    } else {
      None
    }
  }
}
