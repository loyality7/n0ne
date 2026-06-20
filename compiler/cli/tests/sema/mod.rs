fn sema_error(source: &str, error_code: &str) {
    crate::helpers::assert_error(source, error_code);
}

fn sema_ok(source: &str) {
    let res = crate::helpers::compile_n0ne(source);
    assert!(res.is_ok(), "Expected compilation to succeed, but got errors: {:?}", res.err());
}

#[test]
fn test_sema_e001_type_mismatch() {
    sema_error("task main\n    x = 10\n    x = \"hello\"\n", "E001");
}

#[test]
fn test_sema_e002_undefined_variable() {
    sema_error("task main\n    show(x)\n", "E002");
}

#[test]
fn test_sema_e003_undefined_function() {
    sema_error("task main\n    nonexistent()\n", "E003");
}

#[test]
fn test_sema_e004_wrong_arg_count() {
    sema_error("fn f(a: int)\n    show(a.to_string())\ntask main\n    f(1, 2)\n", "E004");
}

#[test]
fn test_sema_e005_wrong_arg_type() {
    sema_error("fn f(a: int)\n    show(a.to_string())\ntask main\n    f(\"hello\")\n", "E005");
}

#[test]
fn test_sema_e006_missing_return() {
    sema_error("fn add(a: int, b: int) -> int\n    show(\"oops\")\n", "E006");
}

#[test]
fn test_sema_e007_unreachable_code() {
    sema_error("fn f() -> int\n    return 1\n    show(\"dead\")\n", "E007");
}

#[test]
fn test_sema_e009_circular_import() {
    use std::fs;
    let temp_dir = std::env::temp_dir().join("n0ne_circular_import_test");
    let _ = fs::create_dir_all(&temp_dir);
    
    let a_path = temp_dir.join("a.n0");
    let b_path = temp_dir.join("b.n0");
    
    fs::write(&a_path, "use ./b\ntask main\n    show(\"a\")\n").unwrap();
    fs::write(&b_path, "use ./a\ntask main\n    show(\"b\")\n").unwrap();
    
    let cli_path = env!("CARGO_BIN_EXE_n0ne");
    let output = std::process::Command::new(cli_path)
        .arg("build")
        .arg(&a_path)
        .current_dir(&temp_dir)
        .output()
        .unwrap();
        
    let _ = fs::remove_dir_all(&temp_dir);
    
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains("E009"), "Expected E009 circular import error, but got stderr:\n{}", stderr);
}

#[test]
fn test_sema_e012_duplicate_name() {
    sema_error("fn f()\n    show(\"\")\nfn f()\n    show(\"\")\n", "E012");
}

#[test]
fn test_sema_e014_break_outside_loop() {
    sema_error("task main\n    break\n", "E014");
}

#[test]
fn test_sema_e015_continue_outside_loop() {
    sema_error("task main\n    continue\n", "E015");
}

#[test]
fn test_sema_valid_programs() {
    sema_ok("task main\n    show(\"hello\")\n");
    sema_ok("fn add(a: int, b: int) -> int\n    return a + b\ntask main\n    add(1, 2)\n");
    sema_ok("type User\n    name: string\ntask main\n    u = User()\n    u.name = \"x\"\n");
    sema_ok("task main\n    while true\n        break\n");
}
