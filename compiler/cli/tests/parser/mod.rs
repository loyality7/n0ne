use lexer::Lexer;
use parser::Parser;
use ast::*;

fn parse(input: &str) -> Program {
    let tokens = Lexer::tokenize(input);
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn assert_parse_error(input: &str, expected_msg: &str) {
    let result = std::panic::catch_unwind(|| {
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        parser.parse();
    });
    assert!(result.is_err(), "Expected parser to panic for input: '{}'", input);
    let err = result.unwrap_err();
    let msg = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        String::new()
    };
    assert!(
        msg.to_lowercase().contains(&expected_msg.to_lowercase()),
        "Expected panic containing '{}', but got '{}'",
        expected_msg,
        msg
    );
}

#[test]
fn test_parser_assign() {
    let prog = parse("task main\n    x = 42\n");
    assert_eq!(
        prog.decls.is_empty(),
        false
    );
}

#[test]
fn test_parser_fn_decl() {
    let prog = parse("fn f()\n    return\n");
    if let TopLevelDecl::FnDecl(ref fd) = prog.decls[0] {
        assert_eq!(fd.name, "f");
        assert!(fd.params.is_empty());
        assert!(fd.return_type.is_none());
    } else {
        panic!("Expected FnDecl");
    }

    let prog2 = parse("fn add(a: int, b: int) -> int\n    return a + b\n");
    if let TopLevelDecl::FnDecl(ref fd) = prog2.decls[0] {
        assert_eq!(fd.name, "add");
        assert_eq!(fd.params.len(), 2);
        assert_eq!(fd.params[0].name, "a");
        assert_eq!(fd.params[1].name, "b");
        assert!(matches!(fd.return_type, Some(Type::Basic(_))));
    } else {
        panic!("Expected FnDecl");
    }
}

#[test]
fn test_parser_task_decl() {
    let prog = parse("task main\n    show(\"hi\")\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert_eq!(td.name, "main");
        assert_eq!(td.body.stmts.len(), 1);
    } else {
        panic!("Expected TaskDecl");
    }
}

#[test]
fn test_parser_type_decl() {
    let prog = parse("type User\n    name: string\n    age: int\n");
    if let TopLevelDecl::TypeDecl(ref td) = prog.decls[0] {
        assert_eq!(td.name, "User");
        assert_eq!(td.fields.len(), 2);
        assert_eq!(td.fields[0].name, "name");
        assert_eq!(td.fields[1].name, "age");
    } else {
        panic!("Expected TypeDecl");
    }
}

#[test]
fn test_parser_method_decl() {
    let prog = parse("fn (self: User) greet()\n    show(self.name)\n");
    if let TopLevelDecl::FnDecl(ref fd) = prog.decls[0] {
        assert_eq!(fd.name, "greet");
        assert!(fd.receiver.is_some());
        assert_eq!(fd.receiver.as_ref().unwrap().type_name, "User");
    } else {
        panic!("Expected MethodDecl");
    }
}

#[test]
fn test_parser_if_stmt() {
    let prog = parse("task t\n    if x > 0\n        show(\"pos\")\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::If { .. }));
    } else {
        panic!("Expected TaskDecl");
    }
}

#[test]
fn test_parser_while_stmt() {
    let prog = parse("task t\n    while x > 0\n        x = x - 1\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::While { .. }));
    }
}

#[test]
fn test_parser_for_stmt() {
    let prog = parse("task t\n    for item in items\n        show(item)\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::For { .. }));
    }
}

#[test]
fn test_parser_break_continue() {
    let prog = parse("task t\n    while true\n        break\n        continue\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::While { .. }));
    }
}

#[test]
fn test_parser_match_stmt() {
    let prog = parse("task t\n    match x\n        1 -> show(\"one\")\n        _ -> show(\"other\")\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::Match { .. }));
    }
}

#[test]
fn test_parser_const_decl() {
    let prog = parse("const MAX = 100\n");
    if let TopLevelDecl::ConstDecl(ref cd) = prog.decls[0] {
        assert_eq!(cd.name, "MAX");
        assert!(matches!(cd.value, Expr::Literal(Literal::Int(100))));
    } else {
        panic!("Expected ConstDecl");
    }
}

#[test]
fn test_parser_precedence() {
    let prog = parse("task t\n    x = 1 + 2 * 3\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        if let Stmt::Assign { ref value, .. } = td.body.stmts[0] {
            if let Expr::BinExpr { ref op, ref right, .. } = value {
                assert_eq!(*op, BinOp::Add);
                assert!(matches!(**right, Expr::BinExpr { .. }));
            }
        }
    }
}

#[test]
fn test_parser_field_access_chain() {
    let prog = parse("task t\n    x = user.address.city\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        if let Stmt::Assign { ref value, .. } = td.body.stmts[0] {
            assert!(matches!(value, Expr::FieldAccess { .. }));
        }
    }
}

#[test]
fn test_parser_method_call() {
    let prog = parse("task t\n    user.greet()\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        assert!(matches!(td.body.stmts[0], Stmt::Expr(Expr::CallExpr { .. })));
    }
}

#[test]
fn test_parser_fstring() {
    let prog = parse("task t\n    x = f\"hello {name}\"\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        if let Stmt::Assign { ref value, .. } = td.body.stmts[0] {
            assert!(matches!(value, Expr::FStringExpr(..)));
        }
    }
}

#[test]
fn test_parser_multiline_string() {
    let prog = parse("task t\n    x = \"hello\\nworld\"\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        if let Stmt::Assign { ref value, .. } = td.body.stmts[0] {
            assert!(matches!(value, Expr::Literal(Literal::String(..))));
        }
    }
}

#[test]
fn test_parser_try_result() {
    let prog = parse("task t\n    x = try risky()\n");
    if let TopLevelDecl::TaskDecl(ref td) = prog.decls[0] {
        if let Stmt::Assign { ref value, .. } = td.body.stmts[0] {
            assert!(matches!(value, Expr::TryExpr(..)));
        }
    }
}

#[test]
fn test_parser_errors() {
    assert_parse_error("fn ()\n    return\n", "expected");
    assert_parse_error("task t\n    if\n        show()\n", "expected prefix expression");
}
