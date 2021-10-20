use super::utils::*;
use crate::*;

#[derive(Clone, PartialEq, Eq)]
pub enum DynTuple {
  Integer(i64),
  Boolean(bool),
  String(CompString),
  Symbol(usize),
  Tuple(Vec<DynTuple>),
}

impl DynTuple {
  pub fn is_true(&self) -> bool {
    match self {
      Self::Boolean(b) => b.clone(),
      _ => panic!("Not a boolean"),
    }
  }
}

impl std::fmt::Debug for DynTuple {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Integer(i) => write!(f, "{}", i),
      Self::Boolean(b) => write!(f, "{}", b),
      Self::String(s) => write!(f, "\"{}\"", s),
      Self::Symbol(s) => write!(f, "{}", s),
      Self::Tuple(cs) => {
        write!(f, "(")?;
        if cs.len() == 1 {
          write!(f, "{:?},", cs[0])?;
        } else {
          for (i, c) in cs.iter().enumerate() {
            write!(f, "{:?}", c)?;
            if i < cs.len() - 1 {
              write!(f, ", ")?;
            }
          }
        }
        write!(f, ")")
      }
    }
  }
}

impl std::ops::BitAnd for DynTuple {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Boolean(b1), Self::Boolean(b2)) => Self::Boolean(b1 & b2),
      _ => panic!("Invalid and operation"),
    }
  }
}

impl std::ops::BitOr for DynTuple {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Boolean(b1), Self::Boolean(b2)) => Self::Boolean(b1 | b2),
      _ => panic!("Invalid or operation"),
    }
  }
}

impl std::ops::Not for DynTuple {
  type Output = Self;

  fn not(self) -> Self {
    match self {
      Self::Boolean(b) => Self::Boolean(!b),
      _ => panic!("Invalid not operation"),
    }
  }
}

impl std::ops::Neg for DynTuple {
  type Output = Self;

  fn neg(self) -> Self {
    match self {
      Self::Integer(i) => Self::Integer(-i),
      _ => panic!("Invalid not operation"),
    }
  }
}

impl std::ops::Add for DynTuple {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Integer(i1), Self::Integer(i2)) => Self::Integer(i1 + i2),
      _ => panic!("Invalid add operation"),
    }
  }
}

impl std::ops::Sub for DynTuple {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Integer(i1), Self::Integer(i2)) => Self::Integer(i1 - i2),
      _ => panic!("Invalid sub operation"),
    }
  }
}

impl std::ops::Mul for DynTuple {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Integer(i1), Self::Integer(i2)) => Self::Integer(i1 * i2),
      _ => panic!("Invalid mul operation"),
    }
  }
}

impl std::ops::Div for DynTuple {
  type Output = Self;

  fn div(self, rhs: Self) -> Self {
    match (self, rhs) {
      (Self::Integer(i1), Self::Integer(i2)) => Self::Integer(i1 / i2),
      _ => panic!("Invalid div operation"),
    }
  }
}

impl From<i64> for DynTuple {
  fn from(i: i64) -> Self {
    Self::Integer(i)
  }
}

impl From<bool> for DynTuple {
  fn from(b: bool) -> Self {
    Self::Boolean(b)
  }
}

impl From<&str> for DynTuple {
  fn from(s: &str) -> Self {
    Self::String(CompString::new(s.to_string()))
  }
}

impl From<String> for DynTuple {
  fn from(s: String) -> Self {
    Self::String(CompString::new(s))
  }
}

impl From<std::rc::Rc<String>> for DynTuple {
  fn from(s: std::rc::Rc<String>) -> Self {
    Self::String(std::sync::Arc::new((*s).clone()))
  }
}

impl From<std::sync::Arc<String>> for DynTuple {
  fn from(s: std::sync::Arc<String>) -> Self {
    Self::String(s.clone())
  }
}

impl From<usize> for DynTuple {
  fn from(u: usize) -> Self {
    Self::Symbol(u)
  }
}

impl From<()> for DynTuple {
  fn from((): ()) -> Self {
    Self::Tuple(vec![])
  }
}

impl<A: Into<DynTuple>> From<(A,)> for DynTuple {
  fn from((a,): (A,)) -> Self {
    Self::Tuple(vec![a.into()])
  }
}

impl<A, B> From<(A, B)> for DynTuple
where
  A: Into<DynTuple>,
  B: Into<DynTuple>
{
  fn from((a, b): (A, B)) -> Self {
    Self::Tuple(vec![a.into(), b.into()])
  }
}

impl<A, B, C> From<(A, B, C)> for DynTuple
where
  A: Into<DynTuple>,
  B: Into<DynTuple>,
  C: Into<DynTuple>,
{
  fn from((a, b, c): (A, B, C)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into()])
  }
}

impl<A, B, C, D> From<(A, B, C, D)> for DynTuple
where
  A: Into<DynTuple>,
  B: Into<DynTuple>,
  C: Into<DynTuple>,
  D: Into<DynTuple>,
{
  fn from((a, b, c, d): (A, B, C, D)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into(), d.into()])
  }
}

impl<A, B, C, D, E> From<(A, B, C, D, E)> for DynTuple
where
  A: Into<DynTuple>,
  B: Into<DynTuple>,
  C: Into<DynTuple>,
  D: Into<DynTuple>,
  E: Into<DynTuple>,
{
  fn from((a, b, c, d, e): (A, B, C, D, E)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into(), d.into(), e.into()])
  }
}

impl<A, B, C, D, E, F> From<(A, B, C, D, E, F)> for DynTuple
where
  A: Into<DynTuple>,
  B: Into<DynTuple>,
  C: Into<DynTuple>,
  D: Into<DynTuple>,
  E: Into<DynTuple>,
  F: Into<DynTuple>,
{
  fn from((a, b, c, d, e, f): (A, B, C, D, E, F)) -> Self {
    Self::Tuple(vec![a.into(), b.into(), c.into(), d.into(), e.into(), f.into()])
  }
}

impl std::ops::Index<TupleAccessor> for DynTuple {
  type Output = DynTuple;

  fn index(&self, acc: TupleAccessor) -> &Self::Output {
    fn index_helper(source: &DynTuple, acc: TupleAccessor) -> &DynTuple {
      if acc.len() == 0 {
        source
      } else {
        match source {
          DynTuple::Tuple(comps) => index_helper(&comps[acc.first_level()], acc.indent()),
          _ => panic!("Should not happen"),
        }
      }
    }
    index_helper(self, acc)
  }
}

impl std::ops::Index<usize> for DynTuple {
  type Output = DynTuple;

  fn index(&self, acc: usize) -> &Self::Output {
    match self {
      Self::Tuple(cs) => &cs[acc],
      _ => panic!("Not a tuple"),
    }
  }
}

impl std::cmp::PartialOrd for DynTuple {
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

impl std::cmp::Ord for DynTuple {
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

impl DynTuple {
  pub fn component_type(&self) -> TupleType {
    match self {
      Self::Integer(_) => TupleType::Integer,
      Self::Boolean(_) => TupleType::Boolean,
      Self::String(_) => TupleType::String,
      Self::Symbol(_) => TupleType::Symbol,
      Self::Tuple(comps) => TupleType::Tuple(comps.iter().map(Self::component_type).collect()),
    }
  }

  pub fn type_check(&self, comp_type: &TupleType) -> bool {
    match (self, comp_type) {
      (Self::Integer(_), TupleType::Integer) => true,
      (Self::Boolean(_), TupleType::Boolean) => true,
      (Self::String(_), TupleType::String) => true,
      (Self::Symbol(_), TupleType::Symbol) => true,
      (Self::Tuple(cs), TupleType::Tuple(fs)) => {
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
