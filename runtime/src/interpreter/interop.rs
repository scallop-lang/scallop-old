use super::*;
use crate::*;

pub trait StaticVariableTrait<Tag: Semiring>: VariableTrait<Tag> {
  fn num_stable_relations(&self) -> usize;

  fn num_stable_elements(&self, rela_id: usize) -> usize;

  fn stable_element(&self, rela_id: usize, elem_id: usize) -> DynElement<Tag>;

  fn num_recent_elements(&self) -> usize;

  fn recent_element(&self, elem_id: usize) -> DynElement<Tag>;
}

impl<Tup: Tuple + Into<DynTuple>, Tag: Semiring> StaticVariableTrait<Tag> for Variable<Tup, Tag> {
  fn num_stable_relations(&self) -> usize {
    self.stable.borrow().len()
  }

  fn num_stable_elements(&self, rela_id: usize) -> usize {
    self.stable.borrow()[rela_id].len()
  }

  fn stable_element(&self, rela_id: usize, elem_id: usize) -> DynElement<Tag> {
    self.stable.borrow()[rela_id][elem_id].clone().into()
  }

  fn num_recent_elements(&self) -> usize {
    self.recent.borrow().len()
  }

  fn recent_element(&self, elem_id: usize) -> DynElement<Tag> {
    self.recent.borrow()[elem_id].clone().into()
  }
}

#[derive(Clone)]
pub struct StaticVariable<'a, Tag: Semiring>(pub &'a dyn StaticVariableTrait<Tag>);

impl<'a, Tag: Semiring> StaticVariable<'a, Tag> {
  pub fn num_stable_relations(&self) -> usize {
    self.0.num_stable_relations()
  }

  pub fn num_stable_elements(&self, rela_id: usize) -> usize {
    self.0.num_stable_elements(rela_id)
  }

  pub fn stable_element(&self, rela_id: usize, elem_id: usize) -> DynElement<Tag> {
    self.0.stable_element(rela_id, elem_id)
  }

  pub fn num_recent_elements(&self) -> usize {
    self.0.num_recent_elements()
  }

  pub fn recent_element(&self, elem_id: usize) -> DynElement<Tag> {
    self.0.recent_element(elem_id)
  }
}
