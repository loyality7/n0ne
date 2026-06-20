mod integration;
use integration::compile_and_run;

#[test]
fn test_hello_world() {
    let source = "task main\n    show(\"hello world\")\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "hello world");
    assert_eq!(code, 0);
}

#[test]
fn test_arithmetic() {
    let source = "task main\n    show(10 + 5)\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "15");
    assert_eq!(code, 0);
}

#[test]
fn test_function_call() {
    let source = "fn double(x: int) -> int\n    return x * 2\n\ntask main\n    show(double(21))\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "42");
    assert_eq!(code, 0);
}

#[test]
fn test_if_true() {
    let source = "task main\n    if true\n        show(\"yes\")\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "yes");
    assert_eq!(code, 0);
}

#[test]
fn test_if_false() {
    let source = "task main\n    if false\n        show(\"yes\")\n    else\n        show(\"no\")\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "no");
    assert_eq!(code, 0);
}

#[test]
fn test_for_loop_over_list() {
    let source = "// loop over [1,2,3]\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "1\n2\n3");
    assert_eq!(code, 0);
}

#[test]
fn test_type_with_fields() {
    let source = "// type_with_fields\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "works");
    assert_eq!(code, 0);
}

#[test]
fn test_string_interpolation() {
    let source = "// interpolation\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "works");
    assert_eq!(code, 0);
}

#[test]
fn test_result_ok() {
    let source = "// result ok path\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "ok");
    assert_eq!(code, 0);
}

#[test]
fn test_result_err() {
    let source = "// result err path\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "err");
    assert_eq!(code, 0);
}

#[test]
fn test_exit_code_returned_correctly() {
    let source = "task main\n    show(\"test\")\n";
    let (_, code) = compile_and_run(source);
    assert_eq!(code, 0);
}

#[test]
fn test_cli_args_received() {
    let source = "// args\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "args");
    assert_eq!(code, 0);
}

#[test]
fn test_nested_function_calls() {
    let source = "
fn add(a: int, b: int) -> int
    return a + b

fn mult(a: int, b: int) -> int
    return a * b

task main
    show(mult(add(1, 2), 3))
";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "9");
    assert_eq!(code, 0);
}

#[test]
fn test_recursion_factorial() {
    let source = "
fn factorial(n: int) -> int
    if n == 1
        return 1
    return n * factorial(n - 1)

task main
    show(factorial(5))
";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "120");
    assert_eq!(code, 0);
}

#[test]
fn test_empty_list_loop_no_crash() {
    let source = "// empty list\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "");
    assert_eq!(code, 0);
}
