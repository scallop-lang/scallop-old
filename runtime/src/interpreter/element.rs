use super::DynTuple;
use crate::{Element, Semiring, Tuple};

#[derive(Debug, Clone)]
pub struct DynElement<Tag: Semiring> {
  pub tup: DynTuple,
  pub tag: Tag,
}

impl<Tag: Semiring> PartialEq for DynElement<Tag> {
  fn eq(&self, other: &Self) -> bool {
    self.tup == other.tup
  }
}

impl<Tag: Semiring> Eq for DynElement<Tag> {}

impl<Tag: Semiring> PartialOrd for DynElement<Tag> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.tup.partial_cmp(&other.tup)
  }
}

impl<Tag: Semiring> Ord for DynElement<Tag> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.tup.cmp(&other.tup)
  }
}

impl<Tup: Tuple + Into<DynTuple>, Tag: Semiring> From<Element<Tup, Tag>> for DynElement<Tag> {
  fn from(elem: Element<Tup, Tag>) -> DynElement<Tag> {
    DynElement {
      tup: elem.tup.into(),
      tag: elem.tag,
    }
  }
}
