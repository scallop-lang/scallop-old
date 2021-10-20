use scallop_interpreter::*;

#[test]
fn test_relation_1() {
  let elements: Vec<(Component, ())> = vec![
    ((1 as usize, 3 as usize).into(), ()),
    ((1 as usize, 4 as usize).into(), ()),
    ((2 as usize, 4 as usize).into(), ()),
    ((4 as usize, 5 as usize).into(), ()),
  ];
  let relation = Relation::from_vec_unchecked(elements);
  for elem in relation.iter() {
    println!("{:?}", elem.component());
  }
}

#[test]
fn test_relation_2() {
  let elements: Vec<(Component, ())> = vec![
    ((1 as usize, true).into(), ()),
    ((1 as usize, false).into(), ()),
    ((2 as usize, true).into(), ()),
    ((4 as usize, true).into(), ()),
  ];
  let relation = Relation::from_vec_unchecked(elements);
  for elem in relation.iter() {
    println!("{:?}", elem.component());
  }
}
