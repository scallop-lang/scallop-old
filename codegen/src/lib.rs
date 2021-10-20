#![feature(proc_macro_span)]

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::collections::*;

use scallop_compiler::{
  ast, ast2ram, ast_analysis, ast_transform, error::CompileError, location, parser, ram2rs, visitor, options::CompileOptions,
};

fn token_stream_to_token_list_and_id_span_map(
  offset: usize,
  tokens: TokenStream,
) -> (Vec<String>, HashMap<usize, proc_macro::Span>) {
  let mut str_tokens = vec![];
  let mut id_span_map = HashMap::new();

  let mut tokens_iter = tokens.into_iter();
  loop {
    if let Some(token) = tokens_iter.next() {
      match token {
        proc_macro::TokenTree::Group(g) => {
          let span = g.span();
          let id = offset + str_tokens.len();
          id_span_map.insert(id, span);

          let (open, num_items, close) = match g.delimiter() {
            proc_macro::Delimiter::Parenthesis => ("(".to_string(), 1, ")".to_string()),
            proc_macro::Delimiter::Brace => ("{".to_string(), 1, "}".to_string()),
            proc_macro::Delimiter::Bracket => ("[".to_string(), 1, "]".to_string()),
            proc_macro::Delimiter::None => ("".to_string(), 0, "".to_string()),
          };
          str_tokens.push(open);
          let new_offset = id + num_items;

          let (group_tokens, group_id_span_map) =
            token_stream_to_token_list_and_id_span_map(new_offset, g.stream());
          str_tokens.extend(group_tokens);
          id_span_map.extend(group_id_span_map);

          str_tokens.push(close);
        }
        proc_macro::TokenTree::Ident(i) => {
          let span = i.span();
          let id = offset + str_tokens.len();
          id_span_map.insert(id, span);
          str_tokens.push(format!("{}", i));
        }
        proc_macro::TokenTree::Punct(p) => {
          let mut curr_p = p.clone();
          let mut span = curr_p.span();
          let mut op = format!("{}", curr_p.as_char());
          while curr_p.spacing() == proc_macro::Spacing::Joint {
            let next_token = tokens_iter.next().unwrap();
            if let proc_macro::TokenTree::Punct(next_p) = next_token {
              span = span.join(next_p.span()).unwrap();
              op += &next_p.as_char().to_string();
              curr_p = next_p;
            } else {
              panic!("Should not happen");
            }
          }
          let id = offset + str_tokens.len();
          id_span_map.insert(id, span);
          str_tokens.push(op);
        }
        proc_macro::TokenTree::Literal(l) => {
          let span = l.span();
          let id = offset + str_tokens.len();
          id_span_map.insert(id, span);
          str_tokens.push(format!("{}", l));
        }
      }
    } else {
      break (str_tokens, id_span_map);
    }
  }
}

fn token_stream_to_string_and_offset_span_map(
  tokens: TokenStream,
) -> (String, HashMap<usize, proc_macro::Span>) {
  let (str_tokens, id_span_map) = token_stream_to_token_list_and_id_span_map(0, tokens);
  let mut result_string = String::new();
  let mut offset_span_map = HashMap::new();
  for (i, str_token) in str_tokens.iter().enumerate() {
    if let Some(span) = id_span_map.get(&i) {
      let offset = result_string.len();
      offset_span_map.insert(offset, span.clone());
    }
    result_string += str_token;
    result_string += " ";
  }
  (result_string, offset_span_map)
}

struct ModifyLocation {
  offset_span_map: HashMap<usize, proc_macro::Span>,
}

impl visitor::NodeVisitorMut for ModifyLocation {
  fn visit_location(&mut self, loc: &mut location::Location) -> Result<(), CompileError> {
    if let Some(span) = self.offset_span_map.get(&loc.byte_offset) {
      let start = span.start();
      loc.row = start.line;
      loc.col = start.column;
    }
    Ok(())
  }
}

fn token_stream_to_ast(tokens: TokenStream) -> Result<ast::Program, CodegenError> {
  let (prog_str, offset_span_map) = token_stream_to_string_and_offset_span_map(tokens);
  let mut ast = parser::parse_str(&prog_str).map_err(CodegenError::CopmileError)?;
  let mut modify_location = ModifyLocation { offset_span_map };
  visitor::visit_program_mut(&mut modify_location, &mut ast).map_err(CodegenError::CopmileError)?;
  Ok(ast)
}

fn scallop_codegen(tokens: TokenStream) -> Result<TokenStream, CodegenError> {
  let options = CompileOptions::default();
  let mut result = quote! {};
  let mut iter = tokens.into_iter();
  loop {
    let first = iter.next();
    let second = iter.next();
    match (first, second) {
      (Some(first), Some(second)) => {
        let name = match &first {
          TokenTree::Ident(id) => format!("{}", id),
          _ => return Err(CodegenError::ExpectedProgramName),
        };
        let prog_token_stream = match second {
          TokenTree::Group(g) => g.stream(),
          _ => return Err(CodegenError::ExpectedBrace),
        };
        let mut ast = token_stream_to_ast(prog_token_stream)?;
        let mut analysis_result = ast_analysis::analyze(&ast, &options).map_err(CodegenError::CopmileError)?;
        ast_transform::transform(&mut ast, &mut analysis_result, &options).map_err(CodegenError::CopmileError)?;
        let ram = ast2ram::ast2ram(&ast).map_err(CodegenError::CopmileError)?;
        let rs_tokens = ram2rs::ram2rs(&name, &ram, &analysis_result, &options);
        result.extend(rs_tokens);
      }
      (Some(_), None) => {
        return Err(CodegenError::ExpectedWhole);
      }
      _ => break,
    }
  }
  Ok(result.into())
}

#[proc_macro]
pub fn scallop(tokens: TokenStream) -> TokenStream {
  match scallop_codegen(tokens) {
    Ok(compiled_rs) => compiled_rs,
    Err(err) => {
      println!("\nScallop Codegen Error:");
      println!("{}\n", err);
      (quote! {}).into()
    }
  }
}

enum CodegenError {
  ExpectedProgramName,
  ExpectedBrace,
  ExpectedWhole,
  CopmileError(CompileError),
}

impl std::fmt::Display for CodegenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ExpectedProgramName => write!(f, "Expected program name"),
      Self::ExpectedBrace => write!(f, "Expected curly braces {{ }}"),
      Self::ExpectedWhole => write!(f, "Expected <NAME> {{ <PROGRAM> }}"),
      Self::CopmileError(err) => write!(f, "{}", err),
    }
  }
}

impl std::fmt::Debug for CodegenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl std::error::Error for CodegenError {}
