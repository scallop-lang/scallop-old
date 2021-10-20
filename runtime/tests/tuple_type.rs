use scallop_runtime::*;

#[test]
fn tuple_type_1() {
  let got = <TupleType as FromType<(usize, usize)>>::from_type();
  let exp = TupleType::Tuple(vec![TupleType::Symbol, TupleType::Symbol]);
  println!("{:?}", got);
  assert_eq!(got, exp);
}
