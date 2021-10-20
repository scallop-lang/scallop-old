use scallop_runtime::interpreter::*;
use scallop_runtime::tags::*;
use scallop_runtime::*;

#[test]
fn test_dataflow_variable_1() {
  let mut var_1 = DynVariable::<()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as i64, 3 as i64).into()),
    ((), (1 as i64, 4 as i64).into()),
    ((), (2 as i64, 4 as i64).into()),
    ((), (4 as i64, 5 as i64).into()),
  ];
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) {
    // Then insert var_1 into var_2
    var_2.insert(&ctx, &DynDataflow::Variable(&var_1));
  }

  // Inspection
  println!("Var 1");
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 2");
  for elem in var_2.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}

#[test]
fn test_dataflow_projection_1() {
  let mut var_1 = DynVariable::<()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as i64, 3 as i64).into()),
    ((), (1 as i64, 4 as i64).into()),
    ((), (2 as i64, 5 as i64).into()),
    ((), (4 as i64, 4 as i64).into()),
  ];
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) {
    // Then insert var_1 into var_2
    var_2.insert(
      &ctx,
      &DynDataflow::Projection {
        source: Box::new(DynDataflow::Variable(&var_1)),
        expression: Expression::Access(TupleAccessor::top(1)),
      },
    );
  }

  // Inspection
  println!("Var 1");
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 2");
  for elem in var_2.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}

#[test]
fn test_dataflow_projection_2() {
  let mut var_1 = DynVariable::<()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as i64, 3 as i64).into()),
    ((), (1 as i64, 4 as i64).into()),
    ((), (2 as i64, 5 as i64).into()),
    ((), (4 as i64, 4 as i64).into()),
  ];
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) {
    // Then insert var_1 into var_2
    var_2.insert(
      &ctx,
      &DynDataflow::Projection {
        source: Box::new(DynDataflow::Variable(&var_1)),
        expression: Expression::Binary(Binary {
          op: BinaryOp::Add,
          lhs: Box::new(Expression::Access(TupleAccessor::top(0))),
          rhs: Box::new(Expression::Access(TupleAccessor::top(1))),
        }),
      },
    );
  }

  // Inspection
  println!("Var 1");
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 2");
  for elem in var_2.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}

#[test]
fn test_dataflow_filter_1() {
  let mut var_1 = DynVariable::<()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data: Vec<((), DynTuple)> = vec![
    ((), (2 as i64, 5 as i64).into()),
    ((), (4 as i64, 4 as i64).into()),
    ((), (6 as i64, 3 as i64).into()),
    ((), (1 as i64, 4 as i64).into()),
    ((), (5 as i64, 1 as i64).into()),
    ((), (1 as i64, 3 as i64).into()),
  ];
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) {
    // Then insert var_1 into var_2
    var_2.insert(
      &ctx,
      &DynDataflow::Filter {
        source: Box::new(DynDataflow::Variable(&var_1)),
        expression: Expression::Binary(Binary {
          op: BinaryOp::Lt,
          lhs: Box::new(Expression::Access(TupleAccessor::top(0))),
          rhs: Box::new(Expression::Access(TupleAccessor::top(1))),
        }),
      },
    );
  }

  // Inspection
  println!("Var 1");
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 2");
  for elem in var_2.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}

#[test]
fn test_dataflow_product_1() {
  let mut var_1 = DynVariable::<()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut var_3 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data: Vec<((), DynTuple)> = vec![
    ((), (0 as i64, 1 as i64).into()),
    ((), (0 as i64, 2 as i64).into()),
    ((), (0 as i64, 3 as i64).into()),
  ];
  var_1.insert_with_context(&mut ctx, data);

  // First insert data to var_2
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as i64, 1 as i64).into()),
    ((), (1 as i64, 2 as i64).into()),
    ((), (1 as i64, 3 as i64).into()),
  ];
  var_2.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) || var_3.changed(&ctx) {
    // Then insert var_1 into var_2
    var_3.insert(
      &ctx,
      &DynDataflow::Projection {
        source: Box::new(DynDataflow::Product {
          i1: Box::new(DynDataflow::Variable(&var_1)),
          i2: Box::new(DynDataflow::Variable(&var_2)),
          ctx: &ctx,
        }),
        expression: Expression::Tuple(vec![
          Expression::Access(TupleAccessor::from_indices(&[0, 0])),
          Expression::Access(TupleAccessor::from_indices(&[1, 0])),
          Expression::Binary(Binary {
            op: BinaryOp::Add,
            lhs: Box::new(Expression::Access(TupleAccessor::from_indices(&[0, 1]))),
            rhs: Box::new(Expression::Access(TupleAccessor::from_indices(&[1, 1]))),
          }),
        ]),
      },
    );
  }

  // Inspection
  println!("Var 1");
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 2");
  for elem in var_2.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
  println!("Var 3");
  for elem in var_3.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}
