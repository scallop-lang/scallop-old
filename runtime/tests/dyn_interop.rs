use scallop_runtime::interpreter::*;
use scallop_runtime::*;

#[test]
fn test_dataflow_interop_1() {
  let mut var_1 = Variable::<(usize, usize), ()>::new();
  let mut var_2 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;

  // First insert data to var_1
  let data = vec![((), (1, 3)), ((), (1, 4)), ((), (2, 5)), ((), (4, 4))];
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) || var_2.changed(&ctx) {
    // Then insert var_1 into var_2
    var_2.insert(
      &ctx,
      &DynDataflow::Projection {
        source: Box::new(DynDataflow::StaticVariable(StaticVariable(&var_1))),
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
