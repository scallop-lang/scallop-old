use super::utils::*;

#[derive(Debug, Clone, Copy)]
pub struct ComponentAccessor {
  size: u8,
  indices: [u8; 3],
}

impl ComponentAccessor {
  pub fn is_root(&self) -> bool {
    self.size == 0
  }

  pub fn first_level(&self) -> usize {
    self.indices[0] as usize
  }

  pub fn root() -> Self {
    Self {
      size: 0,
      indices: [0; 3],
    }
  }

  pub fn top(index: usize) -> Self {
    Self {
      size: 1,
      indices: [index as u8, 0, 0],
    }
  }

  pub fn from_indices(indices: &[u8]) -> Self {
    let mut new_indices = [0; 3];
    for index in 0..indices.len().min(3) {
      new_indices[index] = indices[index];
    }
    Self {
      size: indices.len() as u8,
      indices: new_indices,
    }
  }

  pub fn indent(&self) -> Self {
    Self::from_indices(&self.indices[1..])
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
  Integer(i64),
  Boolean(bool),
  String(CompString),
  Symbol(usize),
  Tuple(Vec<Component>),
}

impl From<i64> for Component {
  fn from(i: i64) -> Self {
    Self::Integer(i)
  }
}

impl From<bool> for Component {
  fn from(b: bool) -> Self {
    Self::Boolean(b)
  }
}

impl From<&str> for Component {
  fn from(s: &str) -> Self {
    Self::String(CompString::new(s.to_string()))
  }
}

impl From<usize> for Component {
  fn from(u: usize) -> Self {
    Self::Symbol(u)
  }
}

impl<A: Into<Component>> From<(A,)> for Component {
  fn from((a,): (A,)) -> Self {
    Self::Tuple(vec![a.into()])
  }
}

impl<A: Into<Component>, B: Into<Component>> From<(A, B)> for Component {
  fn from((a, b): (A, B)) -> Self {
    Self::Tuple(vec![a.into(), b.into()])
  }
}

impl<A: Into<Component>, B: Into<Component>, C: Into<Component>> From<(A, B, C)> for Component {
  fn from((a, b, c): (A, B, C)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into()])
  }
}

impl<A: Into<Component>, B: Into<Component>, C: Into<Component>, D: Into<Component>>
  From<(A, B, C, D)> for Component
{
  fn from((a, b, c, d): (A, B, C, D)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into(), d.into()])
  }
}

impl std::ops::Index<ComponentAccessor> for Component {
  type Output = Component;

  fn index(&self, acc: ComponentAccessor) -> &Self::Output {
    fn index_helper(source: &Component, acc: ComponentAccessor) -> &Component {
      if acc.size == 0 {
        source
      } else {
        match source {
          Component::Tuple(comps) => index_helper(&comps[acc.first_level()], acc.indent()),
          _ => panic!("Should not happen"),
        }
      }
    }
    index_helper(self, acc)
  }
}

impl std::cmp::PartialOrd for Component {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self, other) {
      (Self::Integer(i1), Self::Integer(i2)) => i1.partial_cmp(i2),
      (Self::Boolean(b1), Self::Boolean(b2)) => b1.partial_cmp(b2),
      (Self::String(s1), Self::String(s2)) => s1.partial_cmp(s2),
      (Self::Symbol(s1), Self::Symbol(s2)) => s1.partial_cmp(s2),
      (Self::Tuple(t1), Self::Tuple(t2)) => t1.partial_cmp(t2),
      _ => None,
    }
  }
}

impl std::cmp::Ord for Component {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match (self, other) {
      (Self::Integer(i1), Self::Integer(i2)) => i1.cmp(i2),
      (Self::Boolean(b1), Self::Boolean(b2)) => b1.cmp(b2),
      (Self::String(s1), Self::String(s2)) => s1.cmp(s2),
      (Self::Symbol(s1), Self::Symbol(s2)) => s1.cmp(s2),
      (Self::Tuple(t1), Self::Tuple(t2)) => t1.cmp(t2),
      _ => panic!("Not possible"),
    }
  }
}

impl Component {
  pub fn component_type(&self) -> ComponentType {
    match self {
      Self::Integer(_) => ComponentType::Integer,
      Self::Boolean(_) => ComponentType::Boolean,
      Self::String(_) => ComponentType::String,
      Self::Symbol(_) => ComponentType::Symbol,
      Self::Tuple(comps) => ComponentType::Tuple(comps.iter().map(Self::component_type).collect()),
    }
  }

  pub fn type_check(&self, comp_type: &ComponentType) -> bool {
    match (self, comp_type) {
      (Self::Integer(_), ComponentType::Integer) => true,
      (Self::Boolean(_), ComponentType::Boolean) => true,
      (Self::String(_), ComponentType::String) => true,
      (Self::Symbol(_), ComponentType::Symbol) => true,
      (Self::Tuple(cs), ComponentType::Tuple(fs)) => {
        if cs.len() == fs.len() {
          cs.iter().zip(fs).all(|(c, f)| c.type_check(f))
        } else {
          false
        }
      }
      _ => false,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentType {
  Integer,
  Boolean,
  String,
  Symbol,
  Tuple(Vec<ComponentType>),
}

impl ComponentType {
  pub fn size(&self) -> usize {
    match self {
      Self::Integer => std::mem::size_of::<i64>(),
      Self::Boolean => std::mem::size_of::<bool>(),
      Self::String => std::mem::size_of::<CompString>(),
      Self::Symbol => std::mem::size_of::<usize>(),
      Self::Tuple(types) => types.iter().map(Self::size).sum(),
    }
  }

  pub fn type_of(&self, acc: &ComponentAccessor) -> &ComponentType {
    if acc.is_root() {
      self
    } else {
      match self {
        Self::Tuple(types) => types[acc.first_level()].type_of(&acc.indent()),
        _ => panic!("Not possible"),
      }
    }
  }

  pub fn offset_of(&self, acc: &ComponentAccessor) -> usize {
    if acc.is_root() {
      0
    } else {
      match self {
        Self::Tuple(types) => {
          let prefix: usize = types[0..acc.first_level()].iter().map(Self::size).sum();
          prefix + types[acc.first_level()].offset_of(&acc.indent())
        }
        _ => panic!("Not possible"),
      }
    }
  }
}
