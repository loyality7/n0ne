#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenKind};
    use crate::lexer::Lexer;

    #[test]
    fn test_identifier() {
        let input = "foo bar_123 _temp";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Ident("foo".to_string()), line: 1, column: 1 },
                Token { kind: TokenKind::Ident("bar_123".to_string()), line: 1, column: 5 },
                Token { kind: TokenKind::Ident("_temp".to_string()), line: 1, column: 13 },
                Token { kind: TokenKind::Newline, line: 1, column: 18 },
                Token { kind: TokenKind::Eof, line: 1, column: 18 },
            ]
        );
    }

    #[test]
    fn test_keyword() {
        let input = "fn if else elif for in type task return use const priv extern export unsafe weak and or not try spawn true false";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Fn, line: 1, column: 1 },
                Token { kind: TokenKind::If, line: 1, column: 4 },
                Token { kind: TokenKind::Else, line: 1, column: 7 },
                Token { kind: TokenKind::Elif, line: 1, column: 12 },
                Token { kind: TokenKind::For, line: 1, column: 17 },
                Token { kind: TokenKind::In, line: 1, column: 21 },
                Token { kind: TokenKind::Type, line: 1, column: 24 },
                Token { kind: TokenKind::Task, line: 1, column: 29 },
                Token { kind: TokenKind::Return, line: 1, column: 34 },
                Token { kind: TokenKind::Use, line: 1, column: 41 },
                Token { kind: TokenKind::Const, line: 1, column: 45 },
                Token { kind: TokenKind::Priv, line: 1, column: 51 },
                Token { kind: TokenKind::Extern, line: 1, column: 56 },
                Token { kind: TokenKind::Export, line: 1, column: 63 },
                Token { kind: TokenKind::Unsafe, line: 1, column: 70 },
                Token { kind: TokenKind::Weak, line: 1, column: 77 },
                Token { kind: TokenKind::And, line: 1, column: 82 },
                Token { kind: TokenKind::Or, line: 1, column: 86 },
                Token { kind: TokenKind::Not, line: 1, column: 89 },
                Token { kind: TokenKind::Try, line: 1, column: 93 },
                Token { kind: TokenKind::Spawn, line: 1, column: 97 },
                Token { kind: TokenKind::Bool(true), line: 1, column: 103 },
                Token { kind: TokenKind::Bool(false), line: 1, column: 108 },
                Token { kind: TokenKind::Newline, line: 1, column: 113 },
                Token { kind: TokenKind::Eof, line: 1, column: 113 },
            ]
        );
    }

    #[test]
    fn test_number() {
        let input = "42 3.14 0 99999.888";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Number(42), line: 1, column: 1 },
                Token { kind: TokenKind::Float(3.14), line: 1, column: 4 },
                Token { kind: TokenKind::Number(0), line: 1, column: 9 },
                Token { kind: TokenKind::Float(99999.888), line: 1, column: 11 },
                Token { kind: TokenKind::Newline, line: 1, column: 20 },
                Token { kind: TokenKind::Eof, line: 1, column: 20 },
            ]
        );
    }

    #[test]
    fn test_number_dot_property() {
        let input = "3.foo";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Number(3), line: 1, column: 1 },
                Token { kind: TokenKind::Dot, line: 1, column: 2 },
                Token { kind: TokenKind::Ident("foo".to_string()), line: 1, column: 3 },
                Token { kind: TokenKind::Newline, line: 1, column: 6 },
                Token { kind: TokenKind::Eof, line: 1, column: 6 },
            ]
        );
    }

    #[test]
    fn test_string() {
        let input = r#""hello" "nested \" quote" "line 1\nline 2""#;
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::String("hello".to_string()), line: 1, column: 1 },
                Token { kind: TokenKind::String("nested \" quote".to_string()), line: 1, column: 9 },
                Token { kind: TokenKind::String("line 1\nline 2".to_string()), line: 1, column: 27 },
                Token { kind: TokenKind::Newline, line: 1, column: 43 },
                Token { kind: TokenKind::Eof, line: 1, column: 43 },
            ]
        );
    }

    #[test]
    fn test_indent_dedent() {
        let input = "fn greet(name)\n    show(name)\n";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Fn, line: 1, column: 1 },
                Token { kind: TokenKind::Ident("greet".to_string()), line: 1, column: 4 },
                Token { kind: TokenKind::LParen, line: 1, column: 9 },
                Token { kind: TokenKind::Ident("name".to_string()), line: 1, column: 10 },
                Token { kind: TokenKind::RParen, line: 1, column: 14 },
                Token { kind: TokenKind::Newline, line: 1, column: 15 },
                Token { kind: TokenKind::Indent, line: 2, column: 1 },
                Token { kind: TokenKind::Ident("show".to_string()), line: 2, column: 5 },
                Token { kind: TokenKind::LParen, line: 2, column: 9 },
                Token { kind: TokenKind::Ident("name".to_string()), line: 2, column: 10 },
                Token { kind: TokenKind::RParen, line: 2, column: 14 },
                Token { kind: TokenKind::Newline, line: 2, column: 15 },
                Token { kind: TokenKind::Dedent, line: 3, column: 1 },
                Token { kind: TokenKind::Eof, line: 3, column: 1 },
            ]
        );
    }

    #[test]
    fn test_comment_stripping() {
        let input = "# full line comment\nfn foo() # inline comment\n    # indent comment\n    return 1";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Fn, line: 2, column: 1 },
                Token { kind: TokenKind::Ident("foo".to_string()), line: 2, column: 4 },
                Token { kind: TokenKind::LParen, line: 2, column: 7 },
                Token { kind: TokenKind::RParen, line: 2, column: 8 },
                Token { kind: TokenKind::Newline, line: 2, column: 26 },
                Token { kind: TokenKind::Indent, line: 4, column: 1 },
                Token { kind: TokenKind::Return, line: 4, column: 5 },
                Token { kind: TokenKind::Number(1), line: 4, column: 12 },
                Token { kind: TokenKind::Newline, line: 4, column: 13 },
                Token { kind: TokenKind::Dedent, line: 4, column: 13 },
                Token { kind: TokenKind::Eof, line: 4, column: 13 },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Tab character is not allowed")]
    fn test_error_on_tab() {
        let input = "fn foo()\n\treturn 1";
        Lexer::tokenize(input);
    }

    #[test]
    #[should_panic(expected = "Indentation must be a multiple of 4 spaces")]
    fn test_error_on_invalid_indent() {
        let input = "fn foo()\n  return 1";
        Lexer::tokenize(input);
    }

    #[test]
    #[should_panic(expected = "Indentation does not match any outer level")]
    fn test_error_on_misaligned_dedent() {
        let input = "fn foo()\n    if true\n            bar()\n        baz()";
        Lexer::tokenize(input);
    }

    #[test]
    fn test_operators_delimiters() {
        let input = "+ - * / % ** == != < > <= >= = += -= *= /= ( ) [ ] . , : ->";
        let tokens = Lexer::tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token { kind: TokenKind::Plus, line: 1, column: 1 },
                Token { kind: TokenKind::Minus, line: 1, column: 3 },
                Token { kind: TokenKind::Star, line: 1, column: 5 },
                Token { kind: TokenKind::Slash, line: 1, column: 7 },
                Token { kind: TokenKind::Percent, line: 1, column: 9 },
                Token { kind: TokenKind::Power, line: 1, column: 11 },
                Token { kind: TokenKind::EqEq, line: 1, column: 14 },
                Token { kind: TokenKind::NotEq, line: 1, column: 17 },
                Token { kind: TokenKind::Lt, line: 1, column: 20 },
                Token { kind: TokenKind::Gt, line: 1, column: 22 },
                Token { kind: TokenKind::LtEq, line: 1, column: 24 },
                Token { kind: TokenKind::GtEq, line: 1, column: 27 },
                Token { kind: TokenKind::Eq, line: 1, column: 30 },
                Token { kind: TokenKind::PlusEq, line: 1, column: 32 },
                Token { kind: TokenKind::MinusEq, line: 1, column: 35 },
                Token { kind: TokenKind::StarEq, line: 1, column: 38 },
                Token { kind: TokenKind::SlashEq, line: 1, column: 41 },
                Token { kind: TokenKind::LParen, line: 1, column: 44 },
                Token { kind: TokenKind::RParen, line: 1, column: 46 },
                Token { kind: TokenKind::LBracket, line: 1, column: 48 },
                Token { kind: TokenKind::RBracket, line: 1, column: 50 },
                Token { kind: TokenKind::Dot, line: 1, column: 52 },
                Token { kind: TokenKind::Comma, line: 1, column: 54 },
                Token { kind: TokenKind::Colon, line: 1, column: 56 },
                Token { kind: TokenKind::Arrow, line: 1, column: 58 },
                Token { kind: TokenKind::Newline, line: 1, column: 60 },
                Token { kind: TokenKind::Eof, line: 1, column: 60 },
            ]
        );
    }

    #[test]
    fn test_fstring_lexer() {
        let input = "msg = f\"hello {name} you are {age} years old\"";
        let tokens = Lexer::tokenize(input);
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FStringStart)));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FStringText(_))));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FStringExprStart)));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FStringExprEnd)));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FStringEnd)));
    }

    #[test]
    #[should_panic(expected = "Nested string literal inside f-string expression is not allowed")]
    fn test_nested_string_error() {
        let input = "msg = f\"hello {\"world\"}\"";
        Lexer::tokenize(input);
    }
}
