#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleId(usize);

impl RuleId {
  pub(crate) fn new(id: usize) -> Self {
    Self(id)
  }

  pub fn raw_id(&self) -> usize {
    self.0
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableId {

}
