#[derive(Clone, Debug, PartialEq)]
pub enum Type {
  Symbol,
  Integer,
  Boolean,
  String,
}

impl std::fmt::Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Symbol => write!(f, "Symbol"),
      Self::Integer => write!(f, "Int"),
      Self::Boolean => write!(f, "Bool"),
      Self::String => write!(f, "String"),
    }
  }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
  Eq,
  Ne,
  Lt,
  Lte,
  Gt,
  Gte,
  And,
  Or,
  Add,
  Sub,
  Mult,
  Div,
}

impl BinaryOp {
  pub fn codify(&self) -> String {
    match self {
      Self::Eq => "==",
      Self::Ne => "!=",
      Self::Lt => "<",
      Self::Lte => "<=",
      Self::Gt => ">",
      Self::Gte => ">=",
      Self::And => "&&",
      Self::Or => "||",
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mult => "*",
      Self::Div => "/",
    }.to_string()
  }
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
  Not,
  Pos,
  Neg,
}

impl UnaryOp {
  pub fn codify(&self) -> String {
    match self {
      Self::Not => "!",
      Self::Pos => "+",
      Self::Neg => "-",
    }.to_string()
  }
}
