use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::{Semiring, SemiringContext};

#[derive(Clone)]
pub struct DynVariable<Tag: Semiring> {
  pub stable: Rc<RefCell<Vec<DynRelation<Tag>>>>,
  pub recent: Rc<RefCell<DynRelation<Tag>>>,
  to_add: Rc<RefCell<Vec<DynRelation<Tag>>>>,
}

impl<Tag: Semiring> DynVariable<Tag> {
  pub fn new() -> Self {
    Self {
      stable: Rc::new(RefCell::new(Vec::new())),
      recent: Rc::new(RefCell::new(DynRelation::empty())),
      to_add: Rc::new(RefCell::new(Vec::new())),
    }
  }

  pub fn insert_with_context(
    &self,
    ctx: &mut Tag::Context,
    data: Vec<(
      <<Tag as Semiring>::Context as SemiringContext<Tag>>::Info,
      DynTuple,
    )>,
  ) {
    let elements = data
      .into_iter()
      .map(|(info, tup)| DynElement {
        tup,
        tag: ctx.base_tag(info),
      })
      .collect::<Vec<_>>();
    let dataflow = DynDataflow::Vec(&elements);
    self.insert(ctx, &dataflow);
  }

  pub fn num_stable(&self) -> usize {
    self
      .stable
      .borrow()
      .iter()
      .fold(0, |a, rela| a + rela.len())
  }

  pub fn num_recent(&self) -> usize {
    self.recent.borrow().len()
  }

  pub fn changed(&mut self, ctx: &Tag::Context) -> bool {
    // 1. Merge self.recent into self.stable.
    if !self.recent.borrow().is_empty() {
      let mut recent = ::std::mem::replace(&mut (*self.recent.borrow_mut()), DynRelation::empty());
      while self
        .stable
        .borrow()
        .last()
        .map(|x| x.len() <= 2 * recent.len())
        == Some(true)
      {
        let last = self.stable.borrow_mut().pop().unwrap();
        recent = recent.merge(last, ctx);
      }
      self.stable.borrow_mut().push(recent);
    }

    // 2. Move self.to_add into self.recent.
    let to_add = self.to_add.borrow_mut().pop();
    if let Some(mut to_add) = to_add {
      while let Some(to_add_more) = self.to_add.borrow_mut().pop() {
        to_add = to_add.merge(to_add_more, ctx);
      }

      // Make sure that there is no duplicates
      for batch in self.stable.borrow().iter() {
        let mut slice = &batch[..];
        // Only gallop if the slice is relatively large.
        if slice.len() > 4 * to_add.elements.len() {
          to_add.elements.retain(|x| {
            slice = super::utils::gallop(slice, |y| y < x);
            slice.is_empty() || &slice[0] != x
          });
        } else {
          to_add.elements.retain(|x| {
            while !slice.is_empty() && &slice[0] < x {
              slice = &slice[1..];
            }
            slice.is_empty() || &slice[0] != x
          });
        }
      }

      *self.recent.borrow_mut() = to_add;
    }

    !self.recent.borrow().is_empty()
  }

  pub fn insert<'a>(&self, ctx: &Tag::Context, d: &DynDataflow<'a, Tag>) {
    for batch in d.iter_recent() {
      let data = batch.filter(|e| e.tag.is_valid(ctx)).collect::<Vec<_>>();
      self
        .to_add
        .borrow_mut()
        .push(DynRelation::from_vec(data, ctx));
    }
  }

  pub fn insert_stable<'a>(&self, ctx: &Tag::Context, d: &DynDataflow<'a, Tag>) {
    for batch in d.iter_stable() {
      let data = batch.filter(|e| e.tag.is_valid(ctx)).collect::<Vec<_>>();
      self.to_add.borrow_mut().push(DynRelation::from_vec(data, ctx));
    }
  }

  pub fn complete(&self, ctx: &Tag::Context) -> DynRelation<Tag> {
    assert!(self.recent.borrow().is_empty());
    assert!(self.to_add.borrow().is_empty());
    let mut result = DynRelation::empty();
    while let Some(batch) = self.stable.borrow_mut().pop() {
      result = result.merge(batch, ctx);
    }
    result
  }
}
