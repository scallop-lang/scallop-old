use std::collections::{HashMap, HashSet};

use super::dataflows::*;
use super::interpreter::*;
use super::tags::*;
use super::utils::IdAllocator;
use super::*;

#[derive(Default)]
pub struct Iteration<Tag: Semiring> {
  /// The semiring context for all provenance
  pub semiring_ctx: Tag::Context,

  /// The purely static variables that cannot be accessed in the runtime
  variables: Vec<Box<dyn VariableTrait<Tag>>>,

  /// The static variables that run in static time but also accessible in the runtime
  static_variables: HashMap<String, (TupleType, Box<dyn StaticVariableTrait<Tag>>)>,

  /// The dynamic variables that are created in the runtime
  dynamic_variables: HashMap<String, (TupleType, DynVariable<Tag>)>,

  /// The updates
  dynamic_updates: HashMap<usize, Update>,

  /// Book keeping on the rules; each rule will contain a bunch of updates and temporary dynamic variables
  dynamic_rules: HashMap<usize, Rule>,

  /// Compiler context to keep track and compile
  compiler_context: CompilerContext,

  /// Recording how many round has the iteration been going
  round: u32,

  /// New variables store
  new_dynamic_variables: HashSet<String>,

  /// Dynamic rule id allocator; a new rule id will be allocated using this
  dynamic_rule_id_allocator: IdAllocator,

  /// Dynamic update id allocator; a new update id will be allocated using this
  dynamic_update_id_allocator: IdAllocator,
}

impl<Tag: Semiring> Iteration<Tag> {
  /// Create a new Iteration
  pub fn new() -> Self {
    Self {
      // Provenance semiring
      semiring_ctx: Tag::Context::default(),

      // Variables
      variables: Vec::new(),
      static_variables: HashMap::new(),
      dynamic_variables: HashMap::new(),

      // Dynamic updates and rules
      dynamic_updates: HashMap::new(),
      dynamic_rules: HashMap::new(),
      compiler_context: CompilerContext::new(),

      // Round counter
      round: u32::default(),

      // New variables
      new_dynamic_variables: HashSet::new(),

      // Temporary counters
      dynamic_rule_id_allocator: IdAllocator::new(),
      dynamic_update_id_allocator: IdAllocator::new(),
    }
  }

  /// Deligate `changed` to all of the children variables, including static
  /// variables and dynamic variables. At a high level, it will update the
  /// variables and check if any of the variables has been changed.
  ///
  /// It will move all the `to_add` elements into `recent`, and move all the
  /// `recent` to `stable`.
  ///
  /// It returns true if there is any change to any variable. This denotes
  /// that we have not reached a fix-point and the semi-naive evaluation has
  /// to continue.
  pub fn changed(&mut self) -> bool {
    self.round += 1;
    let mut result = false;
    for variable in self.variables.iter_mut() {
      if variable.changed(&self.semiring_ctx) {
        result = true;
      }
    }
    for (_, (_, variable)) in self.static_variables.iter_mut() {
      if variable.changed(&self.semiring_ctx) {
        result = true;
      }
    }
    for (_, (_, variable)) in self.dynamic_variables.iter_mut() {
      if variable.changed(&self.semiring_ctx) {
        result = true;
      }
    }
    result
  }

  /// Create a new variable with the given `Tup` type.
  pub fn variable<Tup>(&mut self) -> Variable<Tup, Tag>
  where
    Tup: Tuple + 'static,
  {
    let variable = Variable::new();
    self.variables.push(Box::new(variable.clone()));
    variable
  }

  pub fn variable_handle<'a, Tup>(
    &'a mut self,
    var: &'a Variable<Tup, Tag>,
  ) -> VariableHandle<'a, Tup, Tag>
  where
    Tup: Tuple,
  {
    VariableHandle::new(var, &mut self.semiring_ctx)
  }

  pub fn static_variable<Tup>(&mut self, name: &str) -> Variable<Tup, Tag>
  where
    Tup: Tuple + 'static,
    TupleType: FromType<Tup>,
    Tup: Into<DynTuple>,
  {
    let variable = Variable::new();
    let var_type = <TupleType as FromType<Tup>>::from_type();
    self.static_variables.insert(
      name.to_string(),
      (var_type.clone(), Box::new(variable.clone())),
    );
    self
      .compiler_context
      .add_variable(name, VariableKind::Static, var_type);
    variable
  }

  pub fn static_variable_handle<'a, Tup>(
    &'a mut self,
    var: &'a Variable<Tup, Tag>,
  ) -> VariableHandle<'a, Tup, Tag>
  where
    Tup: Tuple,
  {
    VariableHandle::new(var, &mut self.semiring_ctx)
  }

  pub fn get_static_variable<'a>(&'a self, name: &str) -> StaticVariable<'a, Tag> {
    StaticVariable(&*self.static_variables[name].1)
  }

  pub fn dynamic_variable(&mut self, name: &str, tuple_type: TupleType) -> Option<DynVariable<Tag>> {
    if self.dynamic_variables.contains_key(name) {
      return None;
    }

    // Then generate the variable
    let variable = DynVariable::new();

    // First add dynamic variables
    self
      .dynamic_variables
      .insert(name.to_string(), (tuple_type.clone(), variable.clone()));

    // Then insert the name to the new dynamic variables
    self.new_dynamic_variables.insert(name.to_string());

    // Finally add the variable to the compiler context
    self
      .compiler_context
      .add_variable(name, VariableKind::Dynamic, tuple_type);

    // Return the variable
    Some(variable)
  }

  pub fn get_dynamic_variable(&self, name: &str) -> Option<&DynVariable<Tag>> {
    self.dynamic_variables.get(name).map(|var| &var.1)
  }

  pub fn remove_dynamic_variable(&mut self, name: &str) -> bool {
    self.compiler_context.remove_variable(name);
    self.new_dynamic_variables.remove(name);

    // The result of this function is whether there is such named dynamic variable
    self.dynamic_variables.remove(name).is_some()
  }

  pub fn add_dynamic_update(&mut self, update: Update) -> usize {
    let id = self.dynamic_update_id_allocator.allocate();
    self.dynamic_updates.insert(id, update);
    id
  }

  pub fn remove_dynamic_update(&mut self, update_id: usize) -> bool {
    self.dynamic_updates.remove(&update_id).is_some()
  }

  pub fn perform_dynamic_updates(&self) {
    for (_, update) in &self.dynamic_updates {
      // Get the dataflow
      let target_dyn_var = &self.dynamic_variables[&update.target].1;
      let dataflow = self.flow_to_dynamic_dataflow(&update.flow);

      // If target is a new variable, insert the stable batches
      if self.new_dynamic_variables.contains(&update.target) {
        target_dyn_var.insert_stable(&self.semiring_ctx, &dataflow);
      }

      // Insert the recent batches
      target_dyn_var.insert(&self.semiring_ctx, &dataflow);
    }
  }

  pub fn has_new_variable(&self) -> bool {
    !self.new_dynamic_variables.is_empty()
  }

  pub fn clear_new_variables(&mut self) {
    self.new_dynamic_variables.clear();
  }

  fn flow_to_dynamic_dataflow<'a>(&'a self, flow: &Flow) -> DynDataflow<'a, Tag> {
    match flow {
      Flow::Product(f1, f2) => DynDataflow::Product {
        i1: Box::new(self.flow_to_dynamic_dataflow(&f1)),
        i2: Box::new(self.flow_to_dynamic_dataflow(&f2)),
        ctx: &self.semiring_ctx,
      },
      Flow::Intersect(f1, f2) => DynDataflow::Intersection {
        d1: Box::new(self.flow_to_dynamic_dataflow(&f1)),
        d2: Box::new(self.flow_to_dynamic_dataflow(&f2)),
        ctx: &self.semiring_ctx,
      },
      Flow::Join(f1, f2) => DynDataflow::Join {
        d1: Box::new(self.flow_to_dynamic_dataflow(&f1)),
        d2: Box::new(self.flow_to_dynamic_dataflow(&f2)),
        ctx: &self.semiring_ctx,
      },
      Flow::Filter(f, e) => DynDataflow::Filter {
        source: Box::new(self.flow_to_dynamic_dataflow(&f)),
        expression: e.clone(),
      },
      Flow::Project(f, e) => DynDataflow::Projection {
        source: Box::new(self.flow_to_dynamic_dataflow(&f)),
        expression: e.clone(),
      },
      Flow::Find(f, c) => DynDataflow::Find {
        source: Box::new(self.flow_to_dynamic_dataflow(&f)),
        key: c.clone(),
      },
      Flow::ContainsChain(source, key, other) => DynDataflow::Contains {
        d1: Box::new(self.flow_to_dynamic_dataflow(&source)),
        key: key.clone(),
        d2: Box::new(self.flow_to_dynamic_dataflow(&other)),
        ctx: &self.semiring_ctx,
      },
      Flow::StaticVariable(name) => {
        DynDataflow::StaticVariable(self.get_static_variable(&name))
      },
      Flow::DynamicVariable(name) => {
        DynDataflow::Variable(self.get_dynamic_variable(&name).unwrap())
      },
    }
  }

  pub fn add_rule(&mut self, rule_str: &str) -> Result<RuleId, DynCompileError> {
    let rule_to_add = self.compiler_context.compile_rule_from_str(rule_str)?;
    Ok(RuleId::new(self.process_rule_to_add(rule_to_add)))
  }

  pub fn remove_rule(&mut self, rule_id: RuleId) -> bool {
    match self.dynamic_rules.remove(&rule_id.raw_id()) {
      Some(rule) => {
        for update_id in rule.update_ids {
          self.remove_dynamic_update(update_id);
        }
        for tmp_var in rule.tmp_vars {
          self.remove_dynamic_variable(&tmp_var);
        }
        true
      }
      None => false,
    }
  }

  fn process_rule_to_add(&mut self, rule: RuleToAdd) -> usize {
    // Create a rule id
    let rule_id = self.dynamic_rule_id_allocator.allocate();

    // Add temporary variables
    let tmp_vars = rule
      .tmp_vars_to_add
      .into_iter()
      .map(|(name, var_type)| {
        self.dynamic_variable(&name, var_type);
        name
      })
      .collect::<Vec<_>>();

    // Add all the updates
    let update_ids = rule
      .updates_to_add
      .into_iter()
      .map(|update| self.add_dynamic_update(update))
      .collect::<Vec<_>>();

    // Add all the facts
    for (var_name, fact) in rule.facts_to_add {
      let elements = vec![DynElement {
        tup: fact,
        tag: Tag::one(&self.semiring_ctx),
      }];
      self.get_dynamic_variable(&var_name).unwrap().insert(
        &self.semiring_ctx,
        &DynDataflow::Vec(&elements),
      );
    }

    // Add rule
    self.dynamic_rules.insert(rule_id.clone(), Rule {
      tmp_vars,
      update_ids,
    });

    // Return rule id
    rule_id
  }

  pub fn insert<Tup>(&mut self, var: &Variable<Tup, Tag>, data: Vec<Tup>)
  where
    Tup: Tuple,
    <Tag as Semiring>::Context: SemiringContext<Tag, Info = ()>,
  {
    let data = data
      .into_iter()
      .map(|tup| Element {
        tup,
        tag: self.semiring_ctx.base_tag(()),
      })
      .collect::<Vec<_>>();
    var.insert(&self.semiring_ctx, data)
  }

  pub fn insert_ground<Tup>(&self, var: &Variable<Tup, Tag>, data: Vec<Tup>)
  where
    Tup: Tuple,
  {
    let data = data
      .into_iter()
      .map(|tup| Element {
        tup,
        tag: Tag::one(&self.semiring_ctx),
      })
      .collect::<Vec<_>>();
    var.insert(&self.semiring_ctx, data)
  }

  pub fn insert_with_tag_info<Tup>(
    &mut self,
    var: &Variable<Tup, Tag>,
    data: Vec<(<Tag::Context as SemiringContext<Tag>>::Info, Tup)>,
  ) where
    Tup: Tuple,
  {
    var.insert_with_context(&mut self.semiring_ctx, data)
  }

  pub fn insert_dataflow<D, Tup>(&self, var: &Variable<Tup, Tag>, data: D)
  where
    D: Dataflow<Tup, Tag>,
    Tup: Tuple,
  {
    var.insert(&self.semiring_ctx, data)
  }

  pub fn product<D1, D2, T1, T2>(&self, v1: D1, v2: D2) -> Product<D1, D2, T1, T2, Tag>
  where
    T1: Tuple,
    T2: Tuple,
    D1: Dataflow<T1, Tag>,
    D2: Dataflow<T2, Tag>,
  {
    product(v1, v2, &self.semiring_ctx)
  }

  pub fn intersect<D1, D2, Tup>(&self, v1: D1, v2: D2) -> Intersection<D1, D2, Tup, Tag>
  where
    Tup: Tuple,
    D1: Dataflow<Tup, Tag>,
    D2: Dataflow<Tup, Tag>,
  {
    intersect(v1, v2, &self.semiring_ctx)
  }

  pub fn union<D1, D2, Tup>(&self, v1: D1, v2: D2) -> Union<D1, D2, Tup, Tag>
  where
    Tup: Tuple,
    D1: Dataflow<Tup, Tag>,
    D2: Dataflow<Tup, Tag>,
  {
    union(v1, v2, &self.semiring_ctx)
  }

  pub fn join<D1, D2, K, T1, T2>(&self, v1: D1, v2: D2) -> Join<D1, D2, K, T1, T2, Tag>
  where
    K: Tuple,
    T1: Tuple,
    T2: Tuple,
    D1: Dataflow<(K, T1), Tag>,
    D2: Dataflow<(K, T2), Tag>,
  {
    join(v1, v2, &self.semiring_ctx)
  }

  pub fn difference<D1, D2, Tup>(&self, v1: D1, v2: D2) -> Difference<D1, D2, Tup, Tag>
  where
    Tup: Tuple,
    Tag: SemiringWithDifference,
    D1: Dataflow<Tup, Tag>,
    D2: Dataflow<Tup, Tag>,
  {
    difference(v1, v2, &self.semiring_ctx)
  }

  pub fn antijoin<D1, D2, K, T1, T2>(&self, v1: D1, v2: D2) -> Antijoin<D1, D2, K, T1, T2, Tag>
  where
    K: Tuple,
    T1: Tuple,
    T2: Tuple,
    Tag: SemiringWithDifference,
    D1: Dataflow<(K, T1), Tag>,
    D2: Dataflow<(K, T2), Tag>,
  {
    antijoin(v1, v2, &self.semiring_ctx)
  }

  pub fn contains_chain<D1, D2, T1, T2>(
    &self,
    d1: D1,
    key: T1,
    d2: D2,
  ) -> ContainsChain<D1, D2, T1, T2, Tag>
  where
    T1: Tuple,
    T2: Tuple,
    D1: Dataflow<T1, Tag>,
    D2: Dataflow<T2, Tag>,
  {
    contains_chain(d1, key, d2, &self.semiring_ctx)
  }

  pub fn complete<Tup>(&self, var: &Variable<Tup, Tag>) -> Relation<Tup, Tag>
  where
    Tup: Tuple,
  {
    var.complete(&self.semiring_ctx)
  }
}

impl<Tag> Iteration<Tag>
where
  ProbProofContext: SemiringContext<Tag>,
  Tag: Semiring<Context = ProbProofContext>,
{
  pub fn insert_disjunction<Tup>(
    &mut self,
    var: &Variable<Tup, Tag>,
    data: Vec<(<ProbProofContext as SemiringContext<Tag>>::Info, Tup)>,
  ) where
    Tup: Tuple,
  {
    let id = self.semiring_ctx.id_counter;
    self
      .semiring_ctx
      .disjunctions
      .push((id..id + data.len()).collect());
    var.insert_with_context(&mut self.semiring_ctx, data);
  }
}
