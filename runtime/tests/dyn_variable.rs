use scallop_runtime::interpreter::*;
use scallop_runtime::tags::*;

#[test]
fn test_variable_1() {
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as usize, 3 as usize).into()),
    ((), (1 as usize, 4 as usize).into()),
    ((), (2 as usize, 4 as usize).into()),
    ((), (4 as usize, 5 as usize).into()),
  ];
  let mut var_1 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) {
    // Do nothing
  }

  // Inspection
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}

#[test]
fn test_variable_2() {
  let data: Vec<((), DynTuple)> = vec![
    ((), (1 as usize, true).into()),
    ((), (1 as usize, false).into()),
    ((), (2 as usize, true).into()),
    ((), (4 as usize, true).into()),
  ];
  let mut var_1 = DynVariable::<()>::new();
  let mut ctx = UnitSemiringContext;
  var_1.insert_with_context(&mut ctx, data);

  // Iteration
  while var_1.changed(&ctx) {
    // Do nothing
  }

  // Inspection
  for elem in var_1.complete(&ctx).iter() {
    println!("{:?}", elem);
  }
}
