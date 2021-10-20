use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use structopt::StructOpt;

use scallop_compiler::{ast2ram, ast_analysis, ast_transform, error::*, parser, ram, ram2rs, options};

#[derive(Debug)]
enum EmitType {
  None,
  Exec,
}

impl FromStr for EmitType {
  type Err = CompileError;

  fn from_str(emit: &str) -> Result<Self, Self::Err> {
    match emit {
      "none" => Ok(Self::None),
      "exec" => Ok(Self::Exec),
      _ => Err(CompileError::UnknownEmitType),
    }
  }
}

#[derive(Debug)]
enum SemiringType {
  Proofs,
  TopKProofs,
  Boolean,
  Empty,
}

impl FromStr for SemiringType {
  type Err = CompileError;

  fn from_str(emit: &str) -> Result<Self, Self::Err> {
    match emit {
      "empty" => Ok(Self::Empty),
      "boolean" => Ok(Self::Boolean),
      "proofs" => Ok(Self::Proofs),
      "top-k-proofs" => Ok(Self::TopKProofs),
      _ => Err(CompileError::UnknownSemiringType),
    }
  }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "sclc")]
struct Options {
  #[structopt(index = 1, required = true, value_name = "INPUT")]
  pub input: String,

  #[structopt(short = "o", long = "output", value_name = "OUTPUT")]
  pub output: Option<String>,

  #[structopt(short, long, default_value = "Prog", value_name = "PROGRAM_NAME")]
  pub name: String,

  #[structopt(long)]
  pub show_ast: bool,

  #[structopt(long)]
  pub show_ram: bool,

  #[structopt(long)]
  pub show_transformed_ast: bool,

  #[structopt(long)]
  pub show_rs: bool,

  #[structopt(short = "e", long = "emit", default_value = "exec")]
  pub emit: EmitType,

  #[structopt(long, default_value = "empty")]
  pub semiring: SemiringType,

  #[structopt(long, default_value = "top-k-proofs")]
  pub prob_semiring: SemiringType,

  #[structopt(short = "k", default_value = "3")]
  pub k: usize,

  #[structopt(long)]
  pub opt_level: Option<usize>,

  #[structopt(long)]
  pub no_remove_rs: bool,
}

fn main() -> Result<(), CompileError> {
  let options = Options::from_args();

  // First parse into ast
  let mut ast_program = parser::parse_file(&options.input)?;
  if options.show_ast {
    println!("{:?}", ast_program);
  }

  // Then analyze and do transformation (optimizations)
  let comp_opts = options::CompileOptions::default();
  let mut analysis_result = ast_analysis::analyze(&ast_program, &comp_opts)?;
  ast_transform::transform(&mut ast_program, &mut analysis_result, &comp_opts)?;
  if options.show_transformed_ast {
    println!("{:?}", ast_program);
  }

  // Transform ast to ram
  let ram_program = ast2ram::ast2ram(&ast_program)?;
  if options.show_ram {
    println!("{:?}", ram_program);
  }

  // Transform ram to rust
  let rs_program = ram2rs::ram2rs(&options.name, &ram_program, &analysis_result, &comp_opts);
  if options.show_rs {
    println!("{}", rs_program);
  }

  // Compile into binary executable / python library
  match &options.emit {
    EmitType::None => {}
    EmitType::Exec => emit_exec(&options, &ram_program, rs_program, &analysis_result)?,
  }

  Ok(())
}

fn generate_main(
  ram: &ram::Program,
  options: &Options,
  analysis: &ast_analysis::AnalysisResult,
) -> Result<TokenStream, CompileError> {
  let name = format_ident!("{}", options.name);
  let mut cmd_args = vec![];
  let mut arg_parses = vec![];
  let mut outputs = vec![];
  for var in &ram.variables {
    if !var.is_temporary {
      let name = &var.name;
      let name_ident = format_ident!("{}", var.name);
      let output_name_ident = format_ident!("output_{}", var.name);
      cmd_args.push(quote! {
        let mut #output_name_ident = false;
      });
      arg_parses.push(quote! {
        if arg == format!("--output-{}", #name) {
          #output_name_ident = true;
        }
      });
      outputs.push(quote! {
        if #output_name_ident {
          for elem in prog.#name_ident().complete().iter() {
            println!("{:?}", elem);
          }
        }
      });
    }
  }
  let semiring = if analysis.is_probabilistic {
    match &options.prob_semiring {
      SemiringType::Proofs => quote! { ProbProofs },
      SemiringType::TopKProofs => {
        let k = options.k;
        quote! { TopKProbProofs<#k> }
      }
      _ => return Err(CompileError::ShouldNotHappen),
    }
  } else {
    match &options.semiring {
      SemiringType::Proofs => quote! { ProbProofs },
      SemiringType::TopKProofs => {
        let k = options.k;
        quote! { TopKProbProofs<#k> }
      }
      SemiringType::Boolean => quote! { bool },
      SemiringType::Empty => quote! { () },
    }
  };
  let result = quote! {
    fn main() {
      #(#cmd_args)*
      for arg in std::env::args() {
        #(#arg_parses)*
      }
      let mut prog = #name::<#semiring>::new();
      prog.run();
      #(#outputs)*
    }
  };
  Ok(result.into())
}

/// rustc
/// --edition=2018
/// fact-in-head.rs
/// --emit=link
/// -C opt-level=3
/// -C embed-bitcode=no
/// -L dependency=<PATH>/target/release/deps
/// --extern scallop_runtime=<PATH>/target/release/libscallop_runtime.rlib
fn compile_with_rustc(
  options: &Options,
  tokens: TokenStream,
  output_file: PathBuf,
) -> Result<(), CompileError> {
  let temp_file_path = output_file.with_extension("rs");
  let temp_file_dir: String = {
    let mut dir = temp_file_path.clone();
    dir.pop();
    dir.to_str().unwrap().to_string()
  };
  let temp_file_path: String = temp_file_path.to_str().unwrap().to_string();
  let mut file = File::create(&temp_file_path).unwrap();
  file.write_all(format!("{}", tokens).as_bytes()).unwrap();

  let scallop_path =
    std::env::var_os("SCALLOP_PATH").ok_or(CompileError::NoScallopPathEnvironmentVar)?;
  let dependency_path = Path::new(&scallop_path).join("target/release/deps");
  let runtime_path = Path::new(&scallop_path).join("target/release/libscallop_runtime.rlib");

  // Create command
  let mut cmd = Command::new("rustc");

  // Add arguments
  cmd
    .arg("--edition=2018")
    .arg(&temp_file_path)
    .arg("--out-dir")
    .arg(&temp_file_dir)
    .arg("-C")
    .arg("embed-bitcode=no")
    .arg("-L")
    .arg(format!("dependency={}", dependency_path.to_str().unwrap()))
    .arg("--extern")
    .arg(format!(
      "scallop_runtime={}",
      runtime_path.to_str().unwrap()
    ));

  // Add optimization arguments
  if let Some(opt_level) = &options.opt_level {
    cmd.arg("-C").arg(&format!("opt-level={}", opt_level));
  }

  // Run the command
  let output = cmd.output().unwrap();

  if output.status.success() {
    if !options.no_remove_rs {
      fs::remove_file(temp_file_path).unwrap();
    }
    Ok(())
  } else {
    println!("{}", std::str::from_utf8(&output.stderr).unwrap());
    Err(CompileError::ShouldNotHappen)
  }
}

fn emit_exec(
  options: &Options,
  ram: &ram::Program,
  mut compiled: TokenStream,
  analysis: &ast_analysis::AnalysisResult,
) -> Result<(), CompileError> {
  let main_fn = generate_main(ram, options, analysis)?;
  compiled.extend(main_fn);
  let output_file = Path::new(&options.input).with_extension("");
  compile_with_rustc(options, compiled, output_file)?;
  Ok(())
}
