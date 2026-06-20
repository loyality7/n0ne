use ast::*;
use lexer::Lexer;
use parser::Parser;

fn generate_ir(source: &str) -> String {
    let tokens = Lexer::tokenize(source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut gen = codegen_llvm::LLVMGenerator::new();
    gen.generate(&ast)
}

const PROG_1: &str = "fn add(a: int, b: int) -> int\n    return a + b\n";
const PROG_2: &str = "task loop\n    x = 3\n    while x > 0\n        if x == 2\n            break\n        x = x - 1\n";
const PROG_3: &str = "task matching\n    s = \"hello\"\n    match s\n        \"hello\" -> show(\"hi\")\n        _ -> show(\"no\")\n";
const PROG_4: &str = "type User\n    name: string\nfn (self: User) greet()\n    show(self.name)\n";
const PROG_5: &str = "task items\n    l = [1, 2, 3]\n    m = {\"a\": 1}\n";

#[test]
fn snapshot_lexer_token_streams() {
    let tokens1 = format!("{:#?}", Lexer::tokenize(PROG_1));
    let tokens2 = format!("{:#?}", Lexer::tokenize(PROG_2));
    let tokens3 = format!("{:#?}", Lexer::tokenize(PROG_3));
    let tokens4 = format!("{:#?}", Lexer::tokenize(PROG_4));
    let tokens5 = format!("{:#?}", Lexer::tokenize(PROG_5));

    insta::assert_snapshot!("lexer_token_stream_prog1", tokens1);
    insta::assert_snapshot!("lexer_token_stream_prog2", tokens2);
    insta::assert_snapshot!("lexer_token_stream_prog3", tokens3);
    insta::assert_snapshot!("lexer_token_stream_prog4", tokens4);
    insta::assert_snapshot!("lexer_token_stream_prog5", tokens5);
}

#[test]
fn snapshot_ast_output() {
    let ast1 = format!("{:#?}", Parser::new(Lexer::tokenize(PROG_1)).parse());
    let ast2 = format!("{:#?}", Parser::new(Lexer::tokenize(PROG_2)).parse());
    let ast3 = format!("{:#?}", Parser::new(Lexer::tokenize(PROG_3)).parse());
    let ast4 = format!("{:#?}", Parser::new(Lexer::tokenize(PROG_4)).parse());
    let ast5 = format!("{:#?}", Parser::new(Lexer::tokenize(PROG_5)).parse());

    insta::assert_snapshot!("ast_prog1", ast1);
    insta::assert_snapshot!("ast_prog2", ast2);
    insta::assert_snapshot!("ast_prog3", ast3);
    insta::assert_snapshot!("ast_prog4", ast4);
    insta::assert_snapshot!("ast_prog5", ast5);
}

#[test]
fn snapshot_hello_world_ir() {
    let ir = generate_ir("task main\n    show(\"hello world\")\n");
    insta::assert_snapshot!("hello_world_ir", ir);
}

#[test]
fn snapshot_formatter() {
    let ugly = "task   main\n        show(\"hi\")\n";
    let pretty = formatter::format(ugly);
    insta::assert_snapshot!("formatted_ugly", pretty);
}

#[test]
fn perf_compile_time() {
    let source = "task main\n    show(\"hello world\")\n";
    let start = std::time::Instant::now();
    let res = crate::helpers::compile_n0ne(source);
    assert!(res.is_ok(), "Expected compilation to succeed");
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 5000, "compile took {}ms", elapsed.as_millis());
}

#[test]
fn perf_binary_size() {
    let source = "task main\n    show(\"hello world\")\n";
    let (temp_dir, binary_path) = crate::helpers::compile_to_binary(source);
    let size = std::fs::metadata(&binary_path).unwrap().len();
    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(size < 1_000_000, "binary too large: {} bytes", size);
}
