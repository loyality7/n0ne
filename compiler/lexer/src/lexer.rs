use crate::token::{Token, TokenKind};
use crate::cursor::Cursor;

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn tokenize_single_expression_token(cursor: &mut Cursor, brace_nesting: &mut usize) -> Token {
    let tok_line = cursor.line;
    let tok_col = cursor.column;
    let c = cursor.peek().expect("Lexical error: Expected character inside f-string expression");

    if is_ident_start(c) {
        let mut ident_str = String::new();
        while let Some(cc) = cursor.peek() {
            if is_ident_continue(cc) {
                ident_str.push(cursor.bump().unwrap());
            } else {
                break;
            }
        }
        let kind = match ident_str.as_str() {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            _ => TokenKind::Ident(ident_str),
        };
        Token { kind, line: tok_line, column: tok_col }
    } else if c.is_ascii_digit() {
        let mut num_str = String::new();
        while let Some(cc) = cursor.peek() {
            if cc.is_ascii_digit() {
                num_str.push(cursor.bump().unwrap());
            } else {
                break;
            }
        }
        let is_float = if cursor.peek() == Some('.') {
            if let Some(c2) = cursor.peek_next() {
                c2.is_ascii_digit()
            } else {
                false
            }
        } else {
            false
        };
        if is_float {
            num_str.push(cursor.bump().unwrap()); // consume '.'
            while let Some(cc) = cursor.peek() {
                if cc.is_ascii_digit() {
                    num_str.push(cursor.bump().unwrap());
                } else {
                    break;
                }
            }
            let val = num_str.parse::<f64>().expect("Invalid float literal");
            Token { kind: TokenKind::Float(val), line: tok_line, column: tok_col }
        } else {
            let val = num_str.parse::<i64>().expect("Invalid integer literal");
            Token { kind: TokenKind::Number(val), line: tok_line, column: tok_col }
        }
    } else {
        // Operators and Delimiters
        let kind = match cursor.bump().unwrap() {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => {
                *brace_nesting += 1;
                TokenKind::LBrace
            }
            '}' => {
                if *brace_nesting > 0 {
                    *brace_nesting -= 1;
                }
                TokenKind::RBrace
            }
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '%' => TokenKind::Percent,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => {
                if cursor.peek() == Some('*') {
                    cursor.bump();
                    TokenKind::Power
                } else {
                    TokenKind::Star
                }
            }
            '/' => TokenKind::Slash,
            '=' => {
                if cursor.peek() == Some('=') {
                    cursor.bump();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if cursor.peek() == Some('=') {
                    cursor.bump();
                    TokenKind::NotEq
                } else {
                    panic!("Lexical error: Unexpected character '!' inside f-string expression at line {}, column {}", tok_line, tok_col);
                }
            }
            '<' => {
                if cursor.peek() == Some('=') {
                    cursor.bump();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if cursor.peek() == Some('=') {
                    cursor.bump();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            other => {
                panic!("Lexical error: Unexpected character '{}' inside f-string expression at line {}, column {}", other, tok_line, tok_col);
            }
        };
        Token { kind, line: tok_line, column: tok_col }
    }
}

pub struct Lexer;

impl Lexer {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut cursor = Cursor::new(input);
        let mut tokens = Vec::new();
        let mut indent_stack = vec![0];

        while cursor.peek().is_some() {
            // 1. Scan leading spaces on the line
            let start_line = cursor.line;
            let start_col = cursor.column;
            let mut space_count = 0;

            while let Some(c) = cursor.peek() {
                if c == ' ' {
                    cursor.bump();
                    space_count += 1;
                } else if c == '\t' {
                    panic!(
                        "Lexical error: Tab character is not allowed. (line {}, column {})",
                        cursor.line, cursor.column
                    );
                } else {
                    break;
                }
            }

            let next_char = cursor.peek();

            // Check if line is empty or comment-only
            if next_char.is_none() {
                break;
            }

            let next_c = next_char.unwrap();
            if next_c == '\n' || next_c == '\r' {
                if next_c == '\r' {
                    cursor.bump();
                }
                if cursor.peek() == Some('\n') {
                    cursor.bump();
                }
                continue;
            }

            if next_c == '#' {
                cursor.bump(); // consume '#'
                while let Some(c) = cursor.peek() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    cursor.bump();
                }
                if cursor.peek() == Some('\r') {
                    cursor.bump();
                }
                if cursor.peek() == Some('\n') {
                    cursor.bump();
                }
                continue;
            }

            // 2. We have a non-empty, non-comment line. Validate and handle indentation.
            if space_count % 4 != 0 {
                panic!(
                    "Lexical error: Indentation must be a multiple of 4 spaces, got {} spaces (line {}, column {})",
                    space_count, start_line, start_col
                );
            }

            let current_indent = *indent_stack.last().unwrap_or(&0);
            if space_count > current_indent {
                indent_stack.push(space_count);
                tokens.push(Token {
                    kind: TokenKind::Indent,
                    line: start_line,
                    column: start_col,
                });
            } else if space_count < current_indent {
                while let Some(&top) = indent_stack.last() {
                    if top == space_count {
                        break;
                    }
                    if top < space_count {
                        panic!(
                            "Lexical error: Indentation does not match any outer level (line {}, column {})",
                            start_line, start_col
                        );
                    }
                    indent_stack.pop();
                    tokens.push(Token {
                        kind: TokenKind::Dedent,
                        line: start_line,
                        column: start_col,
                    });
                }
            }

            // 3. Process tokens on this line
            loop {
                // Skip horizontal spaces
                while let Some(c) = cursor.peek() {
                    if c == ' ' {
                        cursor.bump();
                    } else if c == '\t' {
                        panic!(
                            "Lexical error: Tab character is not allowed. (line {}, column {})",
                            cursor.line, cursor.column
                        );
                    } else {
                        break;
                    }
                }

                let c = match cursor.peek() {
                    None => {
                        break;
                    }
                    Some('\n') => {
                        let tok_line = cursor.line;
                        let tok_col = cursor.column;
                        cursor.bump();
                        tokens.push(Token {
                            kind: TokenKind::Newline,
                            line: tok_line,
                            column: tok_col,
                        });
                        break;
                    }
                    Some('\r') => {
                        let tok_line = cursor.line;
                        let tok_col = cursor.column;
                        cursor.bump();
                        if cursor.peek() == Some('\n') {
                            cursor.bump();
                        }
                        tokens.push(Token {
                            kind: TokenKind::Newline,
                            line: tok_line,
                            column: tok_col,
                        });
                        break;
                    }
                    Some('#') => {
                        cursor.bump(); // consume '#'
                        while let Some(cc) = cursor.peek() {
                            if cc == '\n' || cc == '\r' {
                                break;
                            }
                            cursor.bump();
                        }
                        continue;
                    }
                    Some(other) => other,
                };

                let tok_line = cursor.line;
                let tok_col = cursor.column;

                if c == 'f' && cursor.peek_next() == Some('"') {
                    cursor.bump(); // consume 'f'
                    cursor.bump(); // consume '"'
                    tokens.push(Token {
                        kind: TokenKind::FStringStart,
                        line: tok_line,
                        column: tok_col,
                    });

                    let mut current_text = String::new();
                    let mut text_start_line = cursor.line;
                    let mut text_start_col = cursor.column;

                    loop {
                        match cursor.peek() {
                            None => panic!(
                                "Lexical error: Unterminated f-string starting at line {}, column {}",
                                tok_line, tok_col
                            ),
                            Some('"') => {
                                cursor.bump(); // consume '"'
                                if !current_text.is_empty() {
                                    tokens.push(Token {
                                        kind: TokenKind::FStringText(current_text.clone()),
                                        line: text_start_line,
                                        column: text_start_col,
                                    });
                                }
                                tokens.push(Token {
                                    kind: TokenKind::FStringEnd,
                                    line: cursor.line,
                                    column: cursor.column,
                                });
                                break;
                            }
                            Some('{') => {
                                cursor.bump(); // consume '{'
                                if !current_text.is_empty() {
                                    tokens.push(Token {
                                        kind: TokenKind::FStringText(current_text.clone()),
                                        line: text_start_line,
                                        column: text_start_col,
                                    });
                                    current_text.clear();
                                }
                                tokens.push(Token {
                                    kind: TokenKind::FStringExprStart,
                                    line: cursor.line,
                                    column: cursor.column,
                                });

                                let mut brace_nesting = 0;
                                loop {
                                    // Skip whitespace inside expression
                                    while let Some(ws) = cursor.peek() {
                                        if ws == ' ' {
                                            cursor.bump();
                                        } else if ws == '\t' {
                                            panic!(
                                                "Lexical error: Tab character is not allowed. (line {}, column {})",
                                                cursor.line, cursor.column
                                            );
                                        } else {
                                            break;
                                        }
                                    }

                                    match cursor.peek() {
                                        None => panic!(
                                            "Lexical error: Unterminated f-string expression starting inside f-string at line {}, column {}",
                                            tok_line, tok_col
                                        ),
                                        Some('}') if brace_nesting == 0 => {
                                            cursor.bump(); // consume '}'
                                            tokens.push(Token {
                                                kind: TokenKind::FStringExprEnd,
                                                line: cursor.line,
                                                column: cursor.column,
                                            });
                                            break;
                                        }
                                        Some('"') => {
                                            panic!(
                                                "Lexical error: Nested string literal inside f-string expression is not allowed. (line {}, column {})",
                                                cursor.line, cursor.column
                                            );
                                        }
                                        _ => {}
                                    };

                                    let expr_tok = tokenize_single_expression_token(&mut cursor, &mut brace_nesting);
                                    tokens.push(expr_tok);
                                }
                                // Reset text segment positions after expression
                                text_start_line = cursor.line;
                                text_start_col = cursor.column;
                            }
                            Some('\\') => {
                                cursor.bump(); // consume '\\'
                                if let Some(esc) = cursor.bump() {
                                    match esc {
                                        'n' => current_text.push('\n'),
                                        'r' => current_text.push('\r'),
                                        't' => current_text.push('\t'),
                                        '\\' => current_text.push('\\'),
                                        '"' => current_text.push('"'),
                                        '{' => current_text.push('{'),
                                        '}' => current_text.push('}'),
                                        _ => current_text.push(esc),
                                    }
                                } else {
                                    panic!(
                                        "Lexical error: Unterminated f-string escape sequence starting inside f-string at line {}, column {}",
                                        tok_line, tok_col
                                    );
                                }
                            }
                            Some(_) => {
                                if current_text.is_empty() {
                                    text_start_line = cursor.line;
                                    text_start_col = cursor.column;
                                }
                                current_text.push(cursor.bump().unwrap());
                            }
                        };
                    }
                } else if is_ident_start(c) {
                    let mut ident_str = String::new();
                    while let Some(cc) = cursor.peek() {
                        if is_ident_continue(cc) {
                            ident_str.push(cursor.bump().unwrap());
                        } else {
                            break;
                        }
                    }

                    let kind = match ident_str.as_str() {
                        "true" => TokenKind::Bool(true),
                        "false" => TokenKind::Bool(false),
                        "fn" => TokenKind::Fn,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "elif" => TokenKind::Elif,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "type" => TokenKind::Type,
                        "task" => TokenKind::Task,
                        "return" => TokenKind::Return,
                        "use" => TokenKind::Use,
                        "const" => TokenKind::Const,
                        "priv" => TokenKind::Priv,
                        "extern" => TokenKind::Extern,
                        "export" => TokenKind::Export,
                        "unsafe" => TokenKind::Unsafe,
                        "weak" => TokenKind::Weak,
                        "and" => TokenKind::And,
                        "or" => TokenKind::Or,
                        "not" => TokenKind::Not,
                        "try" => TokenKind::Try,
                        "spawn" => TokenKind::Spawn,
                        _ => TokenKind::Ident(ident_str),
                    };

                    tokens.push(Token {
                        kind,
                        line: tok_line,
                        column: tok_col,
                    });
                } else if c.is_ascii_digit() {
                    let mut num_str = String::new();
                    while let Some(cc) = cursor.peek() {
                        if cc.is_ascii_digit() {
                            num_str.push(cursor.bump().unwrap());
                        } else {
                            break;
                        }
                    }

                    // Check for Float
                    let is_float = if cursor.peek() == Some('.') {
                        if let Some(c2) = cursor.peek_next() {
                            c2.is_ascii_digit()
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_float {
                        num_str.push(cursor.bump().unwrap()); // consume '.'
                        while let Some(cc) = cursor.peek() {
                            if cc.is_ascii_digit() {
                                num_str.push(cursor.bump().unwrap());
                            } else {
                                break;
                            }
                        }
                        let val = num_str.parse::<f64>().expect("Invalid float literal");
                        tokens.push(Token {
                            kind: TokenKind::Float(val),
                            line: tok_line,
                            column: tok_col,
                        });
                    } else {
                        let val = num_str.parse::<i64>().expect("Invalid integer literal");
                        tokens.push(Token {
                            kind: TokenKind::Number(val),
                            line: tok_line,
                            column: tok_col,
                        });
                    }
                } else if c == '"' {
                    cursor.bump(); // consume starting '"'
                    let mut s = String::new();
                    let mut terminated = false;

                    while let Some(cc) = cursor.peek() {
                        if cc == '"' {
                            cursor.bump(); // consume ending '"'
                            terminated = true;
                            break;
                        } else if cc == '\\' {
                            cursor.bump(); // consume '\'
                            if let Some(esc) = cursor.bump() {
                                match esc {
                                    'n' => s.push('\n'),
                                    'r' => s.push('\r'),
                                    't' => s.push('\t'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    _ => s.push(esc),
                                }
                            } else {
                                panic!(
                                    "Lexical error: Unterminated string literal escape sequence (line {}, column {})",
                                    cursor.line, cursor.column
                                );
                            }
                        } else {
                            s.push(cursor.bump().unwrap());
                        }
                    }

                    if !terminated {
                        panic!(
                            "Lexical error: Unterminated string literal starting at line {}, column {}",
                            tok_line, tok_col
                        );
                    }

                    tokens.push(Token {
                        kind: TokenKind::String(s),
                        line: tok_line,
                        column: tok_col,
                    });
                } else {
                    // Operators and Delimiters
                    let kind = match cursor.bump().unwrap() {
                        '(' => TokenKind::LParen,
                        ')' => TokenKind::RParen,
                        '[' => TokenKind::LBracket,
                        ']' => TokenKind::RBracket,
                        '{' => TokenKind::LBrace,
                        '}' => TokenKind::RBrace,
                        '.' => TokenKind::Dot,
                        ',' => TokenKind::Comma,
                        ':' => TokenKind::Colon,
                        '%' => TokenKind::Percent,
                        '+' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::PlusEq
                            } else {
                                TokenKind::Plus
                            }
                        }
                        '-' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::MinusEq
                            } else if cursor.peek() == Some('>') {
                                cursor.bump();
                                TokenKind::Arrow
                            } else {
                                TokenKind::Minus
                            }
                        }
                        '*' => {
                            if cursor.peek() == Some('*') {
                                cursor.bump();
                                TokenKind::Power
                            } else if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::StarEq
                            } else {
                                TokenKind::Star
                            }
                        }
                        '/' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::SlashEq
                            } else {
                                TokenKind::Slash
                            }
                        }
                        '=' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::EqEq
                            } else {
                                TokenKind::Eq
                            }
                        }
                        '!' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::NotEq
                            } else {
                                panic!(
                                    "Lexical error: Unexpected character '!' at line {}, column {}",
                                    tok_line, tok_col
                                );
                            }
                        }
                        '<' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::LtEq
                            } else {
                                TokenKind::Lt
                            }
                        }
                        '>' => {
                            if cursor.peek() == Some('=') {
                                cursor.bump();
                                TokenKind::GtEq
                            } else {
                                TokenKind::Gt
                            }
                        }
                        other => {
                            panic!(
                                "Lexical error: Unexpected character '{}' at line {}, column {}",
                                other, tok_line, tok_col
                            );
                        }
                    };

                    tokens.push(Token {
                        kind,
                        line: tok_line,
                        column: tok_col,
                    });
                }
            }
        }

        // EOF handling: Check if we need to emit a trailing newline first
        let last_kind = tokens.last().map(|t| &t.kind);
        if let Some(kind) = last_kind {
            match kind {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Indent => {}
                _ => {
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        line: cursor.line,
                        column: cursor.column,
                    });
                }
            }
        }

        // Pop all remaining indentation levels above 0
        while indent_stack.len() > 1 {
            indent_stack.pop();
            tokens.push(Token {
                kind: TokenKind::Dedent,
                line: cursor.line,
                column: cursor.column,
            });
        }

        // Always append EOF token at the very end
        tokens.push(Token {
            kind: TokenKind::Eof,
            line: cursor.line,
            column: cursor.column,
        });

        tokens
    }
}
