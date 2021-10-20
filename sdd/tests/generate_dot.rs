use sdd::*;

/// This shows the Figure 1 of the paper
/// - SDD: A New Canonical Representation of Propositional Knowledge Bases
#[test]
fn sdd_generate_dot_1() {
  // (A ^ B) v (B ^ C) v (C ^ D)
  let form = (bf(0) & bf(1)) | (bf(1) & bf(2)) | (bf(2) & bf(3));
  let vars = vec![1, 0, 3, 2];
  let config = SDDBuilderConfig::new(vars, VTreeType::Balanced, true);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("sdd.dot").unwrap();
}
