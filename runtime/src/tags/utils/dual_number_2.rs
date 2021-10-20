/// The sparse gradient of a dual number
#[derive(Debug, Clone)]
pub enum DualNumber2Grad {
  /// A zero gradient ([..., 0, ...])
  Zero,

  /// A one hot gradient ([0, ..., f, ..., 0]), where
  /// the probability f is at the index i, and the whole gradient is
  /// of dimension size
  OneHot {
    i: usize,
    f: f64,
    size: usize,
  },

  /// A complete version the of gradient
  Full(Vec<f64>),
}

impl DualNumber2Grad {
  /// Get the i-th element of the gradient
  pub fn i(&self, i: &usize) -> f64 {
    match self {
      Self::Zero => 0.0,
      Self::OneHot { i: j, f, .. } => if i == j { f.clone() } else { 0.0 },
      Self::Full(v) => v[i.clone()].clone(),
    }
  }
}

impl std::ops::Mul<f64> for DualNumber2Grad {
  type Output = Self;

  /// Multiply a gradient with a scalar value
  fn mul(self, other: f64) -> Self {
    // Shortcut to multiplying zero
    if other == 0.0 {
      return Self::Zero;
    }

    // Check details
    match self {
      Self::Zero => Self::Zero,
      Self::OneHot { i, f: f_i, size } => {
        Self::OneHot { i, f: f_i * other, size }
      },
      Self::Full(vs) => {
        Self::Full(vs.into_iter().map(|v| v * other).collect::<Vec<_>>())
      },
    }
  }
}

impl std::ops::Add for DualNumber2Grad {
  type Output = Self;

  /// Add two gradients
  fn add(self, other: Self) -> Self {
    match (self, other) {
      // 0 + d2 = d2
      (Self::Zero, d2) => d2,

      // d1 + 0 = 0
      (d1, Self::Zero) => d1,

      //   [0....f1....0] + [0...f2.....0]
      // = [0...f2..f1...0]
      (Self::OneHot { i: i1, f: f1, size: s1 }, Self::OneHot { i: i2, f: f2, size: s2 }) => {
        assert_eq!(s1, s2);
        Self::Full((0..s1).map(|i| {
          let is_i1 = i == i1;
          let is_i2 = i == i2;
          if is_i1 && is_i2 { f1 + f2 }
          else if is_i1 { f1 }
          else if is_i2 { f2 }
          else { 0.0 }
        }).collect::<Vec<_>>())
      },

      // One hot + full
      (Self::OneHot { i, f, size }, Self::Full(vs)) |
      (Self::Full(vs), Self::OneHot { i, f, size }) => {
        assert_eq!(vs.len(), size);
        Self::Full((0..size).map(|j| {
          if j == i { vs[j] + f }
          else { vs[j] }
        }).collect::<Vec<_>>())
      },

      // Full + full
      (Self::Full(vs1), Self::Full(vs2)) => {
        assert_eq!(vs1.len(), vs2.len());
        Self::Full(vs1.iter().zip(vs2.iter()).map(|(v1, v2)| {
          v1 + v2
        }).collect::<Vec<_>>())
      }
    }
  }
}

impl std::ops::Neg for DualNumber2Grad {
  type Output = Self;

  /// Negate a gradient
  fn neg(self) -> Self {
    match self {
      Self::Zero => Self::Zero,
      Self::OneHot { i, f, size } => {
        Self::OneHot { i, f: -f, size }
      },
      Self::Full(vs) => {
        Self::Full(vs.into_iter().map(|v| -v).collect::<Vec<_>>())
      },
    }
  }
}

/// A dual number (p, \nabla_p), the real part and its gradient
#[derive(Debug, Clone)]
pub struct DualNumber2(f64, DualNumber2Grad);

impl DualNumber2 {
  /// Create a new dual number with its real part, and the index of itself being in its gradient
  pub fn new(prob: f64, i: usize, size: usize) -> Self {
    Self(prob, DualNumber2Grad::OneHot { i, f: 1.0, size })
  }

  /// Get the probability (real part) of this dual number
  pub fn prob(&self) -> f64 {
    self.0.clone()
  }

  /// Get the i-th component of its gradient
  pub fn ith_grad(&self, i: &usize) -> f64 {
    self.1.i(i)
  }

  /// Create a zero dual number -- the real part will be zero and the gradient will also be
  /// zero (since it's a constant)
  pub fn zero() -> Self {
    Self(0.0, DualNumber2Grad::Zero)
  }

  /// Create a one dual number -- the real part will be 1 and the gradient will be zero
  /// (since it's a constant)
  pub fn one() -> Self {
    Self(1.0, DualNumber2Grad::Zero)
  }

  /// Add two dual numbers
  ///
  /// (p1, \n_p1) + (p2, \n_p2) = (p1 + p2, \n_p1 + \n_p2)
  pub fn add(&self, other: &Self) -> Self {
    Self(self.0 + other.0, self.1.clone() + other.1.clone())
  }

  /// Multiply two dual numbers
  ///
  /// (p1, \n_p1) * (p2, \n_p2) = (p1 * p2, \n_p1 * p2 + \n_p2 * p1)
  pub fn mult(&self, other: &Self) -> Self {
    Self(self.0 * other.0, self.1.clone() * other.0 + other.1.clone() * self.0)
  }

  /// Negate a (probabilistic) dual number
  ///
  /// -(p, \n_p) = (1 - p, -\n_p)
  pub fn negate(&self) -> Self {
    Self(1.0 - self.0, -self.1.clone())
  }
}
