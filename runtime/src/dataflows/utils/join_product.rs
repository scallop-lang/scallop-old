use crate::*;

#[derive(Clone)]
pub struct JoinProductIterator<K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
{
  pub v1: Vec<Element<(K, T1), Tag>>,
  pub v2: Vec<Element<(K, T2), Tag>>,
  pub i1: usize,
  pub i2: usize,
}

impl<K, T1, T2, Tag> JoinProductIterator<K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
{
  pub fn new(v1: Vec<Element<(K, T1), Tag>>, v2: Vec<Element<(K, T2), Tag>>) -> Self {
    Self {
      v1,
      v2,
      i1: 0,
      i2: 0,
    }
  }
}

impl<K, T1, T2, Tag> Iterator for JoinProductIterator<K, T1, T2, Tag>
where
  K: Tuple,
  T1: Tuple,
  T2: Tuple,
  Tag: Semiring,
{
  type Item = (Element<(K, T1), Tag>, Element<(K, T2), Tag>);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.i1 < self.v1.len() {
        if self.i2 < self.v2.len() {
          let e1 = &self.v1[self.i1];
          let e2 = &self.v2[self.i2];
          let result = (e1.clone(), e2.clone());
          self.i2 += 1;
          return Some(result);
        } else {
          self.i1 += 1;
          self.i2 = 0;
        }
      } else {
        return None;
      }
    }
  }
}
