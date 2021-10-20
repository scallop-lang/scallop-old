use itertools::Itertools;
use sdd::*;

#[test]
fn test_count_1() {
  let form = (bf(1) & !bf(2) & !bf(3)) | (!bf(1) & bf(2) & !bf(3)) | (!bf(1) & !bf(2) & bf(3));
  let config = SDDBuilderConfig::with_formula(&form);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("agg_count_1.dot").unwrap();
  println!("{:?}", sdd);
}

fn count_formula(num_vars: usize, count: usize) -> BooleanFormula {
  let mut outer_form = bf_false();
  let all_combs = (0..num_vars).combinations(count);
  for comb in all_combs {
    let mut inner_form = bf_true();
    for i in 0..num_vars {
      if comb.contains(&i) {
        inner_form = inner_form & bf(i)
      } else {
        inner_form = inner_form & !bf(i)
      }
    }
    outer_form = outer_form | inner_form
  }
  outer_form
}

#[test]
fn test_count_2() {
  let form = count_formula(7, 4);
  let vars = form.collect_vars();
  let vtree = VTree::new_with_type(vars, VTreeType::Balanced);
  let config = SDDBuilderConfig::with_vtree(vtree);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("agg_count_2.dot").unwrap();
}

fn topk_count_formula(num_vars: usize, count: usize, k: usize) -> BooleanFormula {
  let mut outer_form = bf_false();
  let all_combs = (0..num_vars).combinations(count);
  for comb in all_combs.take(k) {
    let mut inner_form = bf_true();
    for i in 0..num_vars {
      if comb.contains(&i) {
        inner_form = inner_form & bf(i)
      } else {
        inner_form = inner_form & !bf(i)
      }
    }
    outer_form = outer_form | inner_form
  }
  outer_form
}

#[test]
fn test_count_3() {
  let form = topk_count_formula(20, 10, 3);
  let vars = form.collect_vars();
  let vtree = VTree::new_with_type(vars, VTreeType::Balanced);
  let config = SDDBuilderConfig::with_vtree(vtree);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("agg_count_3.dot").unwrap();
}

fn topk_more_approx_count_formula(num_vars: usize, count: usize, k: usize) -> BooleanFormula {
  let mut outer_form = bf_false();
  let all_combs = (0..num_vars).combinations(count);
  for comb in all_combs.take(k) {
    let mut inner_form = bf_true();
    for i in 0..num_vars {
      if comb.contains(&i) {
        inner_form = inner_form & bf(i)
      } else {
        inner_form = inner_form & !bf(i)
      }
    }
    outer_form = outer_form | inner_form
  }
  outer_form
}

#[test]
fn test_count_3() {
  let form = topk_count_formula(20, 10, 3);
  let vars = form.collect_vars();
  let vtree = VTree::new_with_type(vars, VTreeType::Balanced);
  let config = SDDBuilderConfig::with_vtree(vtree);
  let sdd = SDDBuilder::with_config(config).build(&form);
  sdd.save_dot("agg_count_3.dot").unwrap();
}
