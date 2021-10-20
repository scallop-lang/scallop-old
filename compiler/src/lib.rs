pub mod ast;
pub mod ast2ram;
pub mod ast_analysis;
pub mod ast_transform;
pub mod common;
pub mod error;
pub mod location;
pub mod parser;
pub mod ram;
pub mod ram2rs;
pub mod visitor;
pub mod options;

mod syntax;

use error::CompileError;
use proc_macro2::TokenStream;

use options::CompileOptions;

pub fn compile_file_to_rs(file_name: &str) -> Result<TokenStream, CompileError> {
  let options = CompileOptions::default();
  let mut ast_program = parser::parse_file(&file_name)?;
  let mut analysis_result = ast_analysis::analyze(&ast_program, &options)?;
  ast_transform::transform(&mut ast_program, &mut analysis_result, &options)?;
  let ram_program = ast2ram::ast2ram(&ast_program)?;
  Ok(ram2rs::ram2rs("Prog", &ram_program, &analysis_result, &options))
}

pub fn compile_str_to_rs(s: &str) -> Result<TokenStream, CompileError> {
  let options = CompileOptions::default();
  let mut ast_program = parser::parse_str(s)?;
  let mut analysis_result = ast_analysis::analyze(&ast_program, &options)?;
  ast_transform::transform(&mut ast_program, &mut analysis_result, &options)?;
  let ram_program = ast2ram::ast2ram(&ast_program)?;
  Ok(ram2rs::ram2rs("Prog", &ram_program, &analysis_result, &options))
}

pub fn compile_ast_to_rs(ast: &ast::Program) -> Result<TokenStream, CompileError> {
  let options = CompileOptions::default();
  let mut ast_program = ast.clone();
  let mut analysis_result = ast_analysis::analyze(&ast_program, &options)?;
  ast_transform::transform(&mut ast_program, &mut analysis_result, &options)?;
  let ram_program = ast2ram::ast2ram(&ast_program)?;
  Ok(ram2rs::ram2rs("Prog", &ram_program, &analysis_result, &options))
}
