use rayon::prelude::*;

use super::*;
use super::tags::*;
use super::wmc::*;

pub struct VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  var: &'a Variable<Tup, Tag>,
  semiring_ctx: &'a mut Tag::Context,
  temp_storage: Vec<Element<Tup, Tag>>,
}

impl<'a, Tup, Tag> VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  pub fn new(var: &'a Variable<Tup, Tag>, semiring_ctx: &'a mut Tag::Context) -> Self {
    let temp_storage = vec![];
    Self {
      var,
      semiring_ctx,
      temp_storage,
    }
  }

  pub fn insert(&self, data: Vec<Tup>)
  where
    <Tag as Semiring>::Context: SemiringContext<Tag, Info = ()>,
  {
    let data = data
      .into_iter()
      .map(|tup| Element {
        tup,
        tag: Tag::one(&self.semiring_ctx),
      })
      .collect::<Vec<_>>();
    self.var.insert(&self.semiring_ctx, data)
  }

  pub fn insert_one_ground(&mut self, tup: Tup) {
    let elem = Element {
      tup,
      tag: Tag::one(&self.semiring_ctx),
    };
    self.temp_storage.push(elem)
  }

  pub fn insert_ground(&self, data: Vec<Tup>) {
    let data = data
      .into_iter()
      .map(|tup| Element {
        tup,
        tag: Tag::one(&self.semiring_ctx),
      })
      .collect::<Vec<_>>();
    self.var.insert(&self.semiring_ctx, data)
  }

  pub fn insert_one_with_tag_info(
    &mut self,
    tag_info: <Tag::Context as SemiringContext<Tag>>::Info,
    tup: Tup,
  ) {
    let elem = Element {
      tup,
      tag: self.semiring_ctx.base_tag(tag_info),
    };
    self.temp_storage.push(elem)
  }

  pub fn insert_with_tag_info(
    &mut self,
    data: Vec<(<Tag::Context as SemiringContext<Tag>>::Info, Tup)>,
  ) {
    self.var.insert_with_context(&mut self.semiring_ctx, data)
  }

  pub fn complete(self) -> Relation<Tup, Tag> {
    self.var.complete(self.semiring_ctx)
  }
}

impl<'a, Tup, Tag> VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  pub fn complete_with_wmc<Wmc>(
    self,
    wmc: &Wmc,
  ) -> Vec<(Tup, Tag, <Wmc as WeightedModelCounter>::Output)>
  where
    Wmc: WeightedModelCounter<Tag = Tag>,
  {
    self.var.complete(self.semiring_ctx).into_iter().into_iter().map(|elem| {
      let wmc_result = wmc.wmc(self.semiring_ctx, &elem.tag);
      (elem.tup, elem.tag, wmc_result)
    }).collect::<Vec<_>>()
  }

  pub fn par_complete_with_wmc<Wmc>(
    self,
    wmc: &Wmc,
  ) -> Vec<(Tup, Tag, <Wmc as WeightedModelCounter>::Output)>
  where
    Wmc: WeightedModelCounter<Tag = Tag>,
  {
    let elements = self.var.complete(self.semiring_ctx).elements;
    par_process_elements(elements, &*self.semiring_ctx, wmc)
  }
}

fn par_process_elements<Tup, Tag, Wmc>(
  elements: Vec<Element<Tup, Tag>>,
  semiring_ctx: &Tag::Context,
  wmc: &Wmc,
) -> Vec<(Tup, Tag, <Wmc as WeightedModelCounter>::Output)>
where
  Tup: Tuple,
  Tag: Semiring,
  Wmc: WeightedModelCounter<Tag = Tag>,
{
  elements.into_par_iter().map(|elem| {
    let wmc_result = wmc.wmc(semiring_ctx, &elem.tag);
    (elem.tup, elem.tag, wmc_result)
  }).collect::<Vec<_>>()
}

impl<'a, Tup, Tag> VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  ProbProofContext: SemiringContext<Tag>,
  Tag: Semiring<Context = ProbProofContext>,
{
  pub fn insert_disjunction(
    &mut self,
    data: Vec<(<ProbProofContext as SemiringContext<Tag>>::Info, Tup)>,
  ) {
    let id = self.semiring_ctx.id_counter;
    self
      .semiring_ctx
      .disjunctions
      .push((id..id + data.len()).collect());
    self.var.insert_with_context(&mut self.semiring_ctx, data);
  }
}

#[cfg(feature = "torch")]
impl<'a, Tup, Tag> VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
  DiffProbProofContext: SemiringContext<Tag>,
  Tag: Semiring<Context = DiffProbProofContext>,
{
  pub fn insert_diff_disjunction(
    &mut self,
    data: Vec<(<DiffProbProofContext as SemiringContext<Tag>>::Info, Tup)>,
  ) {
    let id = self.semiring_ctx.id_counter;
    self
      .semiring_ctx
      .disjunctions
      .push((id..id + data.len()).collect());
    self.var.insert_with_context(&mut self.semiring_ctx, data);
  }
}

impl<'a, Tup, Tag> Drop for VariableHandle<'a, Tup, Tag>
where
  Tup: Tuple,
  Tag: Semiring,
{
  fn drop(&mut self) {
    self
      .var
      .insert(self.semiring_ctx, self.temp_storage.clone())
  }
}
