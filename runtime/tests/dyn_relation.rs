use scallop_runtime::interpreter::*;

#[test]
fn test_relation_1() {
  let data: Vec<(DynTuple, ())> = vec![
    ((1 as usize, 3 as usize).into(), ()),
    ((1 as usize, 4 as usize).into(), ()),
    ((2 as usize, 4 as usize).into(), ()),
    ((4 as usize, 5 as usize).into(), ()),
  ];
  let elements = data
    .into_iter()
    .map(|(tup, tag)| DynElement { tup, tag })
    .collect();
  let relation = DynRelation::from_vec_unchecked(elements);
  for elem in relation.iter() {
    println!("{:?}", elem.tup);
  }
}

#[test]
fn test_relation_2() {
  let data: Vec<(DynTuple, ())> = vec![
    ((1 as usize, true).into(), ()),
    ((1 as usize, false).into(), ()),
    ((2 as usize, true).into(), ()),
    ((4 as usize, true).into(), ()),
  ];
  let elements = data
    .into_iter()
    .map(|(tup, tag)| DynElement { tup, tag })
    .collect();
  let relation = DynRelation::from_vec_unchecked(elements);
  for elem in relation.iter() {
    println!("{:?}", elem.tup);
  }
}
