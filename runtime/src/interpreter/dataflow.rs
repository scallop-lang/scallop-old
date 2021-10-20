use std::cell::Ref;

use super::*;
use crate::*;

#[derive(Clone)]
pub enum DynDataflow<'a, Tag: Semiring> {
  /// Vector dataflow
  Vec(&'a Vec<DynElement<Tag>>),

  /// Relation dataflow
  Relation(&'a DynRelation<Tag>),

  /// Variable dataflow
  Variable(&'a DynVariable<Tag>),

  /// Static dataflow
  StaticVariable(StaticVariable<'a, Tag>),

  /// Projection dataflow
  Projection {
    source: Box<DynDataflow<'a, Tag>>,
    expression: Expression,
  },

  /// Filter dataflow
  Filter {
    source: Box<DynDataflow<'a, Tag>>,
    expression: Expression,
  },

  /// Find dataflow
  Find {
    source: Box<DynDataflow<'a, Tag>>,
    key: DynTuple,
  },

  /// Contains dataflow
  Contains {
    d1: Box<DynDataflow<'a, Tag>>,
    key: DynTuple,
    d2: Box<DynDataflow<'a, Tag>>,
    ctx: &'a Tag::Context,
  },

  /// Product dataflow
  Product {
    i1: Box<DynDataflow<'a, Tag>>,
    i2: Box<DynDataflow<'a, Tag>>,
    ctx: &'a Tag::Context,
  },

  /// Intersection dataflow
  Intersection {
    d1: Box<DynDataflow<'a, Tag>>,
    d2: Box<DynDataflow<'a, Tag>>,
    ctx: &'a Tag::Context,
  },

  /// Join dataflow
  Join {
    d1: Box<DynDataflow<'a, Tag>>,
    d2: Box<DynDataflow<'a, Tag>>,
    ctx: &'a Tag::Context,
  },
}

impl<'a, Tag: Semiring> DynDataflow<'a, Tag> {
  pub fn iter_stable(&self) -> DynDataflowBatches<'a, Tag> {
    match self {
      // A vector has no stable
      Self::Vec(_) => DynDataflowBatches::Empty,

      // A relation has no stable
      Self::Relation(_) => DynDataflowBatches::Empty,

      // A variable's stable is its stable relations
      Self::Variable(v) => DynDataflowBatches::variable_stable(v),

      // Static variable
      Self::StaticVariable(v) => DynDataflowBatches::StaticVariableStable {
        variable: v.clone(),
        rela_id: 0,
      },

      // A projection's stable is its source's stable
      Self::Projection { source, expression } => DynDataflowBatches::map(
        source.iter_stable(),
        BatchUnaryOp::Projection(expression.clone()),
      ),

      // A filter's stable is its source's stable
      Self::Filter { source, expression } => DynDataflowBatches::map(
        source.iter_stable(),
        BatchUnaryOp::Filter(expression.clone()),
      ),

      // A find's stable is its source's stable
      Self::Find { source, key } => {
        DynDataflowBatches::map(source.iter_stable(), BatchUnaryOp::Find(key.clone()))
      }

      Self::Contains { d1, key, d2, ctx } => {
        for b1 in d1.iter_stable() {
          if let Some(tag) = batch_key_search(b1, &key) {
            let op = BatchUnaryOp::MergeTag { tag, ctx };
            return DynDataflowBatches::some(DynDataflowBatches::map(d2.iter_stable(), op));
          }
        }
        DynDataflowBatches::none()
      }

      // A product's stable is a join on its sources' stable
      Self::Product { i1, i2, ctx } => DynDataflowBatches::join(
        i1.iter_stable(),
        i2.iter_stable(),
        BatchBinaryOp::Product { ctx },
      ),

      // An intersection's stable is a join on its sources' stable
      Self::Intersection { d1, d2, ctx } => DynDataflowBatches::join(
        d1.iter_stable(),
        d2.iter_stable(),
        BatchBinaryOp::Intersection { ctx },
      ),

      Self::Join { d1, d2, ctx } => DynDataflowBatches::join(
        d1.iter_stable(),
        d2.iter_stable(),
        BatchBinaryOp::Join { ctx },
      ),
    }
  }

  pub fn iter_recent<'b>(&'b self) -> DynDataflowBatches<'a, Tag> {
    match self {
      // A vector's recent is itself
      Self::Vec(vs) => DynDataflowBatches::single(DynDataflowBatch::vec(vs)),

      // A relation's recent is itself
      Self::Relation(rela) => DynDataflowBatches::single(DynDataflowBatch::vec(&rela.elements)),

      // A variable's recent is its recent
      Self::Variable(v) => DynDataflowBatches::single(DynDataflowBatch::variable_recent(v)),

      // Static variable
      Self::StaticVariable(v) => {
        DynDataflowBatches::single(DynDataflowBatch::StaticVariableRecent {
          variable: v.clone(),
          elem_id: 0,
        })
      }

      // A projection's recent is its source's recent
      Self::Projection { source, expression } => DynDataflowBatches::map(
        source.iter_recent(),
        BatchUnaryOp::Projection(expression.clone()),
      ),

      // A filter's recent is its source's recent
      Self::Filter { source, expression } => DynDataflowBatches::map(
        source.iter_recent(),
        BatchUnaryOp::Filter(expression.clone()),
      ),

      // A find
      Self::Find { source, key } => {
        DynDataflowBatches::map(source.iter_recent(), BatchUnaryOp::Find(key.clone()))
      }

      Self::Contains { d1, key, d2, ctx } => {
        for b1 in d1.iter_recent() {
          if let Some(tag) = batch_key_search(b1, key) {
            let op = BatchUnaryOp::MergeTag { tag, ctx };
            let chain = DynDataflowBatches::chain(d2.iter_stable(), d2.iter_recent());
            return DynDataflowBatches::map(chain, op);
          }
        }
        for b1 in d1.iter_stable() {
          if let Some(tag) = batch_key_search(b1, key) {
            let op = BatchUnaryOp::MergeTag { tag, ctx };
            return DynDataflowBatches::map(d2.iter_recent(), op);
          }
        }
        return DynDataflowBatches::Empty;
      }

      Self::Product { i1, i2, ctx } => {
        let op = BatchBinaryOp::Product { ctx: ctx.clone() };
        let i1_stable = i1.iter_stable();
        let i2_stable = i2.iter_stable();
        let i1_recent = i1.iter_recent();
        let i2_recent = i2.iter_recent();
        DynDataflowBatches::chain(
          DynDataflowBatches::chain(
            DynDataflowBatches::join(i1_recent.clone(), i2_stable, op.clone()),
            DynDataflowBatches::join(i1_stable, i2_recent.clone(), op.clone()),
          ),
          DynDataflowBatches::join(i1_recent, i2_recent, op.clone()),
        )
      }

      Self::Intersection { d1, d2, ctx } => {
        let op = BatchBinaryOp::Intersection { ctx: ctx.clone() };
        let i1_stable = d1.iter_stable();
        let i2_stable = d2.iter_stable();
        let i1_recent = d1.iter_recent();
        let i2_recent = d2.iter_recent();
        DynDataflowBatches::chain(
          DynDataflowBatches::chain(
            DynDataflowBatches::join(i1_recent.clone(), i2_stable, op.clone()),
            DynDataflowBatches::join(i1_stable, i2_recent.clone(), op.clone()),
          ),
          DynDataflowBatches::join(i1_recent, i2_recent, op.clone()),
        )
      }

      Self::Join { d1, d2, ctx } => {
        let op = BatchBinaryOp::Join { ctx: ctx.clone() };
        let i1_stable = d1.iter_stable();
        let i2_stable = d2.iter_stable();
        let i1_recent = d1.iter_recent();
        let i2_recent = d2.iter_recent();
        DynDataflowBatches::chain(
          DynDataflowBatches::chain(
            DynDataflowBatches::join(i1_recent.clone(), i2_stable, op.clone()),
            DynDataflowBatches::join(i1_stable, i2_recent.clone(), op.clone()),
          ),
          DynDataflowBatches::join(i1_recent, i2_recent, op.clone()),
        )
      }
    }
  }
}

pub enum DynDataflowBatches<'a, Tag: Semiring> {
  /// Empty (no batch)
  Empty,

  /// Singleton batch
  Single(Option<DynDataflowBatch<'a, Tag>>),

  /// Optional batches
  Optional(Option<Box<DynDataflowBatches<'a, Tag>>>),

  /// Chain
  Chain {
    b1: Box<DynDataflowBatches<'a, Tag>>,
    b2: Box<DynDataflowBatches<'a, Tag>>,
    use_b1: bool,
  },

  /// Batch Map
  Map {
    source: Box<DynDataflowBatches<'a, Tag>>,
    op: BatchUnaryOp<'a, Tag>,
  },

  /// Batch Join
  Join {
    b1: Box<DynDataflowBatches<'a, Tag>>,
    b1_curr: Option<DynDataflowBatch<'a, Tag>>,
    b2: Box<DynDataflowBatches<'a, Tag>>,
    b2_source: Box<DynDataflowBatches<'a, Tag>>,
    op: BatchBinaryOp<'a, Tag>,
  },

  /// Variable stable batches
  VariableStable {
    relations: Ref<'a, Vec<DynRelation<Tag>>>,
    rela_id: usize,
  },

  /// Static variable stable batches
  StaticVariableStable {
    variable: StaticVariable<'a, Tag>,
    rela_id: usize,
  },
}

impl<'a, Tag: Semiring> DynDataflowBatches<'a, Tag> {
  pub fn variable_stable(v: &'a DynVariable<Tag>) -> Self {
    Self::VariableStable {
      relations: v.stable.borrow(),
      rela_id: 0,
    }
  }

  pub fn single(batch: DynDataflowBatch<'a, Tag>) -> Self {
    Self::Single(Some(batch))
  }

  pub fn some(batches: DynDataflowBatches<'a, Tag>) -> Self {
    Self::Optional(Some(Box::new(batches)))
  }

  pub fn none() -> Self {
    Self::Optional(None)
  }

  pub fn chain(b1: Self, b2: Self) -> Self {
    Self::Chain {
      b1: Box::new(b1),
      b2: Box::new(b2),
      use_b1: true,
    }
  }

  pub fn map(batches: DynDataflowBatches<'a, Tag>, op: BatchUnaryOp<'a, Tag>) -> Self {
    Self::Map {
      source: Box::new(batches),
      op: op,
    }
  }

  pub fn join(mut b1: Self, b2: Self, op: BatchBinaryOp<'a, Tag>) -> Self {
    let b1_curr = b1.next();
    let b2_source = b2.clone();
    Self::Join {
      b1: Box::new(b1),
      b1_curr: b1_curr,
      b2: Box::new(b2),
      b2_source: Box::new(b2_source),
      op: op,
    }
  }
}

impl<'a, Tag: Semiring> Clone for DynDataflowBatches<'a, Tag> {
  fn clone(&self) -> Self {
    match self {
      Self::Empty => Self::Empty,
      Self::Single(s) => Self::Single(s.clone()),
      Self::Optional(o) => Self::Optional(o.clone()),
      Self::Chain { b1, b2, use_b1 } => Self::Chain {
        b1: b1.clone(),
        b2: b2.clone(),
        use_b1: use_b1.clone(),
      },
      Self::Map { source, op } => Self::Map {
        source: source.clone(),
        op: op.clone(),
      },
      Self::Join {
        b1,
        b1_curr,
        b2,
        b2_source,
        op,
      } => Self::Join {
        b1: b1.clone(),
        b1_curr: b1_curr.clone(),
        b2: b2.clone(),
        b2_source: b2_source.clone(),
        op: op.clone(),
      },
      Self::VariableStable { relations, rela_id } => Self::VariableStable {
        relations: Ref::clone(relations),
        rela_id: rela_id.clone(),
      },
      Self::StaticVariableStable { variable, rela_id } => Self::StaticVariableStable {
        variable: variable.clone(),
        rela_id: rela_id.clone(),
      },
    }
  }
}

impl<'a, Tag: Semiring> Iterator for DynDataflowBatches<'a, Tag> {
  type Item = DynDataflowBatch<'a, Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Empty => None,
      Self::Single(s) => s.take(),
      Self::Optional(o) => match o {
        Some(batches) => batches.next(),
        None => None,
      },
      Self::Chain { b1, b2, use_b1 } => {
        if *use_b1 {
          if let Some(b1_batch) = b1.next() {
            return Some(b1_batch);
          } else {
            *use_b1 = false;
          }
        }
        b2.next()
      }
      Self::Map { source, op } => source.next().map(|batch| op.apply(batch)),
      Self::Join {
        b1,
        b1_curr,
        b2,
        b2_source,
        op,
      } => loop {
        match b1_curr {
          Some(b1_curr_batch) => match b2.next() {
            Some(b2_curr_batch) => {
              let result = op.apply(b1_curr_batch.clone(), b2_curr_batch);
              return Some(result);
            }
            None => {
              *b1_curr = b1.next();
              *b2 = b2_source.clone();
            }
          },
          None => return None,
        }
      },
      Self::VariableStable { relations, rela_id } => {
        if *rela_id < relations.len() {
          let result = DynDataflowBatch::VariableStable {
            relations: Ref::clone(relations),
            rela_id: *rela_id,
            elem_id: 0,
          };
          *rela_id += 1;
          Some(result)
        } else {
          None
        }
      }
      Self::StaticVariableStable { variable, rela_id } => {
        if *rela_id < variable.num_stable_relations() {
          let result = DynDataflowBatch::StaticVariableStable {
            variable: variable.clone(),
            rela_id: rela_id.clone(),
            elem_id: 0,
          };
          *rela_id += 1;
          Some(result)
        } else {
          None
        }
      }
    }
  }
}

#[derive(Debug)]
pub enum BatchUnaryOp<'a, Tag: Semiring> {
  Projection(Expression),
  Filter(Expression),
  Find(DynTuple),
  MergeTag { tag: Tag, ctx: &'a Tag::Context },
}

impl<'a, Tag: Semiring> Clone for BatchUnaryOp<'a, Tag> {
  fn clone(&self) -> Self {
    match self {
      Self::Projection(e) => Self::Projection(e.clone()),
      Self::Filter(e) => Self::Filter(e.clone()),
      Self::Find(t) => Self::Find(t.clone()),
      Self::MergeTag { tag, ctx } => Self::MergeTag { tag: tag.clone(), ctx },
    }
  }
}

impl<'a, Tag: Semiring> BatchUnaryOp<'a, Tag> {
  pub fn apply(&self, mut source: DynDataflowBatch<'a, Tag>) -> DynDataflowBatch<'a, Tag> {
    match self {
      Self::Projection(expr) => DynDataflowBatch::Projection {
        source: Box::new(source),
        expression: expr.clone(),
      },
      Self::Filter(expr) => DynDataflowBatch::Filter {
        source: Box::new(source),
        expression: expr.clone(),
      },
      Self::Find(key) => {
        let curr_elem = source.next();
        DynDataflowBatch::Find {
          source: Box::new(source),
          curr_elem: curr_elem,
          key: key.clone(),
        }
      }
      Self::MergeTag { tag, ctx } => DynDataflowBatch::MergeTag {
        source: Box::new(source),
        tag: tag.clone(),
        ctx: ctx,
      },
    }
  }
}

#[derive(Debug)]
pub enum BatchBinaryOp<'a, Tag: Semiring> {
  Product { ctx: &'a Tag::Context },
  Intersection { ctx: &'a Tag::Context },
  Join { ctx: &'a Tag::Context },
}

impl<'a, Tag: Semiring> Clone for BatchBinaryOp<'a, Tag> {
  fn clone(&self) -> Self {
    match self {
      Self::Product { ctx } => Self::Product { ctx },
      Self::Intersection { ctx } => Self::Intersection { ctx },
      Self::Join { ctx } => Self::Join { ctx },
    }
  }
}

impl<'a, Tag: Semiring> BatchBinaryOp<'a, Tag> {
  pub fn apply(
    &self,
    mut i1: DynDataflowBatch<'a, Tag>,
    mut i2: DynDataflowBatch<'a, Tag>,
  ) -> DynDataflowBatch<'a, Tag> {
    match self {
      Self::Product { ctx } => {
        let i1_curr = i1.next();
        DynDataflowBatch::Product {
          i1: Box::new(i1),
          i1_curr: i1_curr,
          i2_source: Box::new(i2.clone()),
          i2_clone: Box::new(i2),
          ctx: ctx,
        }
      }
      Self::Intersection { ctx } => {
        let i1_curr = i1.next();
        let i2_curr = i2.next();
        DynDataflowBatch::Intersection {
          i1: Box::new(i1),
          i1_curr: i1_curr,
          i2: Box::new(i2),
          i2_curr: i2_curr,
          ctx: ctx,
        }
      }
      Self::Join { ctx } => {
        let i1_curr = i1.next();
        let i2_curr = i2.next();
        DynDataflowBatch::Join {
          i1: Box::new(i1),
          i1_curr: i1_curr,
          i2: Box::new(i2),
          i2_curr: i2_curr,
          curr_iter: None,
          ctx: ctx,
        }
      }
    }
  }
}

pub enum DynDataflowBatch<'a, Tag: Semiring> {
  /// Simple vector of element
  Vec(std::slice::Iter<'a, DynElement<Tag>>),

  /// Variable stable iterator
  VariableStable {
    relations: Ref<'a, Vec<DynRelation<Tag>>>,
    rela_id: usize,
    elem_id: usize,
  },

  /// Variable recent iterator
  VariableRecent {
    relation: Ref<'a, DynRelation<Tag>>,
    elem_id: usize,
  },

  /// Static variable stable iterator
  StaticVariableStable {
    variable: StaticVariable<'a, Tag>,
    rela_id: usize,
    elem_id: usize,
  },

  /// Static variable recent iterator
  StaticVariableRecent {
    variable: StaticVariable<'a, Tag>,
    elem_id: usize,
  },

  /// Projection
  Projection {
    source: Box<DynDataflowBatch<'a, Tag>>,
    expression: Expression,
  },

  /// Filter
  Filter {
    source: Box<DynDataflowBatch<'a, Tag>>,
    expression: Expression,
  },

  /// Find
  Find {
    source: Box<DynDataflowBatch<'a, Tag>>,
    curr_elem: Option<DynElement<Tag>>,
    key: DynTuple,
  },

  MergeTag {
    source: Box<DynDataflowBatch<'a, Tag>>,
    tag: Tag,
    ctx: &'a Tag::Context,
  },

  Product {
    i1: Box<DynDataflowBatch<'a, Tag>>,
    i1_curr: Option<DynElement<Tag>>,
    i2_source: Box<DynDataflowBatch<'a, Tag>>,
    i2_clone: Box<DynDataflowBatch<'a, Tag>>,
    ctx: &'a Tag::Context,
  },

  Intersection {
    i1: Box<DynDataflowBatch<'a, Tag>>,
    i1_curr: Option<DynElement<Tag>>,
    i2: Box<DynDataflowBatch<'a, Tag>>,
    i2_curr: Option<DynElement<Tag>>,
    ctx: &'a Tag::Context,
  },

  Join {
    i1: Box<DynDataflowBatch<'a, Tag>>,
    i1_curr: Option<DynElement<Tag>>,
    i2: Box<DynDataflowBatch<'a, Tag>>,
    i2_curr: Option<DynElement<Tag>>,
    curr_iter: Option<JoinProductIterator<Tag>>,
    ctx: &'a Tag::Context,
  },
}

impl<'a, Tag: Semiring> Clone for DynDataflowBatch<'a, Tag> {
  fn clone(&self) -> Self {
    match self {
      Self::Vec(v) => Self::Vec(v.clone()),
      Self::VariableStable {
        relations,
        rela_id,
        elem_id,
      } => Self::VariableStable {
        relations: Ref::clone(relations),
        rela_id: rela_id.clone(),
        elem_id: elem_id.clone(),
      },
      Self::VariableRecent { relation, elem_id } => Self::VariableRecent {
        relation: Ref::clone(relation),
        elem_id: elem_id.clone(),
      },
      Self::StaticVariableStable {
        variable,
        rela_id,
        elem_id,
      } => Self::StaticVariableStable {
        variable: variable.clone(),
        rela_id: rela_id.clone(),
        elem_id: elem_id.clone(),
      },
      Self::StaticVariableRecent { variable, elem_id } => Self::StaticVariableRecent {
        variable: variable.clone(),
        elem_id: elem_id.clone(),
      },
      Self::Projection { source, expression } => Self::Projection {
        source: source.clone(),
        expression: expression.clone(),
      },
      Self::Filter { source, expression } => Self::Filter {
        source: source.clone(),
        expression: expression.clone(),
      },
      Self::Find {
        source,
        curr_elem,
        key,
      } => Self::Find {
        source: source.clone(),
        curr_elem: curr_elem.clone(),
        key: key.clone(),
      },
      Self::MergeTag { source, tag, ctx } => Self::MergeTag {
        source: source.clone(),
        tag: tag.clone(),
        ctx: ctx,
      },
      Self::Product {
        i1,
        i1_curr,
        i2_source,
        i2_clone,
        ctx,
      } => Self::Product {
        i1: i1.clone(),
        i1_curr: i1_curr.clone(),
        i2_source: i2_source.clone(),
        i2_clone: i2_clone.clone(),
        ctx: ctx,
      },
      Self::Intersection {
        i1,
        i1_curr,
        i2,
        i2_curr,
        ctx,
      } => Self::Intersection {
        i1: i1.clone(),
        i1_curr: i1_curr.clone(),
        i2: i2.clone(),
        i2_curr: i2_curr.clone(),
        ctx: ctx,
      },
      Self::Join {
        i1,
        i1_curr,
        i2,
        i2_curr,
        curr_iter,
        ctx,
      } => Self::Join {
        i1: i1.clone(),
        i1_curr: i1_curr.clone(),
        i2: i2.clone(),
        i2_curr: i2_curr.clone(),
        curr_iter: curr_iter.clone(),
        ctx: ctx,
      },
    }
  }
}

impl<'a, Tag: Semiring> DynDataflowBatch<'a, Tag> {
  pub fn step(&mut self, u: usize) {
    match self {
      Self::VariableStable { elem_id, .. } => {
        *elem_id += u;
      }
      Self::VariableRecent { elem_id, .. } => {
        *elem_id += u;
      }
      _ => {
        for _ in 0..u {
          self.next();
        }
      }
    }
  }

  pub fn search_ahead<F>(&mut self, mut cmp: F) -> Option<DynElement<Tag>>
  where
    F: FnMut(&DynTuple) -> bool,
  {
    fn search_ahead_variable_helper_1<Tag, F>(
      relation: &DynRelation<Tag>,
      elem_id: &mut usize,
      mut cmp: F,
    ) -> bool
    where
      Tag: Semiring,
      F: FnMut(&DynTuple) -> bool,
    {
      assert!(*elem_id > 0);
      let mut curr = *elem_id - 1;
      if curr < relation.len() && cmp(&relation[curr].tup) {
        let mut step = 1;
        while curr + step < relation.len() && cmp(&relation[curr + step].tup) {
          curr += step;
          step <<= 1;
        }
        step >>= 1;
        while step > 0 {
          if curr + step < relation.len() && cmp(&relation[curr + step].tup) {
            curr += step;
          }
          step >>= 1;
        }
        *elem_id = curr + 1;
        true
      } else {
        false
      }
    }

    match self {
      Self::VariableStable {
        relations,
        rela_id,
        elem_id,
      } => {
        let relation = &relations[*rela_id];
        if search_ahead_variable_helper_1(relation, elem_id, cmp) {
          self.next()
        } else {
          None
        }
      }
      Self::VariableRecent { relation, elem_id } => {
        if search_ahead_variable_helper_1(relation, elem_id, cmp) {
          self.next()
        } else {
          None
        }
      }
      Self::StaticVariableStable {
        variable,
        rela_id,
        elem_id,
      } => {
        assert!(*elem_id > 0);
        let mut curr = *elem_id - 1;
        let len = variable.num_stable_elements(*rela_id);
        if curr < len && cmp(&variable.stable_element(*rela_id, curr).tup) {
          let mut step = 1;
          while curr + step < len && cmp(&variable.stable_element(*rela_id, curr + step).tup) {
            curr += step;
            step <<= 1;
          }
          step >>= 1;
          while step > 0 {
            if curr + step < len && cmp(&variable.stable_element(*rela_id, curr + step).tup) {
              curr += step;
            }
            step >>= 1;
          }
          *elem_id = curr + 1;
          self.next()
        } else {
          None
        }
      }
      Self::StaticVariableRecent { variable, elem_id } => {
        assert!(*elem_id > 0);
        let mut curr = *elem_id - 1;
        let len = variable.num_recent_elements();
        if curr < len && cmp(&variable.recent_element(curr).tup) {
          let mut step = 1;
          while curr + step < len && cmp(&variable.recent_element(curr + step).tup) {
            curr += step;
            step <<= 1;
          }
          step >>= 1;
          while step > 0 {
            if curr + step < len && cmp(&variable.recent_element(curr + step).tup) {
              curr += step;
            }
            step >>= 1;
          }
          *elem_id = curr + 1;
          self.next()
        } else {
          None
        }
      }
      _ => self.next(),
    }
  }

  pub fn vec(v: &'a Vec<DynElement<Tag>>) -> Self {
    Self::Vec(v.iter())
  }

  pub fn variable_stable(v: &'a DynVariable<Tag>) -> Self {
    Self::VariableStable {
      relations: v.stable.borrow(),
      rela_id: 0,
      elem_id: 0,
    }
  }

  pub fn variable_recent(v: &'a DynVariable<Tag>) -> Self {
    Self::VariableRecent {
      relation: v.recent.borrow(),
      elem_id: 0,
    }
  }
}

impl<'a, Tag: Semiring> Iterator for DynDataflowBatch<'a, Tag> {
  type Item = DynElement<Tag>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Vec(iter) => iter.next().map(Clone::clone),
      Self::VariableStable {
        relations,
        rela_id,
        elem_id,
      } => {
        let relation = &relations[*rela_id];
        if *elem_id < relation.len() {
          let elem = &relation[*elem_id];
          *elem_id += 1;
          Some(elem.clone())
        } else {
          None
        }
      }
      Self::VariableRecent { relation, elem_id } => {
        if *elem_id < relation.len() {
          let elem = &relation[*elem_id];
          *elem_id += 1;
          Some(elem.clone())
        } else {
          None
        }
      }
      Self::StaticVariableStable {
        variable,
        rela_id,
        elem_id,
      } => {
        if *elem_id < variable.num_stable_elements(*rela_id) {
          let elem = variable.stable_element(*rela_id, *elem_id);
          *elem_id += 1;
          Some(elem)
        } else {
          None
        }
      }
      Self::StaticVariableRecent { variable, elem_id } => {
        if *elem_id < variable.num_recent_elements() {
          let elem = variable.recent_element(*elem_id);
          *elem_id += 1;
          Some(elem)
        } else {
          None
        }
      }
      Self::Projection { source, expression } => source.next().map(|elem| DynElement {
        tup: expression.eval(&elem.tup),
        tag: elem.tag,
      }),
      Self::Filter { source, expression } => {
        while let Some(elem) = source.next() {
          if expression.eval(&elem.tup).is_true() {
            return Some(elem);
          }
        }
        None
      }
      Self::Find {
        source,
        curr_elem,
        key,
      } => {
        use std::cmp::Ordering;
        let key = key.clone();
        loop {
          match curr_elem {
            Some(elem) => {
              let fst = elem.tup[0].cmp(&key);
              match fst {
                Ordering::Less => *curr_elem = source.search_ahead(|x| x[0] < key),
                Ordering::Equal => {
                  let result = elem.clone();
                  *curr_elem = source.next();
                  return Some(result);
                }
                Ordering::Greater => return None,
              }
            }
            None => return None,
          }
        }
      }
      Self::MergeTag { source, tag, ctx } => match source.next() {
        Some(elem) => {
          let merged = DynElement {
            tup: elem.tup,
            tag: Tag::mult(ctx, tag, &elem.tag),
          };
          Some(merged)
        }
        None => None,
      },
      Self::Product {
        i1,
        i1_curr,
        i2_source,
        i2_clone,
        ctx,
      } => loop {
        match i1_curr {
          Some(i1_elem) => match i2_clone.next() {
            Some(i2_elem) => {
              let tup = DynTuple::Tuple(vec![i1_elem.tup.clone(), i2_elem.tup.clone()]);
              let tag = Tag::mult(ctx, &i1_elem.tag, &i2_elem.tag);
              let elem = DynElement { tup, tag };
              return Some(elem);
            }
            None => {
              *i1_curr = i1.next();
              *i2_clone = i2_source.clone();
            }
          },
          None => return None,
        }
      },
      Self::Intersection {
        i1,
        i1_curr,
        i2,
        i2_curr,
        ctx,
      } => {
        use std::cmp::Ordering;
        loop {
          match (&i1_curr, &i2_curr) {
            (Some(i1_curr_elem), Some(i2_curr_elem)) => {
              match i1_curr_elem.tup.cmp(&i2_curr_elem.tup) {
                Ordering::Less => {
                  *i1_curr = i1.search_ahead(|i1_next| i1_next < &i2_curr_elem.tup);
                }
                Ordering::Equal => {
                  let tag = Tag::mult(ctx, &i1_curr_elem.tag, &i2_curr_elem.tag);
                  let result = DynElement {
                    tup: i1_curr_elem.tup.clone(),
                    tag,
                  };
                  *i1_curr = i1.next();
                  *i2_curr = i2.next();
                  return Some(result);
                }
                Ordering::Greater => {
                  *i2_curr = i2.search_ahead(|i2_next| i2_next < &i1_curr_elem.tup);
                }
              }
            }
            _ => return None,
          }
        }
      }
      Self::Join {
        i1,
        i1_curr,
        i2,
        i2_curr,
        curr_iter,
        ctx,
      } => {
        use std::cmp::Ordering;
        loop {
          if let Some(curr_prod_iter) = curr_iter {
            if let Some((e1, e2)) = curr_prod_iter.next() {
              let tup = DynTuple::Tuple(vec![
                e1.tup[0].clone(),
                e1.tup[1].clone(),
                e2.tup[1].clone(),
              ]);
              let tag = Tag::mult(ctx, &e1.tag, &e2.tag);
              let result = DynElement { tup, tag };
              return Some(result);
            } else {
              i1.step(curr_prod_iter.v1.len() - 1);
              *i1_curr = i1.next();
              i2.step(curr_prod_iter.v2.len() - 1);
              *i2_curr = i2.next();
              *curr_iter = None;
            }
          }

          match (&i1_curr, &i2_curr) {
            (Some(i1_curr_elem), Some(i2_curr_elem)) => {
              match i1_curr_elem.tup[0].cmp(&i2_curr_elem.tup[0]) {
                Ordering::Less => {
                  *i1_curr = i1.search_ahead(|i1_next| i1_next[0] < i2_curr_elem.tup[0])
                }
                Ordering::Equal => {
                  let key = &i1_curr_elem.tup[0];
                  let v1 = std::iter::once(i1_curr_elem.clone())
                    .chain(i1.clone().take_while(|x| &x.tup[0] == key))
                    .collect::<Vec<_>>();
                  let v2 = std::iter::once(i2_curr_elem.clone())
                    .chain(i2.clone().take_while(|x| &x.tup[0] == key))
                    .collect::<Vec<_>>();
                  let iter = JoinProductIterator::new(v1, v2);
                  *curr_iter = Some(iter);
                }
                Ordering::Greater => {
                  *i2_curr = i2.search_ahead(|i2_next| i2_next[0] < i1_curr_elem.tup[0])
                }
              }
            }
            _ => break None,
          }
        }
      }
    }
  }
}

fn batch_key_search<'a, Tag: Semiring>(
  mut batch: DynDataflowBatch<'a, Tag>,
  key: &DynTuple,
) -> Option<Tag> {
  if let Some(curr) = batch.next() {
    if &curr.tup == key {
      return Some(curr.tag.clone());
    }
  } else {
    return None;
  }
  while let Some(curr) = batch.search_ahead(|i| i < key) {
    if &curr.tup == key {
      return Some(curr.tag.clone());
    }
  }
  return None;
}

#[derive(Clone)]
pub struct JoinProductIterator<Tag: Semiring> {
  v1: Vec<DynElement<Tag>>,
  v2: Vec<DynElement<Tag>>,
  i1: usize,
  i2: usize,
}

impl<Tag: Semiring> JoinProductIterator<Tag> {
  pub fn new(v1: Vec<DynElement<Tag>>, v2: Vec<DynElement<Tag>>) -> Self {
    Self {
      v1,
      v2,
      i1: 0,
      i2: 0,
    }
  }
}

impl<Tag: Semiring> Iterator for JoinProductIterator<Tag> {
  type Item = (DynElement<Tag>, DynElement<Tag>);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.i1 < self.v1.len() {
        if self.i2 < self.v2.len() {
          let e1 = &self.v1[self.i1];
          let e2 = &self.v2[self.i2];
          let result = (e1.clone(), e2.clone());
          self.i2 += 1;
          return Some(result);
        } else {
          self.i1 += 1;
          self.i2 = 0;
        }
      } else {
        return None;
      }
    }
  }
}
