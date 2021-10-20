use std::collections::*;
pub use std::iter::FromIterator;

pub type Disjunction = BTreeSet<usize>;

/// Note: Assumes that the #facts > 2
pub(crate) fn has_conflict_in_disjunction(disj: &Disjunction, facts: &BTreeSet<usize>) -> bool {
  // Short cut 1
  if disj.len() < 2 {
    return false;
  }

  // Short cut 2
  let j_last = disj.last().unwrap(); // Note: disj.len >= 2
  let j_first = disj.first().unwrap(); // Note: disj.len >= 2
  let f_last = facts.last().unwrap(); // Note: facts.len >= 2
  let f_first = facts.first().unwrap(); // Note: facts.len >= 2
  if j_last < f_first || f_last < j_first {
    return false;
  }

  // Start iteration
  let mut j_iter = disj.iter();
  let mut f_iter = facts.iter();
  let mut has_same = false;
  let mut j_curr = j_iter.next();
  let mut f_curr = f_iter.next();
  loop {
    match (j_curr, f_curr) {
      (Some(j_elem), Some(f_elem)) => {
        if j_elem == f_elem {
          if has_same {
            return true;
          } else {
            has_same = true;
          }
          j_curr = j_iter.next();
          f_curr = f_iter.next();
        } else if j_elem < f_elem {
          j_curr = j_iter.next();
        } else {
          f_curr = f_iter.next();
        }
      }
      _ => break,
    }
  }
  false
}

pub type Disjunctions = Vec<Disjunction>;

pub(crate) fn has_conflict_in_disjunctions(disjs: &Disjunctions, facts: &BTreeSet<usize>) -> bool {
  // Short hand
  if facts.len() < 2 {
    return false;
  }

  // Check conflict for each disjunction
  for disj in disjs {
    if has_conflict_in_disjunction(disj, facts) {
      return true;
    }
  }
  false
}

#[cfg(test)]
mod test {
  use std::collections::BTreeSet;
  use std::iter::FromIterator;

  use super::*;

  #[test]
  fn test_disjunction_conflict_1() {
    let disj = BTreeSet::from_iter(vec![1, 2, 3]);
    let facts = BTreeSet::from_iter(vec![2, 3, 4]);
    assert!(has_conflict_in_disjunction(&disj, &facts))
  }

  #[test]
  fn test_disjunction_conflict_2() {
    let disj = BTreeSet::from_iter(vec![1, 2, 3]);
    let facts = BTreeSet::from_iter(vec![3, 4, 5]);
    assert!(!has_conflict_in_disjunction(&disj, &facts))
  }
}
