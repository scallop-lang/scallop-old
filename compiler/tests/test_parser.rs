use scallop_compiler::parser;

#[test]
fn test_parse_query_1() {
  let prog = "
    decl sym(Int, Int).
    decl tri_sym(Int, Int, Int).

    sym(A, B) :- A == B.
    tri_sym(A, B, C) :- sym(A, B), sym(B, C).

    query tri_sym(3, Y, 3).
  ";
  let ast = parser::parse_str(prog).unwrap();
  println!("{:?}", ast);
}
