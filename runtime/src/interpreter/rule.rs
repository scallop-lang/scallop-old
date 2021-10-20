use crate::interpreter::*;
use crate::*;

#[derive(Clone, Debug)]
pub struct RuleToAdd {
  /// The temporary dynamic variables to add
  pub tmp_vars_to_add: Vec<(String, TupleType)>,

  /// The facts to add; the relations will be among the tmp_vars
  pub facts_to_add: Vec<(String, DynTuple)>,

  /// The compiled updates to add
  pub updates_to_add: Vec<Update>,
}

#[derive(Clone, Debug)]
pub struct Rule {
  /// The updates corresopnding to this rule
  pub update_ids: Vec<usize>,

  /// The temporary variables corresponding to this rule
  pub tmp_vars: Vec<String>,
}
