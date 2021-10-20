use std::cmp::Ordering;

use super::element::*;
use super::semiring::*;
use super::*;

#[derive(Clone)]
pub struct Relation<Tup: Tuple, Tag: Semiring = ()> {
  pub elements: Vec<Element<Tup, Tag>>,
}

impl<Tup, Tag> Relation<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  pub fn empty() -> Self {
    Relation { elements: vec![] }
  }

  pub fn from_vec_unchecked(elements: Vec<Element<Tup, Tag>>) -> Self {
    Relation { elements }
  }

  pub fn from_vec(mut elements: Vec<Element<Tup, Tag>>, ctx: &Tag::Context) -> Self {
    // First sort the array
    elements.sort();

    // Deduplicate
    let mut index = 0;
    let mut to_keep = 0;
    if elements.len() > 1 {
      let mut last_index = index;
      to_keep += 1;
      index += 1;
      while index < elements.len() {
        if &elements[index].tup == &elements[last_index].tup {
          let new_tag = Tag::add(&ctx, &elements[last_index].tag, &elements[index].tag);
          elements[last_index].tag = new_tag;
        } else {
          if to_keep < index {
            elements.swap(to_keep, index);
          }
          last_index = to_keep;
          to_keep += 1;
        }
        index += 1;
      }
      elements.truncate(to_keep);
    }

    Relation { elements }
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }

  pub fn is_empty(&self) -> bool {
    self.elements.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &Element<Tup, Tag>> {
    self.elements.iter()
  }

  pub fn into_iter(self) -> impl IntoIterator<Item = Element<Tup, Tag>> {
    self.elements.into_iter()
  }

  pub fn merge(self, other: Self, semiring_ctx: &Tag::Context) -> Self {
    let Relation {
      elements: mut elements1,
    } = self;
    let Relation {
      elements: mut elements2,
    } = other;

    // If one of the element lists is zero-length, we don't need to do any work
    if elements1.is_empty() {
      return Relation {
        elements: elements2,
      };
    }

    if elements2.is_empty() {
      return Relation {
        elements: elements1,
      };
    }

    // Make sure that elements1 starts with the lower element
    // Will not panic since both collections must have at least 1 element at this point
    if elements1[0] > elements2[0] {
      std::mem::swap(&mut elements1, &mut elements2);
    }

    // Fast path for when all the new elements are after the exiting ones
    if elements1[elements1.len() - 1] < elements2[0] {
      elements1.extend(elements2.into_iter());
      // println!("fast path");
      return Relation {
        elements: elements1,
      };
    }

    let mut elements = Vec::with_capacity(elements1.len() + elements2.len());
    let mut elements1 = elements1.drain(..);
    let mut elements2 = elements2.drain(..).peekable();

    elements.push(elements1.next().unwrap());
    if elements.first() == elements2.peek() {
      elements2.next();
    }

    for mut elem in elements1 {
      while elements2.peek().map(|x| x.cmp(&elem)) == Some(Ordering::Less) {
        elements.push(elements2.next().unwrap());
      }
      if elements2.peek().map(|x| x.cmp(&elem)) == Some(Ordering::Equal) {
        // Merge the tags
        let e2 = elements2.peek().unwrap();
        elem.tag = Tag::add(&semiring_ctx, &elem.tag, &e2.tag);

        elements2.next();
      }
      elements.push(elem);
    }

    // Finish draining second list
    elements.extend(elements2);

    Relation { elements }
  }
}

impl<Tup, Tag> std::ops::Deref for Relation<Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  type Target = [Element<Tup, Tag>];

  fn deref(&self) -> &Self::Target {
    &self.elements[..]
  }
}
