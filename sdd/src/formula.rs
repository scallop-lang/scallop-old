use std::collections::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanFormula {
  False,
  True,
  Pos {
    var_id: usize,
  },
  Neg {
    var_id: usize,
  },
  Not {
    form: Box<BooleanFormula>,
  },
  And {
    left: Box<BooleanFormula>,
    right: Box<BooleanFormula>,
  },
  Or {
    left: Box<BooleanFormula>,
    right: Box<BooleanFormula>,
  },
}

/// Create a boolean formula literal using var_id
pub fn bf(var_id: usize) -> BooleanFormula {
  BooleanFormula::Pos { var_id }
}

impl std::ops::BitAnd for BooleanFormula {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self {
    Self::And {
      left: Box::new(self),
      right: Box::new(rhs),
    }
  }
}

impl std::ops::BitOr for BooleanFormula {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self {
    Self::Or {
      left: Box::new(self),
      right: Box::new(rhs),
    }
  }
}

impl std::ops::Not for BooleanFormula {
  type Output = Self;

  fn not(self) -> Self {
    match self {
      Self::Pos { var_id } => Self::Neg { var_id },
      other => Self::Not { form: Box::new(other) },
    }
  }
}

impl BooleanFormula {
  pub fn collect_vars(&self) -> Vec<usize> {
    let mut set = BTreeSet::new();
    self.collect_vars_helper(&mut set);
    set.into_iter().collect()
  }

  fn collect_vars_helper(&self, collection: &mut BTreeSet<usize>) {
    match self {
      Self::True | Self::False => {},
      Self::Pos { var_id } => {
        collection.insert(*var_id);
      }
      Self::Neg { var_id } => {
        collection.insert(*var_id);
      }
      Self::Not { form } => {
        form.collect_vars_helper(collection);
      },
      Self::And { left, right } => {
        left.collect_vars_helper(collection);
        right.collect_vars_helper(collection);
      }
      Self::Or { left, right } => {
        left.collect_vars_helper(collection);
        right.collect_vars_helper(collection);
      }
    }
  }
}
