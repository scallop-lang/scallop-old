use scallop_compiler::{ast::*, ast2ram::*, ast_analysis::*, common::BinaryOp, ram2rs::*, options::*};

fn main() {
  let sum2_ast = Program {
    decls: vec![
      Decl::new((
        "digit".to_string(),
        vec![Type::new(TypeNode::Symbol), Type::new(TypeNode::Integer)],
      )),
      Decl::new((
        "sum".to_string(),
        vec![
          Type::new(TypeNode::Symbol),
          Type::new(TypeNode::Symbol),
          Type::new(TypeNode::Integer),
        ],
      )),
    ],
    facts: vec![],
    disjunctions: vec![],
    rules: vec![Rule::new((
      Atom::new((
        "sum".to_string(),
        vec![
          Argument::Variable(Variable::new("A".to_string())),
          Argument::Variable(Variable::new("B".to_string())),
          Argument::Binary(BinaryExpr::new((
            BinaryOp::Add,
            Box::new(Argument::Variable(Variable::new("DA".to_string()))),
            Box::new(Argument::Variable(Variable::new("DB".to_string()))),
          ))),
        ],
      )),
      vec![
        Literal::new(LiteralNode::Pos(Atom::new((
          "digit".to_string(),
          vec![
            Argument::Variable(Variable::new("A".to_string())),
            Argument::Variable(Variable::new("DA".to_string())),
          ],
        )))),
        Literal::new(LiteralNode::Pos(Atom::new((
          "digit".to_string(),
          vec![
            Argument::Variable(Variable::new("B".to_string())),
            Argument::Variable(Variable::new("DB".to_string())),
          ],
        )))),
      ],
    ))],
    queries: vec![],
  };
  let options = CompileOptions::default();
  let ram = ast2ram(&sum2_ast);
  println!("==== RAM ====");
  println!("{:?}", ram);
  let rs = ram2rs("Sum2", &ram.unwrap(), &AnalysisResult::default(), &options);
  println!("==== Rust ====");
  println!("{}", rs);
}
