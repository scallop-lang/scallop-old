use scallop_runtime::Semiring;

use super::component::*;
use super::element::*;

#[derive(Debug, Clone)]
pub struct Relation<Tag: Semiring> {
  comp_type: ComponentType,
  comp_size: usize,
  data: Vec<u8>,
  tags: Vec<Tag>,
}

impl<Tag: Semiring> Relation<Tag> {
  pub fn new(comp_type: ComponentType) -> Self {
    let comp_size = comp_type.size();
    Self {
      comp_type,
      comp_size,
      data: vec![],
      tags: vec![],
    }
  }

  pub fn from_vec_unchecked(elements: Vec<(Component, Tag)>) -> Self {
    // Get the type from the first element
    let comp_type = if let Some((first_elem, _)) = elements.get(0) {
      first_elem.component_type()
    } else {
      panic!("Has to have at least one element");
    };

    // Initialization
    let comp_size = comp_type.size();
    let (components, tags): (Vec<Component>, Vec<Tag>) = elements.into_iter().unzip();
    let mut data = vec![0; components.len() * comp_size];

    // Fill in the data by iterating through the components
    for (i, component) in components.into_iter().enumerate() {
      // Type check; panic if failed
      if !component.type_check(&comp_type) {
        panic!(
          "Mismatch type of component {:?}, expected {:?}",
          component, comp_type
        );
      }

      // Compute offset and create handle
      let offset = i * comp_size;
      let pointer = &mut data[offset] as *mut u8;
      let mut handle = ElementHandleMut {
        comp_type: &comp_type,
        comp_pointer: pointer,
        tag: &tags[i],
      };

      // Copy data from the component to the data
      handle.copy_from_component(&component);
    }

    // Create the relation
    Self {
      comp_type,
      comp_size,
      data,
      tags,
    }
  }

  pub fn from_vec(elements: Vec<(Component, Tag)>, _: &Tag::Context) -> Self {
    let relation = Self::from_vec_unchecked(elements);
    relation
  }

  pub fn sort(&mut self) {
    unsafe {
      merge_sort(
        &mut self.data,
        &mut self.tags,
        &self.comp_type,
        self.comp_size,
      )
    }
  }

  pub fn len(&self) -> usize {
    self.tags.len()
  }

  pub fn is_empty(&self) -> bool {
    self.tags.is_empty()
  }

  pub fn merge(self, _: Self, _: &Tag::Context) -> Self {
    panic!("Not implemented")
  }

  pub fn iter<'a>(&'a self) -> RelationIter<'a, Tag> {
    RelationIter {
      relation: self,
      index: 0,
    }
  }
}

pub struct RelationIter<'a, Tag: Semiring> {
  relation: &'a Relation<Tag>,
  index: usize,
}

impl<'a, Tag: Semiring> Iterator for RelationIter<'a, Tag> {
  type Item = ElementHandle<'a, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.relation.tags.len() {
      None
    } else {
      let ptr = unsafe {
        self
          .relation
          .data
          .as_ptr()
          .add(self.index * self.relation.comp_size)
      };
      let tag = &self.relation.tags[self.index];
      let handle = ElementHandle::new(&self.relation.comp_type, ptr, tag);
      self.index += 1;
      Some(handle)
    }
  }
}

unsafe fn insert_head<Tag: Semiring>(
  data: &mut [u8],
  tags: &mut [Tag],
  comp_type: &ComponentType,
  comp_size: usize,
) {
  let ith_handle =
    |i: usize| ElementHandle::new(comp_type, data.as_ptr().add(i * comp_size), &tags[0]);
  if tags.len() >= 2 && ith_handle(0) < ith_handle(1) {
    // let buf =
  }
}

unsafe fn merge_sort<Tag: Semiring>(
  data: &mut [u8],
  tags: &mut [Tag],
  comp_type: &ComponentType,
  comp_size: usize,
) {
  const MAX_INSERTION: usize = 20;
  // const MIN_RUN: usize = 10;
  let len = tags.len();
  if len <= MAX_INSERTION {
    if len >= 2 {
      for i in (0..len - 1).rev() {
        insert_head(&mut data[i * comp_size..], tags, comp_type, comp_size);
      }
    }
    return;
  }
}
