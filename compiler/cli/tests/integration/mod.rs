macro_rules! test {
    ($name:ident, source: $src:expr, stdout: $out:expr, exit: $code:expr) => {
        #[test]
        fn $name() {
            let res = crate::helpers::run_n0ne($src);
            assert_eq!(res.exit_code, $code, "Exit code mismatch. Stderr:\n{}", res.stderr);
            assert_eq!(res.stdout.replace("\r\n", "\n"), $out.replace("\r\n", "\n"));
        }
    };
    ($name:ident, source: $src:expr, stdout: $out:expr) => {
        test!($name, source: $src, stdout: $out, exit: 0);
    };
    ($name:ident, $src:expr, $out:expr) => {
        #[test]
        fn $name() {
            let mut indented = String::new();
            for line in $src.lines() {
                if !line.trim().is_empty() {
                    indented.push_str("    ");
                    indented.push_str(line);
                }
                indented.push_str("\n");
            }
            let wrapped = format!("task main\n{}", indented);
            let res = crate::helpers::run_n0ne(&wrapped);
            assert_eq!(res.exit_code, 0, "Exit code mismatch. Stderr:\n{}", res.stderr);
            assert_eq!(res.stdout.replace("\r\n", "\n"), $out.replace("\r\n", "\n"));
        }
    };
}

macro_rules! compile_error_test {
    ($name:ident, source: $src:expr, $(contains: $msg:expr),+ $(,)?) => {
        #[test]
        fn $name() {
            match crate::helpers::compile_n0ne($src) {
                Ok(_) => panic!("Expected compilation to fail, but it succeeded"),
                Err(errs) => {
                    let combined = errs.join("\n");
                    $(
                        assert!(
                            combined.contains($msg),
                            "Expected error to contain '{}', but got:\n{}",
                            $msg,
                            combined
                        );
                    )+
                }
            }
        }
    };
}

// SECTION 1 — BASICS
test!(hello_world,
    source: "task main\n    show(\"hello world\")\n",
    stdout: "hello world\n",
    exit: 0
);

test!(show_int,
    source: "task main\n    show(42.to_string())\n",
    stdout: "42\n",
    exit: 0
);

test!(show_float,
    source: "task main\n    show(3.14159.to_string())\n",
    stdout: "3.14159\n",
    exit: 0
);

test!(show_bool,
    source: "task main\n    show(true.to_string())\n",
    stdout: "true\n",
    exit: 0
);

test!(const_value,
    source: "const MAX = 100\ntask main\n    show(MAX.to_string())\n",
    stdout: "100\n",
    exit: 0
);

// SECTION 2 — ARITHMETIC
test!(add, "show((10 + 5).to_string())", "15\n");
test!(subtract, "show((10 - 3).to_string())", "7\n");
test!(multiply, "show((4 * 5).to_string())", "20\n");
test!(divide, "show((10 / 2).to_string())", "5\n");
test!(modulo, "show((10 % 3).to_string())", "1\n");
test!(power, "show((2 ** 8).to_string())", "256\n");
test!(negative, "show((-5).to_string())", "-5\n");
test!(operator_precedence, "show((2 + 3 * 4).to_string())", "14\n");
test!(parens, "show(((2 + 3) * 4).to_string())", "20\n");

// SECTION 3 — STRINGS
test!(string_len, "show(\"hello\".len().to_string())", "5\n");
test!(string_upper, "show(\"hello\".upper())", "HELLO\n");
test!(string_lower, "show(\"HELLO\".lower())", "hello\n");
test!(string_trim, "show(\"  hi  \".trim())", "hi\n");
test!(string_contains_true, "if \"hello\".contains(\"ell\")\n    show(\"yes\")", "yes\n");
test!(string_contains_false, "if \"hello\".contains(\"xyz\")\n    show(\"yes\")\nelse\n    show(\"no\")", "no\n");
test!(string_starts_with, "if \"hello\".starts_with(\"hel\")\n    show(\"yes\")", "yes\n");
test!(string_ends_with, "if \"hello\".ends_with(\"llo\")\n    show(\"yes\")", "yes\n");
test!(string_replace, "show(\"hello\".replace(\"l\", \"r\"))", "herro\n");
test!(string_concat, "show(\"hello\" + \" \" + \"world\")", "hello world\n");
test!(empty_string, "show(\"\")", "\n");
test!(string_split,
    source: "parts = \"a,b,c\".split(\",\")\nfor p in parts\n    show(p)",
    stdout: "a\nb\nc\n"
);
test!(fstring_basic, "name = \"sarath\"\nshow(f\"hello {name}\")", "hello sarath\n");
test!(fstring_int, "n = 42\nshow(f\"value: {n}\")", "value: 42\n");
test!(fstring_expr, "show(f\"result: {1 + 2}\")", "result: 3\n");
test!(multiline_string,
    source: "task main\n    text = \"\"\"\nhello\nworld\n\"\"\"\n    show(text)",
    stdout: "hello\nworld\n"
);

// SECTION 4 — CONTROL FLOW
test!(if_true, "if true\n    show(\"yes\")", "yes\n");
test!(if_false, "if false\n    show(\"yes\")\nelse\n    show(\"no\")", "no\n");
test!(if_elif_else,
    source: "x = 0\nif x > 0\n    show(\"pos\")\nelif x < 0\n    show(\"neg\")\nelse\n    show(\"zero\")",
    stdout: "zero\n"
);
test!(while_basic,
    source: "x = 3\nwhile x > 0\n    show(x.to_string())\n    x = x - 1",
    stdout: "3\n2\n1\n"
);
test!(while_break,
    source: "x = 0\nwhile true\n    x = x + 1\n    if x == 3\n        break\nshow(x.to_string())",
    stdout: "3\n"
);
test!(while_continue,
    source: "x = 0\nwhile x < 5\n    x = x + 1\n    if x == 3\n        continue\n    show(x.to_string())",
    stdout: "1\n2\n4\n5\n"
);
test!(for_list,
    source: "for i in [1, 2, 3]\n    show(i.to_string())",
    stdout: "1\n2\n3\n"
);
test!(for_break,
    source: "for i in [1,2,3,4,5]\n    if i == 3\n        break\n    show(i.to_string())",
    stdout: "1\n2\n"
);
test!(for_continue,
    source: "for i in [1,2,3,4,5]\n    if i == 3\n        continue\n    show(i.to_string())",
    stdout: "1\n2\n4\n5\n"
);
test!(for_empty_list,
    source: "for i in []\n    show(i.to_string())\nshow(\"done\")",
    stdout: "done\n"
);

// SECTION 5 — MATCH
test!(match_basic,
    source: "x = 2\nmatch x\n    1 -> show(\"one\")\n    2 -> show(\"two\")\n    _ -> show(\"other\")",
    stdout: "two\n"
);
test!(match_string,
    source: "s = \"hello\"\nmatch s\n    \"hello\" -> show(\"hi\")\n    \"bye\"   -> show(\"goodbye\")\n    _       -> show(\"unknown\")",
    stdout: "hi\n"
);
test!(match_default,
    source: "x = 99\nmatch x\n    1 -> show(\"one\")\n    _ -> show(\"other\")",
    stdout: "other\n"
);
test!(match_no_default_no_match,
    source: "x = 5\nmatch x\n    1 -> show(\"one\")\n    2 -> show(\"two\")",
    stdout: ""
);

// SECTION 6 — FUNCTIONS
test!(fn_basic,
    source: "fn greet(name: string)\n    show(f\"hello {name}\")\ntask main\n    greet(\"sarath\")",
    stdout: "hello sarath\n"
);
test!(fn_return,
    source: "fn add(a: int, b: int) -> int\n    return a + b\ntask main\n    show(add(3, 4).to_string())",
    stdout: "7\n"
);
test!(fn_multiple_calls,
    source: "fn double(n: int) -> int\n    return n * 2\ntask main\n    show(double(double(3)).to_string())",
    stdout: "12\n"
);
test!(recursion_factorial,
    source: "fn fact(n: int) -> int\n    if n <= 1\n        return 1\n    return n * fact(n - 1)\ntask main\n    show(fact(5).to_string())",
    stdout: "120\n"
);
test!(recursion_fibonacci,
    source: "fn fib(n: int) -> int\n    if n <= 1\n        return n\n    return fib(n-1) + fib(n-2)\ntask main\n    show(fib(10).to_string())",
    stdout: "55\n"
);

// SECTION 7 — TYPES
test!(type_basic,
    source: "type User\n    name: string\n    age: int\ntask main\n    u = User()\n    u.name = \"sarath\"\n    u.age = 21\n    show(u.name)\n    show(u.age.to_string())",
    stdout: "sarath\n21\n"
);
test!(type_method,
    source: "type User\n    name: string\nfn (self: User) greet()\n    show(f\"hi {self.name}\")\ntask main\n    u = User()\n    u.name = \"sarath\"\n    u.greet()",
    stdout: "hi sarath\n"
);
test!(type_method_return,
    source: "type Counter\n    count: int\nfn (self: Counter) inc() -> int\n    return self.count + 1\ntask main\n    c = Counter()\n    c.count = 5\n    show(c.inc().to_string())",
    stdout: "6\n"
);

// SECTION 8 — LISTS
test!(list_literal, "items = [1,2,3]\nfor i in items\n    show(i.to_string())", "1\n2\n3\n");
test!(list_len, "show([1,2,3].len().to_string())", "3\n");
test!(list_push,
    source: "items = [1,2]\nitems.push(3)\nfor i in items\n    show(i.to_string())",
    stdout: "1\n2\n3\n"
);
test!(list_empty, "items = []\nshow(items.len().to_string())", "0\n");
test!(list_contains_true, "if [1,2,3].contains(2)\n    show(\"yes\")", "yes\n");
test!(list_contains_false, "if [1,2,3].contains(9)\n    show(\"yes\")\nelse\n    show(\"no\")", "no\n");
test!(list_first,
    source: "f = [10,20,30].first()\nif f.is_some\n    show(f.unwrap().to_string())",
    stdout: "10\n"
);
test!(list_last,
    source: "l = [10,20,30].last()\nif l.is_some\n    show(l.unwrap().to_string())",
    stdout: "30\n"
);

// SECTION 9 — MAPS
test!(map_basic,
    source: "data = {\"key\": \"value\"}\nv = data.get(\"key\")\nif v.is_some\n    show(v.unwrap())",
    stdout: "value\n"
);
test!(map_set,
    source: "data = {}\ndata.set(\"x\", \"hello\")\nv = data.get(\"x\")\nif v.is_some\n    show(v.unwrap())",
    stdout: "hello\n"
);
test!(map_has_true, "data = {\"a\": 1}\nif data.has(\"a\")\n    show(\"yes\")", "yes\n");
test!(map_has_false, "data = {\"a\": 1}\nif data.has(\"z\")\n    show(\"yes\")\nelse\n    show(\"no\")", "no\n");
test!(map_missing_key,
    source: "data = {\"a\": 1}\nv = data.get(\"z\")\nif v.is_none\n    show(\"missing\")",
    stdout: "missing\n"
);

// SECTION 10 — RESULT AND OPTION
test!(result_ok,
    source: "fn safe(n: int) -> result[int]\n    if n < 0\n        return err(\"negative\")\n    return ok(n * 2)\ntask main\n    r = safe(5)\n    if r.is_ok\n        show(r.unwrap().to_string())",
    stdout: "10\n"
);
test!(result_err,
    source: "fn safe(n: int) -> result[int]\n    if n < 0\n        return err(\"negative\")\n    return ok(n)\ntask main\n    r = safe(-1)\n    if r.is_err\n        show(r.error)",
    stdout: "negative\n"
);
test!(try_propagate,
    source: "fn risky() -> result[int]\n    return err(\"fail\")\nfn run() -> result[int]\n    val = try risky()\n    return ok(val)\ntask main\n    r = run()\n    if r.is_err\n        show(r.error)",
    stdout: "fail\n"
);
test!(option_some,
    source: "fn find(x: int) -> option[int]\n    if x > 0\n        return some(x * 10)\n    return none\ntask main\n    r = find(5)\n    if r.is_some\n        show(r.unwrap().to_string())",
    stdout: "50\n"
);
test!(option_none,
    source: "fn find(x: int) -> option[int]\n    if x > 0\n        return some(x)\n    return none\ntask main\n    r = find(-1)\n    if r.is_none\n        show(\"nothing\")",
    stdout: "nothing\n"
);

// SECTION 11 — EXIT CODES
test!(exit_zero,
    source: "fn main() -> int\n    return 0",
    stdout: "",
    exit: 0
);
test!(exit_nonzero,
    source: "fn main() -> int\n    return 42",
    stdout: "",
    exit: 42
);

// SECTION 12 — ERROR MESSAGES
compile_error_test!(undefined_var,
    source: "task main\n    show(x)\n",
    contains: "E002",
    contains: "undefined variable",
);
compile_error_test!(type_mismatch,
    source: "task main\n    x = 10\n    x = \"hello\"\n",
    contains: "E001",
    contains: "type mismatch",
);
compile_error_test!(break_outside_loop,
    source: "task main\n    break\n",
    contains: "E014",
);
compile_error_test!(continue_outside_loop,
    source: "task main\n    continue\n",
    contains: "E015",
);
compile_error_test!(missing_return,
    source: "fn add(a: int, b: int) -> int\n    show(\"oops\")\n",
    contains: "E006",
);
compile_error_test!(wrong_arg_count,
    source: "fn f(a: int)\n    show(a.to_string())\ntask main\n    f(1, 2)\n",
    contains: "E004",
);
// SECTION 13 — ENUMS
test!(enum_basic,
    source: "enum Color\n    Red\n    Green\n    Blue\n\ntask main\n    c = Red\n    match c\n        Red -> show(\"is red\")\n        Green -> show(\"is green\")\n        Blue -> show(\"is blue\")",
    stdout: "is red\n"
);

test!(enum_with_data,
    source: "enum Message\n    Quit\n    Move(int, int)\n    Write(string)\n\ntask main\n    m = Move(10, 20)\n    match m\n        Quit -> show(\"quit\")\n        Move(x, y) -> show(f\"move {x} {y}\")\n        Write(s) -> show(s)",
    stdout: "move 10 20\n"
);

compile_error_test!(enum_wrong_bindings,
    source: "enum Message\n    Move(int, int)\n\ntask main\n    m = Move(10, 20)\n    match m\n        Move(x) -> show(\"x\")",
    contains: "E004",
    contains: "wrong number of bindings",
);

compile_error_test!(enum_undefined_variant,
    source: "enum Message\n    Quit\n\ntask main\n    m = Quit\n    match m\n        Other -> show(\"other\")",
    contains: "E003",
    contains: "undefined enum variant",
);

// SECTION 14 — MULTIPLE RETURN VALUES & TUPLES
test!(tuple_basic,
    source: "fn get_pair() -> (int, string)\n    return (42, \"hello\")\n\ntask main\n    val, name = get_pair()\n    show(val.to_string())\n    show(name)",
    stdout: "42\nhello\n"
);

test!(tuple_literal,
    source: "task main\n    t = (1, 2)\n    a, b = t\n    show(a.to_string())\n    show(b.to_string())",
    stdout: "1\n2\n"
);

compile_error_test!(tuple_size_mismatch,
    source: "task main\n    a, b = (1, 2, 3)",
    contains: "E001",
    contains: "cannot unpack tuple of size 3 into 2 variables",
);

compile_error_test!(tuple_unpack_non_tuple,
    source: "task main\n    a, b = 42",
    contains: "E001",
    contains: "expected tuple, found",
);

// SECTION 15 — DEFAULT ARGUMENTS
test!(default_args_basic,
    source: "fn greet(name: string = \"world\") -> string\n    return \"hello \" + name\n\ntask main\n    show(greet())\n    show(greet(\"n0ne\"))",
    stdout: "hello world\nhello n0ne\n"
);

test!(default_args_multiple,
    source: "fn add(a: int, b: int = 10, c: int = 20) -> int\n    return a + b + c\n\ntask main\n    show(add(5).to_string())\n    show(add(5, 15).to_string())\n    show(add(5, 15, 25).to_string())",
    stdout: "35\n40\n45\n"
);

compile_error_test!(default_args_missing_required,
    source: "fn add(a: int, b: int = 10) -> int\n    return a + b\n\ntask main\n    add()",
    contains: "E004",
    contains: "wrong argument count for function 'add'",
);

compile_error_test!(default_args_type_mismatch,
    source: "fn greet(name: string = 42)\n    show(name)\ntask main\n    greet()",
    contains: "E001",
    contains: "type mismatch",
);

// SECTION 16 — TYPE ALIASES
test!(type_alias_basic,
    source: "type ID = int\nfn get_id() -> ID\n    return 42\ntask main\n    id = get_id()\n    show(id.to_string())",
    stdout: "42\n"
);

test!(type_alias_complex,
    source: "type Dict = map[string, int]\nfn get_dict() -> Dict\n    m = {\"a\": 1}\n    return m\ntask main\n    d = get_dict()\n    show(d.get(\"a\").unwrap().to_string())",
    stdout: "1\n"
);

test!(
    list_hofs,
    source: "
task main
    nums = [1, 2, 3, 4, 5]
    
    doubled = nums.map(fn(x: int) -> int x * 2)
    show(doubled.len())
    show(doubled.first().unwrap())
    
    evens = nums.filter(fn(x: int) -> bool x % 2 == 0)
    show(evens.len())
    show(evens.first().unwrap())
    
    sum = nums.reduce(0, fn(acc: int, x: int) -> int acc + x)
    show(sum)
    
    found = nums.find(fn(x: int) -> bool x == 3)
    show(found.unwrap())
    
    has_any = nums.any(fn(x: int) -> bool x > 4)
    show(has_any)
    
    all_positive = nums.all(fn(x: int) -> bool x > 0)
    show(all_positive)
",
    stdout: "5\n2\n2\n2\n15\n3\ntrue\ntrue\n"
);

test!(
    pipe_operator,
    source: "
fn add_one(x: int) -> int
    return x + 1

fn double(x: int) -> int
    return x * 2

fn show_val(x: int) -> int
    show(x)
    return x

task main
    res = 5 |> add_one |> double |> show_val
",
    stdout: "12\n"
);

test!(
    defer_basic,
    source: "
fn test_defer() -> int
    defer show(\"deferred 1\")
    defer show(\"deferred 2\")
    show(\"body\")
    return 42

task main
    res = test_defer()
    show(res.to_string())
",
    stdout: "body\ndeferred 2\ndeferred 1\n42\n"
);

test!(
    defer_early_return,
    source: "
fn test_defer_early(x: int) -> int
    defer show(\"deferred\")
    if x > 0
        show(\"early branch\")
        return 1
    show(\"normal branch\")
    return 2

task main
    test_defer_early(1)
    test_defer_early(0)
",
    stdout: "early branch\ndeferred\nnormal branch\ndeferred\n"
);

compile_error_test!(
    defer_must_be_fn_call,
    source: "
task main
    x = 42
    defer x
",
    contains: "E016",
    contains: "defer expression must be a function call",
);
