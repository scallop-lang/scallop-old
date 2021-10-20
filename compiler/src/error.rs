use super::location::*;
use super::common::*;

#[derive(Clone)]
pub enum CompileError {
  UnknownEmitType,
  UnknownSemiringType,
  NoScallopPathEnvironmentVar,

  // File related
  CannotOpenFile,
  CannotReadFile,

  // Parse errors
  SyntaxError,

  // Compile errors
  FactWithNonConstant {
    loc: Location,
  },
  DuplicatedDeclaration {
    dup: Location,
    rela_name: String,
  },
  UnknownRelation {
    loc: Location,
    rela_name: String,
  },
  IncorrectArity {
    loc: Location,
    rela_name: String,
    found: usize,
    expected: usize,
  },
  ExpressionInBodyLiteral {
    loc: Location,
  },
  ExpressionInQuery {
    loc: Location,
  },
  WildcardInRuleHead {
    loc: Location,
  },
  InvalidWildcard {
    loc: Location,
  },
  DisjunctionHasDifferentRelation {
    loc: Location,
    expected: String,
    found: String,
  },

  UnboundedVariable {
    rule_loc: Location,
    var_loc: Location,
    var_name: String,
  },

  TypeMismatch {
    loc: Location,
    ty: Type,
  },

  TypeUnificationError {
    loc_1: Location,
    loc_2: Location,
    ty_1: Type,
    ty_2: Type,
  },

  CannotInferType,
  CannotUnifyType,
  UnnecessaryIdentityComparison,
  UnnecessaryConstantComparison,
  SomethingStrangeInTypeUnification,

  // Others
  ShouldNotHappen,
  NegationNotImplemented,
  NotImplemented,
}

impl std::fmt::Display for CompileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnknownEmitType => write!(f, "Unknown emit type"),
      Self::UnknownSemiringType => write!(f, "Unknown semiring type"),
      Self::NoScallopPathEnvironmentVar => write!(f, "No SCALLOP_PATH environment variable"),

      // File related
      Self::CannotOpenFile => write!(f, "Cannot open file"),
      Self::CannotReadFile => write!(f, "Cannot read file"),

      // Parse Errors
      Self::SyntaxError => write!(f, "Syntax error"),

      // Compile Errors
      Self::FactWithNonConstant { loc } => {
        write!(f, "[{}] A fact can only contain constants", loc)
      }
      Self::DuplicatedDeclaration { dup, rela_name } => {
        write!(
          f,
          "[{}] Duplicated declaration of relation {}",
          dup, rela_name
        )
      }
      Self::UnknownRelation { loc, rela_name } => {
        write!(f, "[{}] Unknown relation {}", loc, rela_name)
      }
      Self::IncorrectArity {
        loc,
        rela_name,
        found,
        expected,
      } => {
        write!(
          f,
          "[{}] Incorrect arity for relation {}: expected {} arguments, found {}",
          loc, rela_name, expected, found
        )
      }
      Self::TypeMismatch { loc, ty } => {
        write!(f, "[{}] Type mismatch: expected {} type", loc, ty)
      },
      Self::TypeUnificationError { loc_1, loc_2, ty_1, ty_2 } => {
        write!(f, "Cannot unify two types: {} at [{}] and {} at [{}]", ty_1, loc_1, ty_2, loc_2)
      },
      Self::ExpressionInBodyLiteral { loc } => {
        write!(f, "[{}] Disallow expression in body literals", loc)
      }
      Self::ExpressionInQuery { loc } => {
        write!(f, "[{}] Disallow expression in query", loc)
      }
      Self::WildcardInRuleHead { loc } => {
        write!(f, "[{}] Disallow wildcard in rule head", loc)
      }
      Self::InvalidWildcard { loc } => write!(f, "[{}] Invalid wildcard", loc),
      Self::DisjunctionHasDifferentRelation {
        loc,
        expected,
        found,
      } => {
        write!(
          f,
          "[{}] Disjunction has different relation: expected {}, found {}",
          loc, expected, found
        )
      }

      Self::UnboundedVariable {
        var_loc, var_name, ..
      } => {
        write!(f, "[{}] Unbounded variable `{}`", var_loc, var_name)
      }

      Self::CannotInferType => write!(f, "Cannot infer type"),
      Self::CannotUnifyType => write!(f, "Cannot unify type"),
      Self::UnnecessaryIdentityComparison => write!(f, "Unnecessary comparison to itself"),
      Self::UnnecessaryConstantComparison => write!(
        f,
        "Unnecessary constant comparison; consider using directly `true` or `false`"
      ),
      Self::SomethingStrangeInTypeUnification => {
        write!(f, "Something stranage in type unification")
      }

      // Others
      Self::ShouldNotHappen => write!(f, "Should not happen"),
      Self::NegationNotImplemented => write!(f, "Negation is not implemented yet"),
      Self::NotImplemented => write!(f, "Not implemented"),
    }
  }
}

impl std::fmt::Debug for CompileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl std::error::Error for CompileError {}
