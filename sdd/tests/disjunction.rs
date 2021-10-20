use sdd::*;

#[test]
fn test_disjunction_1() {
  let form = ((bf(1) & bf(3)) | (bf(2) & bf(4))) & (!(bf(1) & bf(3)));
  let config = SDDBuilderConfig::with_formula(&form);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("disj_1.dot").unwrap();
  println!("{:?}", sdd);
}

#[test]
fn test_no_disjunction_1() {
  let form = (bf(1) & bf(3)) | (bf(2) & bf(4));
  let config = SDDBuilderConfig::with_formula(&form);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("disj_2.dot").unwrap();
  println!("{:?}", sdd);
}
