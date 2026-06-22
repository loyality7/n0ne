use ast::*;
use lexer::TokenKind;
use crate::Parser;

impl Parser {
    pub(crate) fn parse_program(&mut self) -> Program {
        let mut decls = Vec::new();
        while !self.is_at_end() {
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }
            decls.push(self.parse_top_level_decl());
        }
        Program { decls }
    }

    pub(crate) fn parse_top_level_decl(&mut self) -> TopLevelDecl {
        if self.check(TokenKind::Fn) {
            TopLevelDecl::FnDecl(self.parse_fn_decl())
        } else if self.check(TokenKind::Type) {
            self.parse_type_or_alias_decl()
        } else if self.check(TokenKind::Enum) {
            TopLevelDecl::EnumDecl(self.parse_enum_decl())
        } else if self.check(TokenKind::Task) {
            TopLevelDecl::TaskDecl(self.parse_task_decl())
        } else if self.check(TokenKind::Use) {
            TopLevelDecl::UseDecl(self.parse_use_decl())
        } else if self.check(TokenKind::Const) {
            let const_decl = self.parse_const_decl();
            self.consume(TokenKind::Newline);
            TopLevelDecl::ConstDecl(const_decl)
        } else {
            let tok = self.peek();
            panic!(
                "Parser error: Expected top-level declaration (fn, type, task, use, const), found '{}' at {}:{}",
                tok.kind, tok.line, tok.column
            );
        }
    }

    pub(crate) fn parse_fn_decl(&mut self) -> FnDecl {
        self.consume(TokenKind::Fn);

        // Check for receiver (self: Type)
        let receiver = if self.check(TokenKind::LParen) {
            // We need to check if the signature is "fn (self User)" or "fn greet(x: int)".
            // If the next tokens inside LPAREN look like (Ident, Ident), it's receiver.
            // If it has a Colon (e.g. self: User), it's also a receiver. Let's support both.
            let next_tok = self.peek_next();
            let looks_like_receiver = match &next_tok.kind {
                TokenKind::Ident(_) => true,
                _ => false,
            };

            if looks_like_receiver {
                self.consume(TokenKind::LParen);
                let self_tok = self.advance();
                let self_name = match &self_tok.kind {
                    TokenKind::Ident(name) => name.clone(),
                    other => panic!(
                        "Parser error: Expected receiver name, found '{}' at {}:{}",
                        other, self_tok.line, self_tok.column
                    ),
                };
                
                // Read the receiver type. It could be optionally preceded by colon.
                if self.check(TokenKind::Colon) {
                    self.advance();
                }
                
                let type_tok = self.advance();
                let type_name = match &type_tok.kind {
                    TokenKind::Ident(name) => name.clone(),
                    other => panic!(
                        "Parser error: Expected receiver type name, found '{}' at {}:{}",
                        other, type_tok.line, type_tok.column
                    ),
                };
                self.consume(TokenKind::RParen);
                Some(Receiver {
                    name: self_name,
                    type_name,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Parse function name
        let name_tok = self.advance();
        let name = match &name_tok.kind {
            TokenKind::Ident(n) => n.clone(),
            other => panic!(
                "Parser error: Expected function name, found '{}' at {}:{}",
                other, name_tok.line, name_tok.column
            ),
        };

        let params = self.parse_params();
        let return_type = self.parse_return_type();

        self.consume(TokenKind::Newline);
        let body = self.parse_block();

        FnDecl {
            name,
            receiver,
            params,
            return_type,
            body,
        }
    }

    pub(crate) fn parse_params(&mut self) -> Vec<Param> {
        self.consume(TokenKind::LParen);
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let p_tok = self.advance();
                let p_name = match &p_tok.kind {
                    TokenKind::Ident(n) => n.clone(),
                    other => panic!(
                        "Parser error: Expected parameter name, found '{}' at {}:{}",
                        other, p_tok.line, p_tok.column
                    ),
                };
                let p_type = if self.check(TokenKind::Colon) {
                    self.consume(TokenKind::Colon);
                    self.parse_type()
                } else {
                    Type::Basic("unknown".to_string())
                };
                let default_value = if self.check(TokenKind::Eq) {
                    self.consume(TokenKind::Eq);
                    Some(self.parse_expr())
                } else {
                    None
                };
                params.push(Param {
                    name: p_name,
                    type_ann: p_type,
                    default_value,
                });
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen);
        params
    }

    pub(crate) fn parse_return_type(&mut self) -> Option<Type> {
        if self.check(TokenKind::Arrow) {
            self.advance(); // consume "->"
            Some(self.parse_type())
        } else {
            None
        }
    }

    pub(crate) fn parse_type_or_alias_decl(&mut self) -> TopLevelDecl {
        self.consume(TokenKind::Type);
        let name_tok = self.advance();
        let name = match &name_tok.kind {
            TokenKind::Ident(n) => n.clone(),
            other => panic!(
                "Parser error: Expected type name, found '{}' at {}:{}",
                other, name_tok.line, name_tok.column
            ),
        };

        if self.check(TokenKind::Eq) {
            self.consume(TokenKind::Eq);
            let target_type = self.parse_type();
            if self.check(TokenKind::Newline) {
                self.consume(TokenKind::Newline);
            }
            TopLevelDecl::TypeAliasDecl(TypeAliasDecl { name, target_type })
        } else {
            self.consume(TokenKind::Newline);
            self.consume(TokenKind::Indent);

            let mut fields = Vec::new();
            while !self.check(TokenKind::Dedent) && !self.is_at_end() {
                if self.check(TokenKind::Newline) {
                    self.advance();
                    continue;
                }
                let f_tok = self.advance();
                let f_name = match &f_tok.kind {
                    TokenKind::Ident(n) => n.clone(),
                    other => panic!(
                        "Parser error: Expected field name, found '{}' at {}:{}",
                        other, f_tok.line, f_tok.column
                    ),
                };
                self.consume(TokenKind::Colon);
                let f_type = self.parse_type();
                self.consume(TokenKind::Newline);
                fields.push(Field {
                    name: f_name,
                    type_ann: f_type,
                });
            }
            self.consume(TokenKind::Dedent);

            TopLevelDecl::TypeDecl(TypeDecl { name, fields })
        }
    }

    pub(crate) fn parse_enum_decl(&mut self) -> EnumDecl {
        self.consume(TokenKind::Enum);
        let name_tok = self.advance();
        let name = match &name_tok.kind {
            TokenKind::Ident(n) => n.clone(),
            other => panic!(
                "Parser error: Expected enum name, found '{}' at {}:{}",
                other, name_tok.line, name_tok.column
            ),
        };
        self.consume(TokenKind::Newline);
        self.consume(TokenKind::Indent);

        let mut variants = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }
            let v_tok = self.advance();
            let v_name = match &v_tok.kind {
                TokenKind::Ident(n) => n.clone(),
                other => panic!(
                    "Parser error: Expected variant name, found '{}' at {}:{}",
                    other, v_tok.line, v_tok.column
                ),
            };

            let mut fields = Vec::new();
            if self.check(TokenKind::LParen) {
                self.consume(TokenKind::LParen);
                if !self.check(TokenKind::RParen) {
                    loop {
                        let f_type = self.parse_type();
                        fields.push(f_type);
                        if self.check(TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RParen);
            }
            self.consume(TokenKind::Newline);
            variants.push(EnumVariant {
                name: v_name,
                fields,
            });
        }
        self.consume(TokenKind::Dedent);

        EnumDecl { name, variants }
    }

    pub(crate) fn parse_task_decl(&mut self) -> TaskDecl {
        self.consume(TokenKind::Task);
        let name_tok = self.advance();
        let name = match &name_tok.kind {
            TokenKind::Ident(n) => n.clone(),
            other => panic!(
                "Parser error: Expected task name, found '{}' at {}:{}",
                other, name_tok.line, name_tok.column
            ),
        };
        self.consume(TokenKind::Newline);
        let body = self.parse_block();
        TaskDecl { name, body }
    }

    pub(crate) fn parse_use_decl(&mut self) -> UseDecl {
        self.consume(TokenKind::Use);
        let mut path = String::new();
        let tok_line = self.peek().line;
        let tok_col = self.peek().column;
        while !self.check(TokenKind::Newline) && !self.check(TokenKind::LBrace) && !self.is_at_end() {
            let tok = self.advance();
            path.push_str(&Self::token_to_string(&tok.kind));
        }
        if path.is_empty() {
            panic!("Parser error: Expected path after 'use' at {}:{}", tok_line, tok_col);
        }
        
        let kind = if path.starts_with("./") || path.starts_with("../") {
            UseKind::Local
        } else if path.contains('/') {
            UseKind::Package
        } else {
            UseKind::Stdlib
        };

        let mut items = None;
        if self.check(TokenKind::LBrace) {
            self.consume(TokenKind::LBrace);
            let mut list = Vec::new();
            while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                if self.check(TokenKind::Newline) {
                    self.advance();
                    continue;
                }
                let tok = self.advance();
                let name = match &tok.kind {
                    TokenKind::Ident(name) => name.clone(),
                    other => panic!(
                        "Parser error: Expected identifier in import items list, found '{}' at {}:{}",
                        other, tok.line, tok.column
                    ),
                };
                list.push(name);
                if self.check(TokenKind::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenKind::RBrace);
            items = Some(list);
        }

        self.consume(TokenKind::Newline);
        UseDecl { path, kind, items }
    }

    pub(crate) fn token_to_string(kind: &TokenKind) -> String {
        match kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Number(n) => n.to_string(),
            TokenKind::Float(f) => f.to_string(),
            TokenKind::String(s) => s.clone(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::SlashEq => "/=".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Star => "*".to_string(),
            other => format!("{}", other),
        }
    }

    pub(crate) fn parse_const_decl(&mut self) -> ConstDecl {
        self.consume(TokenKind::Const);
        let name_tok = self.advance();
        let name = match &name_tok.kind {
            TokenKind::Ident(n) => n.clone(),
            other => panic!(
                "Parser error: Expected identifier, found '{}' at {}:{}",
                other, name_tok.line, name_tok.column
            ),
        };
        self.consume(TokenKind::Eq);
        let value = self.parse_expr();
        ConstDecl { name, value }
    }

    pub(crate) fn parse_type(&mut self) -> Type {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::LParen => {
                let mut types = Vec::new();
                if !self.check(TokenKind::RParen) {
                    types.push(self.parse_type());
                    while self.check(TokenKind::Comma) {
                        self.consume(TokenKind::Comma);
                        types.push(self.parse_type());
                    }
                }
                self.consume(TokenKind::RParen);
                Type::Tuple(types)
            }
            TokenKind::Ident(name) => match name.as_str() {
                "list" => {
                    self.consume(TokenKind::LBracket);
                    let inner = self.parse_type();
                    self.consume(TokenKind::RBracket);
                    Type::List(Box::new(inner))
                }
                "map" => {
                    self.consume(TokenKind::LBracket);
                    let key = self.parse_type();
                    self.consume(TokenKind::Comma);
                    let val = self.parse_type();
                    self.consume(TokenKind::RBracket);
                    Type::Map(Box::new(key), Box::new(val))
                }
                "result" => {
                    self.consume(TokenKind::LBracket);
                    let inner = self.parse_type();
                    self.consume(TokenKind::RBracket);
                    Type::Result(Box::new(inner))
                }
                "option" => {
                    self.consume(TokenKind::LBracket);
                    let inner = self.parse_type();
                    self.consume(TokenKind::RBracket);
                    Type::Option(Box::new(inner))
                }
                _ => Type::Basic(name.clone()),
            },
            other => panic!(
                "Parser error: Expected type name, found '{}' at {}:{}",
                other, tok.line, tok.column
            ),
        }
    }
}
