use scallop_compiler::{ast::*, ast2ram::*, ast_analysis::*, error::*, ram2rs::*, options::*};

fn main() -> Result<(), CompileError> {
  let path_ast = Program {
    decls: vec![
      Decl::new((
        "edge".to_string(),
        vec![Type::new(TypeNode::Symbol), Type::new(TypeNode::Symbol)],
      )),
      Decl::new((
        "path".to_string(),
        vec![Type::new(TypeNode::Symbol), Type::new(TypeNode::Symbol)],
      )),
    ],
    facts: vec![],
    disjunctions: vec![],
    rules: vec![
      Rule::new((
        Atom::new((
          "path".to_string(),
          vec![
            Argument::Variable(Variable::new("A".to_string())),
            Argument::Variable(Variable::new("B".to_string())),
          ],
        )),
        vec![Literal::new(LiteralNode::Pos(Atom::new((
          "edge".to_string(),
          vec![
            Argument::Variable(Variable::new("A".to_string())),
            Argument::Variable(Variable::new("B".to_string())),
          ],
        ))))],
      )),
      Rule::new((
        Atom::new((
          "path".to_string(),
          vec![
            Argument::Variable(Variable::new("A".to_string())),
            Argument::Variable(Variable::new("C".to_string())),
          ],
        )),
        vec![
          Literal::new(LiteralNode::Pos(Atom::new((
            "edge".to_string(),
            vec![
              Argument::Variable(Variable::new("A".to_string())),
              Argument::Variable(Variable::new("B".to_string())),
            ],
          )))),
          Literal::new(LiteralNode::Pos(Atom::new((
            "path".to_string(),
            vec![
              Argument::Variable(Variable::new("B".to_string())),
              Argument::Variable(Variable::new("C".to_string())),
            ],
          )))),
        ],
      )),
    ],
    queries: vec![],
  };

  let options = CompileOptions::default();
  let ram = ast2ram(&path_ast)?;
  println!("==== RAM ====");
  println!("{:?}", ram);
  let rs = ram2rs("Path", &ram, &AnalysisResult::default(), &options);
  println!("==== Rust ====");
  println!("{}", rs);

  Ok(())
}
