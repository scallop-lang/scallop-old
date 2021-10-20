pub trait Node: Clone {
  type T;

  fn new(t: Self::T) -> Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Location {
  pub id: usize,
  pub byte_offset: usize,
  pub length: usize,
  pub row: usize,
  pub col: usize,
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.row, self.col)
  }
}

impl Location {
  pub fn new() -> Self {
    Self {
      id: 0,
      byte_offset: 0,
      length: 0,
      row: 0,
      col: 0,
    }
  }

  pub fn with_id(id: usize) -> Self {
    Self {
      id,
      byte_offset: 0,
      length: 0,
      row: 0,
      col: 0,
    }
  }

  pub fn offset(byte_offset: usize) -> Self {
    Self {
      id: 0,
      byte_offset,
      length: 0,
      row: 0,
      col: 0,
    }
  }

  pub fn span(begin: usize, end: usize) -> Self {
    Self {
      id: 0,
      byte_offset: begin,
      length: end - begin,
      row: 0,
      col: 0,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Located<T: Node> {
  pub location: Location,
  pub node: T,
}

impl<T: Node> Located<T> {
  pub fn new(t: T::T) -> Self {
    Self {
      location: Location::new(),
      node: T::new(t),
    }
  }

  pub fn offset(byte_offset: usize, t: T::T) -> Self {
    Self {
      location: Location::offset(byte_offset),
      node: T::new(t),
    }
  }

  pub fn span(begin: usize, end: usize, t: T::T) -> Self {
    Self {
      location: Location::span(begin, end),
      node: T::new(t),
    }
  }

  pub fn clone_with_id(&self, id: usize) -> Self {
    let new_location = Location::with_id(id);
    Self {
      location: new_location,
      node: self.node.clone(),
    }
  }
}
