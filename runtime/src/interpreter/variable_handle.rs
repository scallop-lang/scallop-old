use super::*;
use crate::*;

pub struct DynVariableHandle<'a, Tag: Semiring> {
  var: DynVariable<Tag>,
  ctx: &'a mut Tag::Context,
}

impl<'a, Tag: Semiring> DynVariableHandle<'a, Tag> {
  pub fn new(var: DynVariable<Tag>, ctx: &'a mut Tag::Context) -> Self {
    Self { var, ctx }
  }

  pub fn complete(&self) -> DynRelation<Tag> {
    self.var.complete(self.ctx)
  }
}
