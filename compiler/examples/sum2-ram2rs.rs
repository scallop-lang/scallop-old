use scallop_compiler::{ast_analysis::*, common::*, ram::*, ram2rs::*, options::*};

fn main() {
  let sum2_ram = Program {
    variables: vec![
      Variable {
        is_temporary: false,
        name: "digit".to_string(),
        arg_types: VarType::Tuple(vec![
          VarType::Base(Type::Symbol),
          VarType::Base(Type::Integer),
        ]),
      },
      Variable {
        is_temporary: false,
        name: "sum".to_string(),
        arg_types: VarType::Tuple(vec![
          VarType::Tuple(vec![
            VarType::Base(Type::Symbol),
            VarType::Base(Type::Symbol),
          ]),
          VarType::Base(Type::Integer),
        ]),
      },
    ],
    updates: vec![Update {
      into_var: "sum".to_string(),
      flow: Flow::Project(
        Box::new(Flow::Product(
          Box::new(Flow::Variable("digit".to_string())),
          Box::new(Flow::Variable("digit".to_string())),
        )),
        Argument::Tuple(vec![
          Argument::Tuple(vec![
            Argument::Element(vec![0, 0]),
            Argument::Element(vec![1, 0]),
          ]),
          Argument::Binary(
            BinaryOp::Add,
            Box::new(Argument::Element(vec![0, 1])),
            Box::new(Argument::Element(vec![1, 1])),
          ),
        ]),
      ),
    }],
    facts: vec![],
    disjunctions: vec![],
  };

  let options = CompileOptions::default();
  println!("{}", ram2rs("Sum2", &sum2_ram, &AnalysisResult::default(), &options));
}
