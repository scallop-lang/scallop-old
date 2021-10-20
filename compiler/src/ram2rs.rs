use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn;

use super::ast_analysis::*;
use super::common::*;
use super::ram::*;
use super::options::CompileOptions;

fn type_to_rs(ty: &Type, _: &CompileOptions) -> TokenStream {
  match ty {
    Type::Boolean => quote! { bool },
    Type::Integer => quote! { i64 },
    Type::String => quote! { &'static str },
    Type::Symbol => quote! { usize },
  }
}

fn var_type_to_rs(ty: &VarType, o: &CompileOptions) -> TokenStream {
  match ty {
    VarType::Empty => quote! { () },
    VarType::Base(b) => type_to_rs(b, o),
    VarType::Tuple(t) => {
      let elems = t.iter().map(|vt| var_type_to_rs(vt, o)).collect::<Vec<_>>();
      quote! { (#(#elems),*) }
    }
  }
}

fn const_to_rs(constant: &Constant, _: &CompileOptions) -> TokenStream {
  match constant {
    Constant::Symbol(s) => quote! { #s },
    Constant::Boolean(b) => quote! { #b },
    Constant::Integer(i) => quote! { #i },
    Constant::String(s) => quote! { #s },
  }
}

fn bin_op_to_rs(bin_op: &BinaryOp, _: &CompileOptions) -> TokenStream {
  match bin_op {
    BinaryOp::Eq => quote! { == },
    BinaryOp::Ne => quote! { != },
    BinaryOp::Lt => quote! { < },
    BinaryOp::Lte => quote! { <= },
    BinaryOp::Gt => quote! { > },
    BinaryOp::Gte => quote! { >= },
    BinaryOp::And => quote! { && },
    BinaryOp::Or => quote! { || },
    BinaryOp::Add => quote! { + },
    BinaryOp::Sub => quote! { - },
    BinaryOp::Mult => quote! { * },
    BinaryOp::Div => quote! { / },
  }
}

fn una_op_to_rs(una_op: &UnaryOp, _: &CompileOptions) -> TokenStream {
  match una_op {
    UnaryOp::Pos => quote! { + },
    UnaryOp::Neg => quote! { - },
    UnaryOp::Not => quote! { ! },
  }
}

fn arg_to_rs(arg: &Argument, o: &CompileOptions) -> TokenStream {
  match arg {
    Argument::Element(indices) => {
      if indices.len() == 0 {
        quote! { arg }
      } else {
        let indices = indices.iter().map(|id| syn::Index::from(*id));
        quote! { arg.#(#indices).* }
      }
    }
    Argument::Tuple(tuple) => {
      let elems = tuple.iter().map(|a| arg_to_rs(a, o)).collect::<Vec<_>>();
      quote! { (#(#elems),*) }
    }
    Argument::Constant(constant) => const_to_rs(constant, o),
    Argument::Binary(op, op1, op2) => {
      let rs_op = bin_op_to_rs(op, o);
      let rs_op1 = arg_to_rs(op1, o);
      let rs_op2 = arg_to_rs(op2, o);
      quote! { #rs_op1 #rs_op #rs_op2 }
    }
    Argument::Unary(op, op1) => {
      let rs_op = una_op_to_rs(op, o);
      let rs_op1 = arg_to_rs(op1, o);
      quote! { #rs_op #rs_op1 }
    }
  }
}

fn flow_to_rs_helper(flow: &Flow, is_arg: bool, o: &CompileOptions) -> TokenStream {
  match flow {
    Flow::Product(f1, f2) => {
      let f1_rs = flow_to_rs_helper(f1, true, o);
      let f2_rs = flow_to_rs_helper(f2, true, o);
      quote! { self.iter.product(#f1_rs, #f2_rs) }
    }
    Flow::Intersect(f1, f2) => {
      let f1_rs = flow_to_rs_helper(f1, true, o);
      let f2_rs = flow_to_rs_helper(f2, true, o);
      quote! { self.iter.intersect(#f1_rs, #f2_rs) }
    }
    Flow::Join(f1, f2) => {
      let f1_rs = flow_to_rs_helper(f1, true, o);
      let f2_rs = flow_to_rs_helper(f2, true, o);
      quote! { self.iter.join(#f1_rs, #f2_rs) }
    }
    Flow::Filter(flow, filter) => {
      let flow_rs = flow_to_rs_helper(flow, false, o);
      let filter_rs = arg_to_rs(filter, o);
      quote! { #flow_rs.filter(|arg| #filter_rs) }
    }
    Flow::Project(flow, project) => {
      let flow_rs = flow_to_rs_helper(flow, false, o);
      let project_rs = arg_to_rs(project, o);
      quote! { #flow_rs.project(|arg| #project_rs) }
    }
    Flow::Find(flow, key) => {
      let flow_rs = flow_to_rs_helper(flow, false, o);
      let key_rs = const_to_rs(key, o);
      quote! { #flow_rs.find(#key_rs) }
    }
    Flow::ContainsChain(flow_to_find, key, source) => {
      let f1_rs = flow_to_rs_helper(flow_to_find, true, o);
      let key_rs = arg_to_rs(&Argument::Tuple(key.iter().map(|c| {
        Argument::Constant(c.clone())
      }).collect::<Vec<_>>()), o);
      let f2_rs = flow_to_rs_helper(source, true, o);
      quote! { self.iter.contains_chain(#f1_rs, #key_rs, #f2_rs) }
    }
    Flow::Variable(var) => {
      let var = format_ident!("{}", var);
      if is_arg {
        quote! { &self.#var }
      } else {
        quote! { self.#var }
      }
    }
  }
}

fn flow_to_rs(flow: &Flow, o: &CompileOptions) -> TokenStream {
  flow_to_rs_helper(flow, true, o)
}

fn semiring_constraint(analysis: &AnalysisResult, _: &CompileOptions) -> TokenStream {
  if analysis.is_probabilistic {
    quote! { where Tag: Semiring<Context = ProbProofContext>, ProbProofContext: SemiringContext<Tag, Info = f32> }
  } else {
    quote! { where Tag: Semiring }
  }
}

fn struct_def(name: &str, ram: &Program, analysis: &AnalysisResult, o: &CompileOptions) -> TokenStream {
  let name = format_ident!("{}", name);
  let constraint = semiring_constraint(analysis, o);
  let variables = ram
    .variables
    .iter()
    .map(|var| {
      let name = format_ident!("{}", &var.name);
      let arg_types = var_type_to_rs(&var.arg_types, o);
      quote! { #name: Variable<#arg_types, Tag> }
    })
    .collect::<Vec<_>>();
  quote! {
    pub struct #name<Tag> #constraint {
      iter: Iteration<Tag>,
      #(#variables),*
    }
  }
}

fn update_to_rs(update: &Update, o: &CompileOptions) -> TokenStream {
  let into_var = format_ident!("{}", &update.into_var);
  let flow = flow_to_rs(&update.flow, o);
  quote! { self.iter.insert_dataflow(&self.#into_var, #flow); }
}

fn impl_handles(name: &str, ram: &Program, analysis: &AnalysisResult, o: &CompileOptions) -> TokenStream {
  let name = format_ident!("{}", &name);
  let constraint = semiring_constraint(analysis, o);
  let handle_accessors = ram.variables.iter().filter_map(|var| {
    if var.is_temporary {
      None
    } else {
      let var_name = format_ident!("{}", var.name);
      let arg_types = var_type_to_rs(&var.arg_types, o);
      Some(quote! {
        pub fn #var_name<'a>(&'a mut self) -> VariableHandle<'a, #arg_types, Tag> {
          self.iter.variable_handle(&self.#var_name)
        }
      })
    }
  });
  quote! {
    impl<Tag> #name<Tag> #constraint {
      #(#handle_accessors)*
    }
  }
}

fn fact_insertion(ram: &Program, o: &CompileOptions) -> Vec<TokenStream> {
  ram
    .variables
    .iter()
    .map(|var| {
      let name = format_ident!("{}", var.name);
      let (prob_facts, non_prob_facts): (Vec<&Fact>, Vec<&Fact>) = ram
        .facts
        .iter()
        .filter(|fact| fact.predicate == var.name)
        .partition(|fact| fact.prob.is_some());
      let insert_prob_facts = prob_facts
        .iter()
        .map(|fact| {
          let prob = fact.prob.unwrap();
          let tup = fact.args.iter().map(|c| const_to_rs(c, o));
          quote! { (#prob, (#(#tup),*)) }
        })
        .collect::<Vec<_>>();
      let insert_non_prob_facts = non_prob_facts
        .iter()
        .map(|fact| {
          let tup = fact.args.iter().map(|c| const_to_rs(c, o));
          quote! { (#(#tup),*) }
        })
        .collect::<Vec<_>>();
      let insert_prob_facts_quote = if insert_prob_facts.len() > 0 {
        quote! {
          self.iter.insert_with_tag_info(&self.#name, [#(#insert_prob_facts),*].to_vec());
        }
      } else {
        quote! {}
      };
      let insert_non_prob_facts_quote = if insert_non_prob_facts.len() > 0 {
        quote! {
          self.iter.insert_ground(&self.#name, [#(#insert_non_prob_facts),*].to_vec());
        }
      } else {
        quote! {}
      };
      quote! {
        #insert_prob_facts_quote
        #insert_non_prob_facts_quote
      }
    })
    .collect::<Vec<_>>()
}

fn disjunction_insertion(ram: &Program, analysis: &AnalysisResult, o: &CompileOptions) -> Vec<TokenStream> {
  ram
    .disjunctions
    .iter()
    .map(|disjunction| {
      let name = format_ident!("{}", &analysis.disj_rela_map[&disjunction.id]);
      let facts = disjunction
        .facts
        .iter()
        .map(|fact| {
          let prob = match &fact.prob {
            Some(p) => quote! { #p },
            None => quote! { 1.0f32 },
          };
          let tup = fact.args.iter().map(|c| const_to_rs(c, o));
          quote! { ( #prob, (#(#tup),*) ) }
        })
        .collect::<Vec<_>>();
      quote! {
        self.iter.insert_disjunction(&self.#name, vec![#(#facts),*]);
      }
    })
    .collect::<Vec<_>>()
}

fn impl_prog(name: &str, ram: &Program, analysis: &AnalysisResult, o: &CompileOptions) -> TokenStream {
  let name = format_ident!("{}", name);

  let constraint = semiring_constraint(analysis, o);

  let init_variables = ram
    .variables
    .iter()
    .map(|var| {
      let arg_types = var_type_to_rs(&var.arg_types, o);
      let raw_name = &var.name;
      let name = format_ident!("{}", var.name);
      quote! { let #name = iter.static_variable::<#arg_types>(#raw_name); }
    })
    .collect::<Vec<_>>();

  let init_struct_fields = ram
    .variables
    .iter()
    .map(|var| format_ident!("{}", var.name))
    .collect::<Vec<_>>();

  let var_facts_insertion = fact_insertion(ram, o);

  let var_disjunction_insertion = disjunction_insertion(ram, analysis, o);

  let updates = ram.updates.iter().map(|u| update_to_rs(u, o)).collect::<Vec<_>>();
  quote! {
    impl<Tag> Program<Tag> for #name<Tag> #constraint {
      fn new() -> Self {
        let mut iter = Iteration::new();
        #(#init_variables)*
        Self { iter, #(#init_struct_fields),* }
      }
      fn iteration(&self) -> &Iteration<Tag> {
        &self.iter
      }
      fn iteration_mut(&mut self) -> &mut Iteration<Tag> {
        &mut self.iter
      }
      fn initialize(&mut self) {
        #(#var_facts_insertion)*
        #(#var_disjunction_insertion)*
      }
      fn update(&self) {
        #(#updates)*
      }
    }
  }
}

pub fn ram2rs(
  name: &str,
  ram: &Program,
  analysis: &AnalysisResult,
  options: &CompileOptions,
) -> TokenStream {
  let module_name = format_ident!("scallop_{}", name.to_lowercase());
  let sd = struct_def(name, ram, analysis, options);
  let ih = impl_handles(name, ram, analysis, options);
  let ip = impl_prog(name, ram, analysis, options);
  quote! {
    mod #module_name {
      pub use scallop_runtime::*;
      use scallop_runtime::dataflows::*;
      use scallop_runtime::interpreter::*;
      #sd
      #ih
      #ip
    }
    use #module_name::*;
  }
}
