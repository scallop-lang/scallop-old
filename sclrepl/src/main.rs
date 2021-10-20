use std::str::FromStr;

use structopt::StructOpt;
use linefeed::{Interface, ReadResult};

use scallop_runtime::{Program, EmptyProgram, Semiring, TupleType};
use scallop_compiler::{parser, ast::TypeNode};

#[derive(Debug)]
enum SemiringType {
  Empty,
  Boolean,
  Proofs,
  TopKProofs,
}

impl FromStr for SemiringType {
  type Err = &'static str;

  fn from_str(emit: &str) -> Result<Self, Self::Err> {
    match emit {
      "empty" => Ok(Self::Empty),
      "boolean" => Ok(Self::Boolean),
      "proofs" => Ok(Self::Proofs),
      "top-k-proofs" => Ok(Self::TopKProofs),
      _ => Err("Unknown semiring type"),
    }
  }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "sclrepl")]
struct Options {
  #[structopt(long, default_value = "empty")]
  pub semiring: SemiringType,
}

fn main() -> std::io::Result<()> {
  let options = Options::from_args();
  match options.semiring {
    SemiringType::Empty => run::<()>(),
    SemiringType::Boolean => run::<bool>(),
    SemiringType::Proofs => panic!("Not implemented"),
    SemiringType::TopKProofs => panic!("Not implemented"),
  }
}

fn run<Tag: Semiring>() -> std::io::Result<()> {
  let mut prog = EmptyProgram::<Tag>::new();
  let reader = Interface::new("sclrepl")?;
  reader.set_prompt("scallop> ")?;
  while let ReadResult::Input(input) = reader.read_line()? {
    let maybe_item = parser::parse_item(&input);
    match maybe_item {
      Ok(item) => match item {
        parser::Item::Decl(var) => {
          let tuple_type = TupleType::Tuple(var.node.arg_types.iter().map(|ty| {
            match ty.node {
              TypeNode::Boolean => TupleType::Boolean,
              TypeNode::Integer => TupleType::Integer,
              TypeNode::String => TupleType::String,
              TypeNode::Symbol => TupleType::Symbol,
            }
          }).collect::<Vec<_>>());
          let var_name = var.node.predicate;
          match prog.add_variable(&var_name, tuple_type) {
            Ok(_) => {},
            Err(e) => println!("{}", e),
          }
        },
        parser::Item::Fact(f) => {
          println!("Trying to decl fact {:?}", f);
        },
        parser::Item::Disjunction(d) => {
          println!("Trying to decl disjunction {:?}", d);
        },
        parser::Item::Rule(r) => {
          println!("Trying to decl rule {:?}", r);
        },
        parser::Item::Query(q) => {
          println!("Trying to decl query {:?}", q);
        }
      },
      Err(e) => {
        println!("{}", e);
      }
    }
  }
  Ok(())
}
