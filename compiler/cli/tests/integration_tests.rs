mod integration;
use integration::{compile_and_run, compile_and_run_with_files, compile_and_run_with_stdin};

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
    let source = r#"
fn make_list() -> list[int]
    l = c_alloc(24)
    c_store_int(l, 0, 1)
    c_store_int(l, 16, 3)
    data = c_alloc(24)
    c_store_int(data, 0, 1)
    c_store_int(data, 8, 2)
    c_store_int(data, 16, 3)
    c_store_int(l, 8, data)
    return l

task main
    l = make_list()
    for x in l
        show(x)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim().replace("\r\n", "\n"), "1\n2\n3");
    assert_eq!(code, 0);
}

#[test]
fn test_type_with_fields() {
    let source = r#"
type User
    name: string
    age: int

fn make_user(name: string, age: int) -> User
    u = c_alloc(24)
    c_store_int(u, 0, 1)
    c_store_string(u, 8, name)
    c_store_int(u, 16, age)
    return u

task main
    u = make_user("sarath", 30)
    if u.age == 30
        show("works")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "works");
    assert_eq!(code, 0);
}

#[test]
fn test_string_interpolation() {
    let source = r#"
task main
    name = "sarath"
    age = 21
    gpa = 3.8
    happy = true
    msg = f"hello {name} you are {age} years old with gpa {gpa} and happy={happy}"
    show(msg)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "hello sarath you are 21 years old with gpa 3.800000 and happy=true");
    assert_eq!(code, 0);
}

#[test]
fn test_result_ok() {
    let source = r#"
fn make_result(should_fail: int) -> result[string]
    res = c_alloc(32)
    c_store_int(res, 0, 1)
    if should_fail == 1
        c_store_int(res, 8, 1)
        c_store_string(res, 24, "err")
    else
        c_store_int(res, 8, 0)
        c_store_string(res, 16, "ok")
    return res

task main
    res = make_result(0)
    if res.is_err
        show("err")
    else
        show("ok")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "ok");
    assert_eq!(code, 0);
}

#[test]
fn test_result_err() {
    let source = r#"
fn make_result(should_fail: int) -> result[string]
    res = c_alloc(32)
    c_store_int(res, 0, 1)
    if should_fail == 1
        c_store_int(res, 8, 1)
        c_store_string(res, 24, "err")
    else
        c_store_int(res, 8, 0)
        c_store_string(res, 16, "ok")
    return res

task main
    res = make_result(1)
    if res.is_err
        show("err")
    else
        show("ok")
"#;
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
    let source = r#"
task main
    show(c_argv(1))
"#;
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
    let source = r#"
fn make_list() -> list[int]
    l = c_alloc(24)
    c_store_int(l, 0, 1)
    c_store_int(l, 16, 0)
    c_store_int(l, 8, 0)
    return l

task main
    l = make_list()
    for x in l
        show(x)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "");
    assert_eq!(code, 0);
}

#[test]
fn test_hello_world_fn() {
    let source = "fn main()\n    show(\"hello world\")\n";
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "hello world");
    assert_eq!(code, 0);
}

#[test]
fn test_native_list_literals() {
    let source = r#"
task main
    items = [10, 20, 30]
    for x in items
        show(x)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim().replace("\r\n", "\n"), "10\n20\n30");
    assert_eq!(code, 0);
}

#[test]
fn test_native_empty_list_no_crash() {
    let source = r#"
task main
    items = []
    for x in items
        show(x)
    show("done")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "done");
    assert_eq!(code, 0);
}

#[test]
fn test_native_map_literal() {
    let source = r#"
task main
    data = {"key": "value"}
    show("map ok")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(out.trim(), "map ok");
    assert_eq!(code, 0);
}

#[test]
fn test_string_methods() {
    let source = "
task main
    s = \"hello\"
    show(s.len())
    show(s.contains(\"ell\"))
    show(s.contains(\"x\"))
    show(s.starts_with(\"he\"))
    show(s.ends_with(\"lo\"))
    
    u = s.upper()
    show(u)
    
    l = u.lower()
    show(l)
    
    spaced = \"  trim me  \"
    show(spaced.trim())
    
    csv = \"a,b,c\"
    parts = csv.split(\",\")
    show(parts.len())
    
    r = s.replace(\"l\", \"x\")
    show(r)
    
    sl = s.slice(1, 4)
    show(sl)
    
    opt1 = \"123\".to_int()
    if opt1.is_some
        show(opt1.value)
        
    opt2 = \"abc\".to_int()
    if opt2.is_none
        show(999)
        
    opt3 = \"12.34\".to_float()
    if opt3.is_some
        show(opt3.value)
        
    opt4 = \"xyz\".to_float()
    if opt4.is_none
        show(888)
";
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    let expected = "\
5
1
0
1
1
HELLO
hello
trim me
3
hexxo
ell
123
999
12.340000
888";
    assert_eq!(out.trim().replace("\r\n", "\n"), expected.trim());
}

#[test]
fn test_list_methods() {
    let source = "
task main
    items = [10, 20]
    show(items.len())
    
    items.push(30)
    show(items.len())
    
    show(items.contains(20))
    show(items.contains(40))
    
    opt_pop = items.pop()
    if opt_pop.is_some
        show(opt_pop.value)
        
    show(items.len())
    
    opt_first = items.first()
    if opt_first.is_some
        show(opt_first.value)
        
    opt_last = items.last()
    if opt_last.is_some
        show(opt_last.value)
";
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    let expected = "\
2
3
1
0
30
2
10
20";
    assert_eq!(out.trim().replace("\r\n", "\n"), expected.trim());
}

#[test]
fn test_map_methods() {
    let source = "
task main
    data = {\"a\": 1, \"b\": 2}
    show(data.has(\"a\"))
    show(data.has(\"c\"))
    
    data.set(\"c\", 3)
    show(data.has(\"c\"))
    
    opt1 = data.get(\"c\")
    if opt1.is_some
        show(opt1.value)
        
    opt2 = data.get(\"d\")
    if opt2.is_none
        show(777)
        
    data.delete(\"a\")
    show(data.has(\"a\"))
    
    keys = data.keys()
    show(keys.len())
    
    vals = data.values()
    show(vals.len())
";
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    let expected = "\
1
0
1
3
777
0
2
2";
    assert_eq!(out.trim().replace("\r\n", "\n"), expected.trim());
}

#[test]
fn test_numeric_methods() {
    let source = "
task main
    n = 123
    show(n.to_string())
    
    f = 4.56
    show(f.to_string())
    
    show(n.to_float())
    show(f.to_int())
";
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    let expected = "\
123
4.560000
123.000000
4";
    assert_eq!(out.trim().replace("\r\n", "\n"), expected.trim());
}

#[test]
fn test_import_local() {
    let files = vec![
        ("utils.n0", "fn greet(name: string) -> string\n    return \"Hello, \" + name\n"),
        ("main.n0", "use ./utils\n\ntask main\n    show(utils.greet(\"Sarath\"))\n"),
    ];
    let (out, code) = compile_and_run_with_files(files);
    assert_eq!(out.trim(), "Hello, Sarath");
    assert_eq!(code, 0);
}

#[test]
fn test_import_stdlib() {
    let files = vec![
        ("main.n0", "use io\nuse fs\n\ntask main\n    io.show(\"testing io\")\n    fs.write(\"test_file.txt\", \"hello fs\")\n    if fs.exists(\"test_file.txt\")\n        io.show(\"file exists\")\n    fs.delete(\"test_file.txt\")\n"),
    ];
    let (out, code) = compile_and_run_with_files(files);
    assert_eq!(out.trim().replace("\r\n", "\n"), "testing io\nfile exists");
    assert_eq!(code, 0);
}

#[test]
fn test_circular_import_error() {
    let files = vec![
        ("a.n0", "use ./b\n"),
        ("b.n0", "use ./a\n"),
        ("main.n0", "use ./a\ntask main\n    show(1)\n"),
    ];
    let (out, _) = compile_and_run_with_files(files);
    assert!(out.contains("E009") || out.contains("circular"));
}

#[test]
fn test_missing_local_file_error() {
    let files = vec![
        ("main.n0", "use ./missing_file\ntask main\n    show(1)\n"),
    ];
    let (out, _) = compile_and_run_with_files(files);
    assert!(out.contains("E011") || out.contains("not found") || out.contains("does not exist"));
}

#[test]
fn test_unknown_stdlib_module_error() {
    let files = vec![
        ("main.n0", "use not_a_real_stdlib\ntask main\n    show(1)\n"),
    ];
    let (out, _) = compile_and_run_with_files(files);
    assert!(out.contains("E010") || out.contains("unknown standard library module"));
}

#[test]
fn test_fs_write_and_read() {
    let _ = std::fs::remove_file("write_read_test.txt");
    let source = r#"
use io
use fs

task main
    content = "hello standard library"
    try fs.write("write_read_test.txt", content)
    read_back = try fs.read("write_read_test.txt")
    io.show(read_back)
    try fs.delete("write_read_test.txt")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "hello standard library");
}

#[test]
fn test_fs_exists_and_delete() {
    let _ = std::fs::remove_file("exists_test.txt");
    let source = r#"
use io
use fs

task main
    try fs.write("exists_test.txt", "content")
    if fs.exists("exists_test.txt")
        io.show("exists before delete")
    try fs.delete("exists_test.txt")
    if fs.exists("exists_test.txt")
        io.show("exists after delete")
    else
        io.show("does not exist after delete")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    assert_eq!(out.trim().replace("\r\n", "\n"), "exists before delete\ndoes not exist after delete");
}

#[test]
fn test_fs_mkdir() {
    let _ = std::fs::remove_dir("test_dir_path");
    let source = r#"
use io
use fs

task main
    try fs.mkdir("test_dir_path")
    if fs.exists("test_dir_path")
        io.show("directory exists")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "directory exists");
    let _ = std::fs::remove_dir("test_dir_path");
}

#[test]
fn test_fs_list() {
    let _ = std::fs::remove_file("test_list_dir/file1.txt");
    let _ = std::fs::remove_file("test_list_dir/file2.txt");
    let _ = std::fs::remove_dir("test_list_dir");
    let source = r#"
use io
use fs

task main
    try fs.mkdir("test_list_dir")
    try fs.write("test_list_dir/file1.txt", "1")
    try fs.write("test_list_dir/file2.txt", "2")
    files = try fs.list("test_list_dir")
    for file in files
        io.show(file)
    try fs.delete("test_list_dir/file1.txt")
    try fs.delete("test_list_dir/file2.txt")
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0, "Compilation/Execution failed. Output was:\n{}", out);
    let _ = std::fs::remove_dir("test_list_dir");
    let mut sorted_lines: Vec<&str> = out.lines().collect();
    sorted_lines.sort();
    assert_eq!(sorted_lines, vec!["file1.txt", "file2.txt"]);
}

#[test]
fn test_fs_error_handling() {
    let source = r#"
use io
use fs

task main
    result = fs.read("non_existent_file_xyz.txt")
    if result.is_err
        io.show(result.error)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0);
    assert!(out.contains("file not found") || out.contains("could not"));
}

#[test]
fn test_io_stdin_read() {
    let source = r#"
use io

task main
    line = io.read()
    io.show("stdin read: " + line)
"#;
    let (out, code) = compile_and_run_with_stdin(source, "hello stdin input\n");
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "stdin read: hello stdin input");
}

#[test]
fn test_json_encode_map() {
    let source = r#"
use io
use json

task main
    data = {"name": "n0ne", "version": "0.1"}
    result = json.encode(data)
    io.show(result)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0, "Compilation/Execution failed. Output was:\n{}", out);
    let trimmed = out.trim();
    assert!(trimmed.contains("\"name\""), "expected name key, got: {}", trimmed);
    assert!(trimmed.contains("\"n0ne\""), "expected n0ne value, got: {}", trimmed);
    assert!(trimmed.contains("\"version\""), "expected version key, got: {}", trimmed);
    assert!(trimmed.contains("\"0.1\""), "expected 0.1 value, got: {}", trimmed);
}

#[test]
fn test_json_decode() {
    let source = r#"
use io
use json

task main
    data = try json.decode("{\"name\": \"n0ne\", \"lang\": \"rust\"}")
    if data.has("name")
        io.show("has name")
    if data.has("lang")
        io.show("has lang")
    keys = data.keys()
    io.show(keys.len().to_string())
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0, "Compilation/Execution failed. Output was:\n{}", out);
    let lines: Vec<&str> = out.trim().lines().collect();
    assert!(lines.contains(&"has name"), "expected 'has name', got: {:?}", lines);
    assert!(lines.contains(&"has lang"), "expected 'has lang', got: {:?}", lines);
    assert!(lines.contains(&"2"), "expected key count '2', got: {:?}", lines);
}

#[test]
fn test_json_encode_string() {
    let source = r#"
use io
use json

task main
    result = json.encode("hello world")
    io.show(result)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0, "Compilation/Execution failed. Output was:\n{}", out);
    assert_eq!(out.trim(), "\"hello world\"");
}

#[test]
fn test_new_syntax_features() {
    let source = r#"
fn get_result(ok_val: int) -> result[int]
    res = c_alloc(32)
    c_store_int(res, 0, 1)
    if ok_val == 1
        c_store_int(res, 8, 0)
        c_store_int(res, 16, 42)
    else
        c_store_int(res, 8, 1)
        c_store_string(res, 24, "some error")
    return res

task main
    print("hello print")
    res1 = get_result(1)
    if res1.is_ok
        print(res1.unwrap())
    res2 = get_result(0)
    if res2.is_err
        print(res2.error)
"#;
    let (out, code) = compile_and_run(source);
    assert_eq!(code, 0, "Compilation/Execution failed. Output was:\n{}", out);
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "hello print");
    assert_eq!(lines[1], "42");
    assert_eq!(lines[2], "some error");
}

