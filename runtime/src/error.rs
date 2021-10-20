use scallop_compiler::error::CompileError;

#[derive(Clone, Debug)]
pub enum RuntimeError {
  VariableAlreadyExisted,
  UndefinedVariable,
  CompileError(CompileError),
}

impl std::fmt::Display for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::VariableAlreadyExisted => f.write_str("Error: Variable already existed"),
      Self::UndefinedVariable => f.write_str("Error: Undfined variable"),
      Self::CompileError(e) => {
        f.write_str("Compile error:")?;
        f.write_fmt(format_args!("{}", e))
      }
    }
  }
}
