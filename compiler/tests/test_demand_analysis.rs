use scallop_compiler::{*, options::CompileOptions};

#[test]
fn test_demand_analysis_1() {
  let prog_str = "
    decl base(Int, Int).
    decl foo(Int, Int).
    decl bar(Int, Int, Int).

    foo(A, B) :- base(A, B).
    bar(A, B, C) :- foo(A, B), foo(B, C).

    query bar(3, B, 3).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let ast = parser::parse_str(prog_str).unwrap();
  let analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  println!("{:?}", analysis.demands);
}

#[test]
fn test_demand_analysis_2() {
  let prog_str = "
    decl edge(Int, Int).
    decl path(Int, Int).

    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).

    query path(3, C).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let ast = parser::parse_str(prog_str).unwrap();
  let analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  println!("{:?}", analysis.demands);
}

/// Testing having multiple quries
#[test]
fn test_demand_analysis_3() {
  let prog_str = "
    decl edge(Int, Int).
    decl path(Int, Int).

    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).

    query path(3, C).
    query path(A, 8).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let ast = parser::parse_str(prog_str).unwrap();
  let analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  println!("{:?}", analysis.demands);
}

#[test]
fn test_demand_analysis_4() {
  let prog_str = "
    decl edge(Int, Int).
    decl path(Int, Int).

    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).

    query path(3, C).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}

#[test]
fn test_demand_analysis_5() {
  let prog_str = "
    decl edge(Int, Int).
    decl path(Int, Int).

    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).

    query path(A, 7).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}

#[test]
fn test_demand_analysis_6() {
  let prog_str = "
    decl edge(Int, Int).
    decl path(Int, Int).

    path(A, B) :- edge(A, B).
    path(A, C) :- edge(A, B), path(B, C).

    query path(2, 7).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}

#[test]
fn test_demand_analysis_7() {
  let prog_str = "
    decl eval(Symbol, Int).
    decl binary_op(Symbol, String, Symbol, Symbol).
    decl unary_op(Symbol, String, Symbol).
    decl constant(Symbol, Int).

    eval(N, R) :- constant(N, R).
    eval(N, RA + RB) :- binary_op(N, \"+\", A, B), eval(A, RA), eval(B, RB).
    eval(N, RA - RB) :- binary_op(N, \"-\", A, B), eval(A, RA), eval(B, RB).
    eval(N, -R) :- unary_op(N, \"-\", M), eval(M, R).

    constant(0, 3).
    constant(1, 5).
    binary_op(2, \"+\", 0, 1).

    query eval(2, R).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}

#[test]
fn test_demand_analysis_8() {
  let prog_str = "
    decl base(Int, Int).
    decl foo(Int, Int).
    decl bar(Int, Int, Int).

    foo(A, B) :- base(A, B).
    bar(A, B, C) :- foo(A, B), foo(B, C).

    query bar(3, B, 3).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}

#[test]
fn test_demand_analysis_9() {
  let prog_str = "
    decl father(Symbol, Symbol).
    decl mother(Symbol, Symbol).
    decl parent(Symbol, Symbol).
    decl grand_parent(Symbol, Symbol).

    parent(X, Y) :- father(X, Y).
    parent(X, Y) :- mother(X, Y).
    grand_parent(X, Z) :- parent(X, Y), parent(Y, Z).

    father(0, 1).
    mother(0, 2).
    father(1, 3).
    mother(1, 4).
    father(2, 5).
    mother(2, 6).

    query grand_parent(0, Z).
  ";
  let opt = CompileOptions::default().with_demand_transform();
  let mut ast = parser::parse_str(prog_str).unwrap();
  let mut analysis = ast_analysis::analyze(&ast, &opt).unwrap();
  ast_transform::transform(&mut ast, &mut analysis, &opt).unwrap();
  println!("{:?}", analysis.demands);
  println!("{}", ast.codify());
}
