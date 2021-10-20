use std::fs::File;
use std::io::prelude::*;
use termion::color;

use super::ast::*;
use super::error::*;
use super::location::*;
use super::syntax;
use super::visitor::*;

pub enum Item {
  Decl(Decl),
  Fact(Fact),
  Disjunction(Disjunction),
  Rule(Rule),
  Query(Query),
}

fn row_col(src: &str, byte_offset: usize) -> (usize, usize) {
  let mut curr_line_num = 1;
  let mut curr_line_start = 0;
  for i in 0..src.len() {
    if src.chars().nth(i).unwrap() == '\n' {
      curr_line_num += 1;
      curr_line_start = i + 1;
    }
    if i == byte_offset {
      return (curr_line_num, i - curr_line_start + 1);
    }
  }
  panic!("Byte offset not hit");
}

type ParseError<'a> = lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'a str>;

fn print_syntax_error(s: &str, e: ParseError) -> CompileError {
  match &e {
    ParseError::UnrecognizedToken { token, .. } => {
      let offset = token.0;
      let (row, col) = row_col(s, offset);
      println!(
        "{}[Error]{} Syntax error at row {} col {}:",
        color::Fg(color::Red),
        color::Fg(color::Reset),
        row,
        col,
      );
      let snippet = s[(offset - 10).max(10)..(offset + 10).min(s.len())].to_string();
      println!("{}", snippet);
      println!("{}", e);
    }
    _ => {
      println!("{}", e);
    }
  }
  CompileError::SyntaxError
}

pub fn parse_rule(s: &str) -> Result<Rule, CompileError> {
  let parser = syntax::RuleParser::new();
  let result = parser.parse(&s).map_err(|e| {
    print_syntax_error(s, e)
  });
  result
}

/// Parse a query atom
///
/// This is actually parsing an atom, of the form `predicate(arg0, arg1, ...)`.
/// To parse for query one would actually need a `query` keyword in the front
pub fn parse_query(s: &str) -> Result<Query, CompileError> {
  let parser = syntax::AtomParser::new();
  let result = parser.parse(&s).map_err(|e| {
    print_syntax_error(s, e)
  });
  result.map(|a| {
    Query {
      location: a.location.clone(),
      node: QueryNode::new(a),
    }
  })
}

pub fn parse_item(s: &str) -> Result<Item, CompileError> {
  let parser = syntax::ItemParser::new();
  let result = parser.parse(&s).map_err(|e| {
    print_syntax_error(s, e)
  });
  result
}

fn parse_items(s: &str) -> Result<Vec<Item>, CompileError> {
  let parser = syntax::ItemsParser::new();
  let result = parser.parse(&s).map_err(|e| {
    print_syntax_error(s, e)
  });
  result
}

struct LocationAssigner {
  id_counter: usize,
  newlines: Vec<usize>,
}

impl LocationAssigner {
  fn new(source: &str) -> Self {
    let newlines: Vec<usize> = source
      .char_indices()
      .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
      .collect();
    Self {
      id_counter: 0,
      newlines: newlines,
    }
  }

  fn assign_and_increment_counter(&mut self, l: &mut Location) {
    let (row, col) = self.row_col(l.byte_offset);
    l.id = self.id_counter;
    l.row = row;
    l.col = col;
    self.id_counter += 1;
  }

  // Faster than the other row_col because all the newline locations are cached
  fn row_col(&self, byte_offset: usize) -> (usize, usize) {
    let line_num = self
      .newlines
      .binary_search(&byte_offset)
      .unwrap_or_else(|x| x);
    let last_newline = if line_num == 0 {
      0
    } else {
      self.newlines[line_num - 1]
    };
    return (line_num + 1, byte_offset - last_newline);
  }
}

impl NodeVisitorMut for LocationAssigner {
  fn visit_location(&mut self, l: &mut Location) -> Result<(), CompileError> {
    self.assign_and_increment_counter(l);
    Ok(())
  }
}

fn assign_node_locations(src: &str, ast: &mut Program) {
  let mut assigner = LocationAssigner::new(src);

  // Use unwrap because this cannot fail
  visit_program_mut(&mut assigner, ast).unwrap();
}

pub fn parse_str(s: &str) -> Result<Program, CompileError> {
  let items = parse_items(s)?;
  let mut decls = vec![];
  let mut facts = vec![];
  let mut disjunctions = vec![];
  let mut rules = vec![];
  let mut queries = vec![];
  for item in items {
    match item {
      Item::Decl(d) => decls.push(d),
      Item::Fact(f) => facts.push(f),
      Item::Disjunction(d) => disjunctions.push(d),
      Item::Rule(r) => rules.push(r),
      Item::Query(q) => queries.push(q),
    }
  }
  let mut ast = Program {
    decls,
    facts,
    disjunctions,
    rules,
    queries,
  };
  assign_node_locations(s, &mut ast);
  Ok(ast)
}

pub fn parse_file(filename: &str) -> Result<Program, CompileError> {
  // Initialize a string containing file content
  let mut contents = String::new();

  // Open the file with the given file path
  let mut file = File::open(filename).map_err(|_| CompileError::CannotOpenFile)?;

  // Read the file content into the string
  file
    .read_to_string(&mut contents)
    .map_err(|_| CompileError::CannotReadFile)?;

  // Parse the string
  parse_str(&contents)
}
