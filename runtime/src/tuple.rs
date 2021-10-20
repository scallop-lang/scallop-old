use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

pub trait Tuple: Sized + Ord + Clone + Debug + Send + Sync {}

impl<Tup> Tuple for Tup where Tup: Sized + Ord + Clone + Debug + Send + Sync {}

#[derive(Debug, Clone, Copy)]
pub struct TupleAccessor {
  size: u8,
  indices: [u8; 3],
}

impl TupleAccessor {
  pub fn len(&self) -> usize {
    self.size as usize
  }

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
    let mut new_indices = [0; 3];
    for index in 1..self.size {
      new_indices[index as usize - 1] = self.indices[index as usize];
    }
    Self {
      size: self.size - 1,
      indices: new_indices,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TupleType {
  Integer,
  Boolean,
  String,
  Symbol,
  Tuple(Vec<TupleType>),
}

impl TupleType {
  pub fn type_of(&self, acc: &TupleAccessor) -> &TupleType {
    if acc.is_root() {
      self
    } else {
      match self {
        Self::Tuple(types) => types[acc.first_level()].type_of(&acc.indent()),
        _ => panic!("Not possible"),
      }
    }
  }
}

pub trait FromType<T> {
  fn from_type() -> Self;
}

impl FromType<usize> for TupleType {
  fn from_type() -> Self {
    Self::Symbol
  }
}

impl FromType<bool> for TupleType {
  fn from_type() -> Self {
    Self::Boolean
  }
}

impl FromType<i64> for TupleType {
  fn from_type() -> Self {
    Self::Integer
  }
}

impl FromType<Rc<String>> for TupleType {
  fn from_type() -> Self {
    Self::String
  }
}

impl FromType<Arc<String>> for TupleType {
  fn from_type() -> Self {
    Self::String
  }
}

impl FromType<String> for TupleType {
  fn from_type() -> Self {
    Self::String
  }
}

impl FromType<&'static str> for TupleType {
  fn from_type() -> Self {
    Self::String
  }
}

impl FromType<()> for TupleType {
  fn from_type() -> Self {
    Self::Tuple(vec![])
  }
}

impl<A> FromType<(A,)> for TupleType
where
  Self: FromType<A>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type()
    ])
  }
}

impl<A, B> FromType<(A, B)> for TupleType
where
  Self: FromType<A>,
  Self: FromType<B>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type(),
      <Self as FromType<B>>::from_type(),
    ])
  }
}

impl<A, B, C> FromType<(A, B, C)> for TupleType
where
  Self: FromType<A>,
  Self: FromType<B>,
  Self: FromType<C>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type(),
      <Self as FromType<B>>::from_type(),
      <Self as FromType<C>>::from_type(),
    ])
  }
}

impl<A, B, C, D> FromType<(A, B, C, D)> for TupleType
where
  Self: FromType<A>,
  Self: FromType<B>,
  Self: FromType<C>,
  Self: FromType<D>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type(),
      <Self as FromType<B>>::from_type(),
      <Self as FromType<C>>::from_type(),
      <Self as FromType<D>>::from_type(),
    ])
  }
}

impl<A, B, C, D, E> FromType<(A, B, C, D, E)> for TupleType
where
  Self: FromType<A>,
  Self: FromType<B>,
  Self: FromType<C>,
  Self: FromType<D>,
  Self: FromType<E>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type(),
      <Self as FromType<B>>::from_type(),
      <Self as FromType<C>>::from_type(),
      <Self as FromType<D>>::from_type(),
      <Self as FromType<E>>::from_type(),
    ])
  }
}

impl<A, B, C, D, E, F> FromType<(A, B, C, D, E, F)> for TupleType
where
  Self: FromType<A>,
  Self: FromType<B>,
  Self: FromType<C>,
  Self: FromType<D>,
  Self: FromType<E>,
  Self: FromType<F>,
{
  fn from_type() -> Self {
    Self::Tuple(vec![
      <Self as FromType<A>>::from_type(),
      <Self as FromType<B>>::from_type(),
      <Self as FromType<C>>::from_type(),
      <Self as FromType<D>>::from_type(),
      <Self as FromType<E>>::from_type(),
      <Self as FromType<F>>::from_type(),
    ])
  }
}
