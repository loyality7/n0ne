use lexer::Lexer;
use lexer::token::TokenKind;

fn test_lex(input: &str, expected: &[TokenKind]) {
    let tokens = Lexer::tokenize(input);
    let kinds: Vec<TokenKind> = tokens
        .into_iter()
        .map(|t| t.kind)
        .filter(|k| !matches!(k, TokenKind::Newline | TokenKind::Eof))
        .collect();
    assert_eq!(kinds, expected);
}

fn test_lex_error(input: &str, expected_panic_msg: &str) {
    let result = std::panic::catch_unwind(|| {
        Lexer::tokenize(input);
    });
    assert!(result.is_err(), "Expected panic for input '{}'", input);
    let err = result.unwrap_err();
    let msg = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        String::new()
    };
    assert!(
        msg.to_lowercase().contains(&expected_panic_msg.to_lowercase()),
        "Expected panic containing '{}', but got '{}'",
        expected_panic_msg,
        msg
    );
}

#[test]
fn test_lexer_keywords() {
    test_lex("fn", &[TokenKind::Fn]);
    test_lex("task", &[TokenKind::Task]);
    test_lex("if", &[TokenKind::If]);
    test_lex("elif", &[TokenKind::Elif]);
    test_lex("else", &[TokenKind::Else]);
    test_lex("for", &[TokenKind::For]);
    test_lex("in", &[TokenKind::In]);
    test_lex("while", &[TokenKind::While]);
    test_lex("break", &[TokenKind::Break]);
    test_lex("continue", &[TokenKind::Continue]);
    test_lex("return", &[TokenKind::Return]);
    test_lex("type", &[TokenKind::Type]);
    test_lex("use", &[TokenKind::Use]);
    test_lex("const", &[TokenKind::Const]);
    test_lex("match", &[TokenKind::Match]);
    test_lex("and", &[TokenKind::And]);
    test_lex("or", &[TokenKind::Or]);
    test_lex("not", &[TokenKind::Not]);
    test_lex("try", &[TokenKind::Try]);
    test_lex("true", &[TokenKind::Bool(true)]);
    test_lex("false", &[TokenKind::Bool(false)]);
}

#[test]
fn test_lexer_literals() {
    test_lex("42", &[TokenKind::Number(42)]);
    test_lex("3.14", &[TokenKind::Float(3.14)]);
    test_lex("\"hello\"", &[TokenKind::String("hello".to_string())]);
    test_lex("\"\"", &[TokenKind::String("".to_string())]);
    test_lex("\"hello\\nworld\"", &[TokenKind::String("hello\nworld".to_string())]);
}

#[test]
fn test_lexer_operators() {
    test_lex("+", &[TokenKind::Plus]);
    test_lex("-", &[TokenKind::Minus]);
    test_lex("*", &[TokenKind::Star]);
    test_lex("/", &[TokenKind::Slash]);
    test_lex("%", &[TokenKind::Percent]);
    test_lex("**", &[TokenKind::Power]);
    test_lex("==", &[TokenKind::EqEq]);
    test_lex("!=", &[TokenKind::NotEq]);
    test_lex("<=", &[TokenKind::LtEq]);
    test_lex(">=", &[TokenKind::GtEq]);
    test_lex("->", &[TokenKind::Arrow]);
}

#[test]
fn test_lexer_indent_dedent() {
    let tokens = Lexer::tokenize("fn main()\n    show()\n");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Ident("show".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_lexer_two_level_indent() {
    let tokens = Lexer::tokenize("fn f()\n    if true\n        show()\n");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Ident("f".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::If,
            TokenKind::Bool(true),
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Ident("show".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Dedent,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_lexer_comments_stripped() {
    test_lex("x = 5 # comment\n", &[
        TokenKind::Ident("x".to_string()),
        TokenKind::Eq,
        TokenKind::Number(5)
    ]);
    test_lex("# full line comment\nx = 1\n", &[
        TokenKind::Ident("x".to_string()),
        TokenKind::Eq,
        TokenKind::Number(1)
    ]);
}

#[test]
fn test_lexer_errors() {
    test_lex_error("@invalid", "unexpected character");
    test_lex_error("\t", "tab character is not allowed");
    test_lex_error("   x", "indentation must be a multiple of 4");
}
