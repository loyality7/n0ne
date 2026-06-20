#[cfg(test)]
mod tests {
    use crate::Parser;
    use ast::*;
    use lexer::Lexer;

    #[test]
    fn test_parse_fn() {
        let input = "fn greet(name: string) -> string\n    return name\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::FnDecl(FnDecl {
                name: "greet".to_string(),
                receiver: None,
                params: vec![Param {
                    name: "name".to_string(),
                    type_ann: Type::Basic("string".to_string())
                }],
                return_type: Some(Type::Basic("string".to_string())),
                body: Block {
                    stmts: vec![Stmt::Return(Some(Expr::Ident("name".to_string())))]
                }
            })]
        );
    }

    #[test]
    fn test_parse_method() {
        let input = "fn (self: User) greet(name: string)\n    show(name)\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::FnDecl(FnDecl {
                name: "greet".to_string(),
                receiver: Some(Receiver {
                    name: "self".to_string(),
                    type_name: "User".to_string()
                }),
                params: vec![Param {
                    name: "name".to_string(),
                    type_ann: Type::Basic("string".to_string())
                }],
                return_type: None,
                body: Block {
                    stmts: vec![Stmt::Expr(Expr::CallExpr {
                        callee: Box::new(Expr::Ident("show".to_string())),
                        args: vec![Expr::Ident("name".to_string())]
                    })]
                }
            })]
        );
    }

    #[test]
    fn test_parse_type() {
        let input = "type User\n    name: string\n    age: int\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TypeDecl(TypeDecl {
                name: "User".to_string(),
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        type_ann: Type::Basic("string".to_string())
                    },
                    Field {
                        name: "age".to_string(),
                        type_ann: Type::Basic("int".to_string())
                    }
                ]
            })]
        );
    }

    #[test]
    fn test_parse_task() {
        let input = "task main\n    run()\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "main".to_string(),
                body: Block {
                    stmts: vec![Stmt::Expr(Expr::CallExpr {
                        callee: Box::new(Expr::Ident("run".to_string())),
                        args: vec![]
                    })]
                }
            })]
        );
    }

    #[test]
    fn test_parse_if_else_real() {
        // Test parsing an if/else block inside a task
        let input = "task check\n    if true\n        show()\n    elif false\n        hide()\n    else\n        ignore()\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "check".to_string(),
                body: Block {
                    stmts: vec![Stmt::If {
                        cond: Expr::Literal(Literal::Bool(true)),
                        then_branch: Block {
                            stmts: vec![Stmt::Expr(Expr::CallExpr {
                                callee: Box::new(Expr::Ident("show".to_string())),
                                args: vec![]
                            })]
                        },
                        elifs: vec![(
                            Expr::Literal(Literal::Bool(false)),
                            Block {
                                stmts: vec![Stmt::Expr(Expr::CallExpr {
                                    callee: Box::new(Expr::Ident("hide".to_string())),
                                    args: vec![]
                                })]
                            }
                        )],
                        else_branch: Some(Block {
                            stmts: vec![Stmt::Expr(Expr::CallExpr {
                                callee: Box::new(Expr::Ident("ignore".to_string())),
                                args: vec![]
                            })]
                        })
                    }]
                }
            })]
        );
    }

    #[test]
    fn test_parse_for() {
        let input = "task loop\n    for x in items\n        show(x)\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "loop".to_string(),
                body: Block {
                    stmts: vec![Stmt::For {
                        var: "x".to_string(),
                        iterable: Expr::Ident("items".to_string()),
                        body: Block {
                            stmts: vec![Stmt::Expr(Expr::CallExpr {
                                callee: Box::new(Expr::Ident("show".to_string())),
                                args: vec![Expr::Ident("x".to_string())]
                            })]
                        }
                    }]
                }
            })]
        );
    }

    #[test]
    fn test_parse_precedence() {
        let input = "task prec\n    x = a + b * c ** d\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "prec".to_string(),
                body: Block {
                    stmts: vec![Stmt::Assign {
                        target: Expr::Ident("x".to_string()),
                        op: AssignOp::Eq,
                        value: Expr::BinExpr {
                            left: Box::new(Expr::Ident("a".to_string())),
                            op: BinOp::Add,
                            right: Box::new(Expr::BinExpr {
                                left: Box::new(Expr::Ident("b".to_string())),
                                op: BinOp::Mul,
                                right: Box::new(Expr::BinExpr {
                                    left: Box::new(Expr::Ident("c".to_string())),
                                    op: BinOp::Pow,
                                    right: Box::new(Expr::Ident("d".to_string()))
                                })
                            })
                        }
                    }]
                }
            })]
        );
    }

    #[test]
    fn test_parse_try() {
        let input = "task attempt\n    val = try risky()\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "attempt".to_string(),
                body: Block {
                    stmts: vec![Stmt::Assign {
                        target: Expr::Ident("val".to_string()),
                        op: AssignOp::Eq,
                        value: Expr::TryExpr(Box::new(Expr::CallExpr {
                            callee: Box::new(Expr::Ident("risky".to_string())),
                            args: vec![]
                        }))
                    }]
                }
            })]
        );
    }

    #[test]
    fn test_parse_use() {
        let input = "use http\nuse ./mymodule\nuse pkg/name\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![
                TopLevelDecl::UseDecl(UseDecl {
                    path: "http".to_string()
                }),
                TopLevelDecl::UseDecl(UseDecl {
                    path: "./mymodule".to_string()
                }),
                TopLevelDecl::UseDecl(UseDecl {
                    path: "pkg/name".to_string()
                }),
            ]
        );
    }

    #[test]
    fn test_parse_fstring() {
        let input = "task main\n    msg = f\"hello {name} you are {21} old\"\n";
        let tokens = Lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        let program = parser.parse();

        assert_eq!(
            program.decls,
            vec![TopLevelDecl::TaskDecl(TaskDecl {
                name: "main".to_string(),
                body: Block {
                    stmts: vec![Stmt::Assign {
                        target: Expr::Ident("msg".to_string()),
                        op: AssignOp::Eq,
                        value: Expr::FStringExpr(vec![
                            FStringPart::Text("hello ".to_string()),
                            FStringPart::Expr(Expr::Ident("name".to_string())),
                            FStringPart::Text(" you are ".to_string()),
                            FStringPart::Expr(Expr::Literal(Literal::Int(21))),
                            FStringPart::Text(" old".to_string()),
                        ])
                    }]
                }
            })]
        );
    }
}
