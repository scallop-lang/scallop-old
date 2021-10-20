use super::iteration::*;
use super::semiring::*;
use super::tuple::*;
use super::interpreter::*;
use super::error::*;

pub trait Program<Tag>
where
  Tag: Semiring,
{
  /// Create a new Program
  fn new() -> Self;

  /// Obtain the iteration
  fn iteration(&self) -> &Iteration<Tag>;

  /// Obtain the mutable iteration
  fn iteration_mut(&mut self) -> &mut Iteration<Tag>;

  /// Get the semiring context from the iteration
  fn semiring_context(&self) -> &Tag::Context {
    &self.iteration().semiring_ctx
  }

  /// Add a dynamic variable with a given tuple type
  fn add_variable(&mut self, name: &str, tup_type: TupleType) -> Result<(), RuntimeError> {
    match self.iteration_mut().dynamic_variable(name, tup_type) {
      Some(_) => Ok(()),
      None => Err(RuntimeError::VariableAlreadyExisted),
    }
  }

  /// Get a dynamic variable given the name
  fn get_variable<'a>(&'a mut self, name: &str) -> Result<DynVariableHandle<'a, Tag>, RuntimeError> {
    match self.iteration_mut().get_dynamic_variable(name) {
      Some(var) => {
        let var = var.clone();
        let ctx = &mut self.iteration_mut().semiring_ctx;
        Ok(DynVariableHandle::new(var, ctx))
      },
      None => Err(RuntimeError::UndefinedVariable)
    }
  }

  /// Remove a dynamic variable using the name
  fn remove_variable(&mut self, name: &str) -> Result<(), RuntimeError> {
    if self.iteration_mut().remove_dynamic_variable(name) {
      Ok(())
    } else {
      Err(RuntimeError::UndefinedVariable)
    }
  }

  /// Dynamically compile a rule (in its string form)
  ///
  /// Returns a result containing the rule id (`usize`), which could be later
  /// used to remove the rule.
  fn add_rule(&mut self, rule_str: &str) -> Result<RuleId, RuntimeError> {
    self.iteration_mut().add_rule(rule_str).map_err(|e| {
      let DynCompileError::CompileError(e) = e;
      RuntimeError::CompileError(e)
    })
  }

  /// Remove a compiled rule using its rule id obtained from `add_rule`
  fn remove_rule(&mut self, rule_id: RuleId) -> Result<(), RuntimeError> {
    if self.iteration_mut().remove_rule(rule_id) {
      Ok(())
    } else {
      Err(RuntimeError::UndefinedVariable)
    }
  }

  /// Initialize the
  fn initialize(&mut self) {}

  /// The static update function of program
  fn update(&self) {}

  /// Run the program
  fn run(&mut self) {
    // First initialize the program
    self.initialize();

    // Enter the main loop; will execute if there is new variable or the iteration has been changed
    while self.iteration().has_new_variable() || self.iteration_mut().changed() {

      // First call the update function
      self.update();

      // Then perform the dynamic update
      self.iteration().perform_dynamic_updates();

      // Clear the new variable after the first
      self.iteration_mut().clear_new_variables();
    }
  }
}

pub struct EmptyProgram<Tag: Semiring> {
  iter: Iteration<Tag>,
}

impl<Tag: Semiring> Program<Tag> for EmptyProgram<Tag> {
  fn new() -> Self {
    Self { iter: Iteration::new() }
  }

  fn iteration(&self) -> &Iteration<Tag> {
    &self.iter
  }

  fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
    &mut self.iter
  }
}
