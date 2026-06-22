use ast::*;
use lexer::TokenKind;
use crate::Parser;
use crate::precedence::Precedence;

impl Parser {
    pub(crate) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_precedence(Precedence::None)
    }

    pub(crate) fn parse_expr_comma(&mut self) -> Expr {
        let first = self.parse_expr_precedence(Precedence::None);
        if self.check(TokenKind::Comma) {
            let mut exprs = vec![first];
            while self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma);
                // Allow trailing comma in tuples before closing parenthesis, newline, or assignment operator
                if self.check(TokenKind::RParen) || self.check(TokenKind::Newline) || self.check(TokenKind::Eq) {
                    break;
                }
                exprs.push(self.parse_expr_precedence(Precedence::None));
            }
            Expr::Tuple(exprs)
        } else {
            first
        }
    }

    pub(crate) fn parse_expr_precedence(&mut self, prec: Precedence) -> Expr {
        let mut left = self.parse_prefix();

        while prec < self.current_precedence() {
            left = self.parse_infix(left);
        }

        left
    }

    pub(crate) fn parse_prefix(&mut self) -> Expr {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(name) => Expr::Ident(name.clone()),
            TokenKind::Number(n) => Expr::Literal(Literal::Int(*n)),
            TokenKind::Float(f) => Expr::Literal(Literal::Float(*f)),
            TokenKind::String(s) => Expr::Literal(Literal::String(s.clone())),
            TokenKind::Bool(b) => Expr::Literal(Literal::Bool(*b)),
            TokenKind::Fn => {
                let params = self.parse_params();
                let return_type = self.parse_return_type();
                let body = if self.check(TokenKind::Newline) {
                    self.consume(TokenKind::Newline);
                    self.parse_block()
                } else {
                    let expr = self.parse_expr_precedence(Precedence::None);
                    Block {
                        stmts: vec![Stmt::Return(Some(expr))],
                    }
                };
                Expr::AnonymousFn {
                    params,
                    return_type,
                    body,
                }
            }
            TokenKind::LParen => {
                if self.check(TokenKind::RParen) {
                    self.advance(); // consume RParen
                    Expr::Tuple(vec![])
                } else {
                    let expr = self.parse_expr_comma();
                    self.consume(TokenKind::RParen);
                    expr
                }
            }
            TokenKind::FStringStart => {
                let mut parts = Vec::new();
                while !self.check(TokenKind::FStringEnd) {
                    if let TokenKind::FStringText(text) = self.peek_kind() {
                        let text_val = text.clone();
                        self.advance();
                        parts.push(FStringPart::Text(text_val));
                    } else if self.check(TokenKind::FStringExprStart) {
                        self.advance(); // consume FStringExprStart
                        let expr = self.parse_expr();
                        self.consume(TokenKind::FStringExprEnd);
                        parts.push(FStringPart::Expr(expr));
                    } else {
                        let tok = self.peek();
                        panic!(
                            "Parser error: Unexpected token '{}' inside f-string at {}:{}",
                            tok.kind, tok.line, tok.column
                        );
                    }
                }
                self.consume(TokenKind::FStringEnd);
                Expr::FStringExpr(parts)
            }
            TokenKind::LBracket => {
                // List literal: [expr, expr, ...]
                let mut items = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        items.push(self.parse_expr());
                        if self.check(TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBracket);
                Expr::ListLiteral(items)
            }
            TokenKind::LBrace => {
                // Map literal: {"key": value, ...}
                let mut pairs = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        let key = self.parse_expr();
                        self.consume(TokenKind::Colon);
                        let value = self.parse_expr();
                        pairs.push((key, value));
                        if self.check(TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBrace);
                Expr::MapLiteral(pairs)
            }
            TokenKind::Minus => {
                let expr = self.parse_expr_precedence(Precedence::Unary);
                Expr::UnaryExpr {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Not => {
                let expr = self.parse_expr_precedence(Precedence::Unary);
                Expr::UnaryExpr {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Try => {
                let expr = self.parse_expr_precedence(Precedence::Unary);
                Expr::TryExpr(Box::new(expr))
            }
            other => panic!(
                "Parser error: Expected prefix expression, found '{}' at {}:{}",
                other, tok.line, tok.column
            ),
        }
    }

    pub(crate) fn current_precedence(&self) -> Precedence {
        match self.peek_kind() {
            TokenKind::LParen => Precedence::Call,
            TokenKind::LBracket => Precedence::Call,
            TokenKind::Dot => Precedence::Call,
            TokenKind::Power => Precedence::Power,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::EqEq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq => Precedence::Comparison,
            TokenKind::And => Precedence::And,
            TokenKind::Or => Precedence::Or,
            TokenKind::Pipe => Precedence::Pipe,
            _ => Precedence::None,
        }
    }

    pub(crate) fn parse_infix(&mut self, left: Expr) -> Expr {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::LParen => {
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr());
                        if self.check(TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RParen);
                Expr::CallExpr {
                    callee: Box::new(left),
                    args,
                }
            }
            TokenKind::LBracket => {
                let line = tok.line;
                let index = self.parse_expr();
                self.consume(TokenKind::RBracket);
                Expr::Index {
                    expr: Box::new(left),
                    index: Box::new(index),
                    line,
                }
            }
            TokenKind::Dot => {
                let field_tok = self.advance().clone();
                let field = match field_tok.kind {
                    TokenKind::Ident(name) => name,
                    other => panic!(
                        "Parser error: Expected field name, found '{}' at {}:{}",
                        other, field_tok.line, field_tok.column
                    ),
                };
                Expr::FieldAccess {
                    expr: Box::new(left),
                    field,
                }
            }
            TokenKind::Power => {
                // Right associative, parse at same precedence level
                let right = self.parse_expr_precedence(Precedence::Power);
                Expr::BinExpr {
                    left: Box::new(left),
                    op: BinOp::Pow,
                    right: Box::new(right),
                }
            }
            TokenKind::Pipe => {
                let right = self.parse_expr_precedence(Precedence::Pipe);
                match right {
                    Expr::CallExpr { callee, mut args } => {
                        args.insert(0, left);
                        Expr::CallExpr { callee, args }
                    }
                    _ => Expr::CallExpr {
                        callee: Box::new(right),
                        args: vec![left],
                    },
                }
            }
            kind => {
                let op = match &kind {
                    TokenKind::Plus => BinOp::Add,
                    TokenKind::Minus => BinOp::Sub,
                    TokenKind::Star => BinOp::Mul,
                    TokenKind::Slash => BinOp::Div,
                    TokenKind::Percent => BinOp::Mod,
                    TokenKind::EqEq => BinOp::Eq,
                    TokenKind::NotEq => BinOp::Ne,
                    TokenKind::Lt => BinOp::Lt,
                    TokenKind::Gt => BinOp::Gt,
                    TokenKind::LtEq => BinOp::Le,
                    TokenKind::GtEq => BinOp::Ge,
                    TokenKind::And => BinOp::And,
                    TokenKind::Or => BinOp::Or,
                    other => panic!(
                        "Parser error: Unexpected infix operator '{}' at {}:{}",
                        other, tok.line, tok.column
                    ),
                };
                // Left associative, parse at next precedence level
                let right_prec = self.kind_precedence(&kind);
                let right = self.parse_expr_precedence(right_prec);
                Expr::BinExpr {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
        }
    }

    pub(crate) fn kind_precedence(&self, kind: &TokenKind) -> Precedence {
        match kind {
            TokenKind::Power => Precedence::Power,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::EqEq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq => Precedence::Comparison,
            TokenKind::And => Precedence::And,
            TokenKind::Or => Precedence::Or,
            TokenKind::Pipe => Precedence::Pipe,
            _ => Precedence::None,
        }
    }
}
