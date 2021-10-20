use std::cell::RefCell;
use std::rc::Rc;

use super::*;

pub trait VariableTrait<Tag>
where
  Tag: Semiring,
{
  fn changed(&mut self, semiring_ctx: &Tag::Context) -> bool;
}

#[derive(Clone)]
pub struct Variable<Tup: Tuple, Tag: Semiring = ()> {
  pub stable: Rc<RefCell<Vec<Relation<Tup, Tag>>>>,
  pub recent: Rc<RefCell<Relation<Tup, Tag>>>,
  to_add: Rc<RefCell<Vec<Relation<Tup, Tag>>>>,
}

impl<Tup, Tag> Variable<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  pub fn new() -> Self {
    Variable {
      stable: Rc::new(RefCell::new(Vec::new())),
      recent: Rc::new(RefCell::new(Relation::empty())),
      to_add: Rc::new(RefCell::new(Vec::new())),
    }
  }

  pub fn insert_with_context(
    &self,
    semiring_ctx: &mut Tag::Context,
    data: Vec<(
      <<Tag as Semiring>::Context as SemiringContext<Tag>>::Info,
      Tup,
    )>,
  ) {
    let elements = data
      .into_iter()
      .map(|(info, tup)| Element {
        tup,
        tag: semiring_ctx.base_tag(info),
      })
      .collect::<Vec<_>>();
    self.insert(semiring_ctx, elements);
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

  pub fn complete(&self, semiring_ctx: &Tag::Context) -> Relation<Tup, Tag> {
    assert!(self.recent.borrow().is_empty());
    assert!(self.to_add.borrow().is_empty());
    let mut result: Relation<Tup, Tag> = Relation::empty();
    while let Some(batch) = self.stable.borrow_mut().pop() {
      result = result.merge(batch, semiring_ctx);
    }
    result
  }
}

pub trait InsertIntoVariable<D, Tag>
where
  Tag: Semiring,
{
  fn insert(&self, ctx: &<Tag as Semiring>::Context, d: D);
}

impl<D, Tup, Tag> InsertIntoVariable<D, Tag> for Variable<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  D: Dataflow<Tup, Tag>,
{
  fn insert(&self, ctx: &Tag::Context, d: D) {
    let batches = d.iter_recent();
    for batch in batches {
      let data = batch.filter(|e| e.tag.is_valid(ctx)).collect::<Vec<_>>();
      self.to_add.borrow_mut().push(Relation::from_vec(data, ctx));
    }
  }
}

impl<Tup, Tag> VariableTrait<Tag> for Variable<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn changed(&mut self, semiring_ctx: &Tag::Context) -> bool {
    // 1. Merge self.recent into self.stable.
    if !self.recent.borrow().is_empty() {
      let mut recent = ::std::mem::replace(&mut (*self.recent.borrow_mut()), Relation::empty());
      while self
        .stable
        .borrow()
        .last()
        .map(|x| x.len() <= 2 * recent.len())
        == Some(true)
      {
        let last = self.stable.borrow_mut().pop().unwrap();
        recent = recent.merge(last, semiring_ctx);
      }
      self.stable.borrow_mut().push(recent);
    }

    // 2. Move self.to_add into self.recent.
    let to_add = self.to_add.borrow_mut().pop();
    if let Some(mut to_add) = to_add {
      while let Some(to_add_more) = self.to_add.borrow_mut().pop() {
        to_add = to_add.merge(to_add_more, semiring_ctx);
      }

      // Make sure that there is no duplicates
      for batch in self.stable.borrow().iter() {
        let mut slice = &batch[..];
        // Only gallop if the slice is relatively large.
        if slice.len() > 4 * to_add.elements.len() {
          to_add.elements.retain(|x| {
            slice = utils::gallop::gallop(slice, |y| y < x);
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
}
