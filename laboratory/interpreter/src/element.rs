use super::utils::*;

use scallop_runtime::Semiring;

use super::component::*;

#[derive(Debug, Clone, Eq)]
pub struct ElementHandle<'a, Tag: Semiring> {
  comp_type: &'a ComponentType,
  comp_pointer: *const u8,
  tag: &'a Tag,
}

impl<'a, Tag: Semiring> ElementHandle<'a, Tag> {
  pub fn new(comp_type: &'a ComponentType, ptr: *const u8, tag: &'a Tag) -> Self {
    Self {
      comp_type,
      comp_pointer: ptr,
      tag,
    }
  }

  pub fn component(&self) -> Component {
    fn get_component(comp_type: &ComponentType, ptr: *const u8) -> Component {
      unsafe {
        match comp_type {
          ComponentType::Integer => {
            let i: i64 = *(ptr as *const i64);
            Component::Integer(i)
          }
          ComponentType::Boolean => {
            let b: bool = *(ptr as *const bool);
            Component::Boolean(b)
          }
          ComponentType::String => {
            let s: CompString = (*(ptr as *const CompString)).clone();
            Component::String(s)
          }
          ComponentType::Symbol => {
            let s: usize = *(ptr as *const usize);
            Component::Symbol(s)
          }
          ComponentType::Tuple(fs) => {
            let mut cs = Vec::with_capacity(fs.len());
            let mut acc = 0;
            for f in fs {
              cs.push(get_component(f, ptr.add(acc)));
              acc += f.size();
            }
            Component::Tuple(cs)
          }
        }
      }
    }
    get_component(self.comp_type, self.comp_pointer)
  }

  pub fn tag(&self) -> &Tag {
    self.tag
  }
}

impl<'a, Tag: Semiring> std::cmp::PartialEq for ElementHandle<'a, Tag> {
  fn eq(&self, other: &Self) -> bool {
    fn eq_helper(comp_type: &ComponentType, p1: *const u8, p2: *const u8) -> bool {
      unsafe {
        match comp_type {
          ComponentType::Integer => {
            let i1 = *(p1 as *const i64);
            let i2 = *(p2 as *const i64);
            i1 == i2
          }
          ComponentType::Boolean => {
            let b1 = *(p1 as *const bool);
            let b2 = *(p2 as *const bool);
            b1 == b2
          }
          ComponentType::String => {
            let s1 = &*(p1 as *const CompString);
            let s2 = &*(p2 as *const CompString);
            s1 == s2
          }
          ComponentType::Symbol => {
            let s1 = *(p1 as *const usize);
            let s2 = *(p2 as *const usize);
            s1 == s2
          }
          ComponentType::Tuple(field_types) => {
            let mut acc = 0;
            for field_type in field_types {
              if !eq_helper(field_type, p1.add(acc), p2.add(acc)) {
                return false;
              }
              acc += field_type.size()
            }
            true
          }
        }
      }
    }
    eq_helper(self.comp_type, self.comp_pointer, other.comp_pointer)
  }
}

impl<'a, Tag: Semiring> std::cmp::PartialOrd for ElementHandle<'a, Tag> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    fn cmp_helper(
      comp_type: &ComponentType,
      p1: *const u8,
      p2: *const u8,
    ) -> Option<std::cmp::Ordering> {
      unsafe {
        match comp_type {
          ComponentType::Integer => {
            let i1 = *(p1 as *const i64);
            let i2 = *(p2 as *const i64);
            i1.partial_cmp(&i2)
          }
          ComponentType::Boolean => {
            let b1 = *(p1 as *const bool);
            let b2 = *(p2 as *const bool);
            b1.partial_cmp(&b2)
          }
          ComponentType::String => {
            let s1 = &*(p1 as *const CompString);
            let s2 = &*(p2 as *const CompString);
            s1.partial_cmp(&s2)
          }
          ComponentType::Symbol => {
            let s1 = *(p1 as *const usize);
            let s2 = *(p2 as *const usize);
            s1.partial_cmp(&s2)
          }
          ComponentType::Tuple(field_types) => {
            let mut acc = 0;
            for field_type in field_types {
              match cmp_helper(field_type, p1.add(acc), p2.add(acc)) {
                Some(std::cmp::Ordering::Equal) => {}
                other => return other,
              }
              acc += field_type.size()
            }
            Some(std::cmp::Ordering::Equal)
          }
        }
      }
    }
    cmp_helper(self.comp_type, self.comp_pointer, other.comp_pointer)
  }
}

#[derive(Debug, Clone)]
pub struct ElementHandleMut<'a, Tag: Semiring> {
  pub comp_type: &'a ComponentType,
  pub comp_pointer: *mut u8,
  pub tag: &'a Tag,
}

impl<'a, Tag: Semiring> ElementHandleMut<'a, Tag> {
  pub fn swap(&mut self, other: &mut Self) {
    let size = self.comp_type.size();
    let mut buffer = Vec::<u8>::with_capacity(size);
    unsafe {
      std::ptr::copy(self.comp_pointer, buffer.as_mut_ptr(), size);
      std::ptr::copy(other.comp_pointer, self.comp_pointer, size);
      std::ptr::copy(buffer.as_ptr(), other.comp_pointer, size);
    }
  }

  pub fn copy_from_component(&mut self, comp: &Component) {
    fn copy_helper(pointer: *mut u8, comp: &Component) -> usize {
      unsafe {
        match comp {
          Component::Integer(i) => {
            *(pointer as *mut i64) = *i;
            std::mem::size_of::<i64>()
          }
          Component::Boolean(b) => {
            *(pointer as *mut bool) = *b;
            std::mem::size_of::<bool>()
          }
          Component::Symbol(s) => {
            *(pointer as *mut usize) = *s;
            std::mem::size_of::<usize>()
          }
          Component::String(s) => {
            *(pointer as *mut CompString) = s.clone();
            std::mem::size_of::<CompString>()
          }
          Component::Tuple(fs) => {
            let mut acc = 0;
            for f in fs {
              acc += copy_helper(pointer.add(acc), f);
            }
            acc
          }
        }
      }
    }
    copy_helper(self.comp_pointer, comp);
  }
}

impl<'a, Tag: Semiring> std::cmp::PartialEq for ElementHandleMut<'a, Tag> {
  fn eq(&self, other: &Self) -> bool {
    fn eq_helper(comp_type: &ComponentType, p1: *const u8, p2: *const u8) -> bool {
      unsafe {
        match comp_type {
          ComponentType::Integer => {
            let i1 = *(p1 as *const i64);
            let i2 = *(p2 as *const i64);
            i1 == i2
          }
          ComponentType::Boolean => {
            let b1 = *(p1 as *const bool);
            let b2 = *(p2 as *const bool);
            b1 == b2
          }
          ComponentType::String => {
            let s1 = &*(p1 as *const CompString);
            let s2 = &*(p2 as *const CompString);
            s1 == s2
          }
          ComponentType::Symbol => {
            let s1 = *(p1 as *const usize);
            let s2 = *(p2 as *const usize);
            s1 == s2
          }
          ComponentType::Tuple(field_types) => {
            let mut acc = 0;
            for field_type in field_types {
              if !eq_helper(field_type, p1.add(acc), p2.add(acc)) {
                return false;
              }
              acc += field_type.size()
            }
            true
          }
        }
      }
    }
    eq_helper(self.comp_type, self.comp_pointer, other.comp_pointer)
  }
}

impl<'a, Tag: Semiring> std::cmp::PartialOrd for ElementHandleMut<'a, Tag> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    fn cmp_helper(
      comp_type: &ComponentType,
      p1: *const u8,
      p2: *const u8,
    ) -> Option<std::cmp::Ordering> {
      unsafe {
        match comp_type {
          ComponentType::Integer => {
            let i1 = *(p1 as *const i64);
            let i2 = *(p2 as *const i64);
            i1.partial_cmp(&i2)
          }
          ComponentType::Boolean => {
            let b1 = *(p1 as *const bool);
            let b2 = *(p2 as *const bool);
            b1.partial_cmp(&b2)
          }
          ComponentType::String => {
            let s1 = &*(p1 as *const CompString);
            let s2 = &*(p2 as *const CompString);
            s1.partial_cmp(&s2)
          }
          ComponentType::Symbol => {
            let s1 = *(p1 as *const usize);
            let s2 = *(p2 as *const usize);
            s1.partial_cmp(&s2)
          }
          ComponentType::Tuple(field_types) => {
            let mut acc = 0;
            for field_type in field_types {
              match cmp_helper(field_type, p1.add(acc), p2.add(acc)) {
                Some(std::cmp::Ordering::Equal) => {}
                other => return other,
              }
              acc += field_type.size()
            }
            Some(std::cmp::Ordering::Equal)
          }
        }
      }
    }
    cmp_helper(self.comp_type, self.comp_pointer, other.comp_pointer)
  }
}
