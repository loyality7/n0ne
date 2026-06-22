use ast::*;
use lexer::TokenKind;
use crate::Parser;

impl Parser {
    pub(crate) fn parse_block(&mut self) -> Block {
        self.consume(TokenKind::Indent);
        let mut stmts = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt());
        }
        self.consume(TokenKind::Dedent);
        Block { stmts }
    }

    pub(crate) fn parse_stmt(&mut self) -> Stmt {
        if self.check(TokenKind::If) {
            self.parse_if_stmt()
        } else if self.check(TokenKind::For) {
            self.parse_for_stmt()
        } else if self.check(TokenKind::While) {
            self.parse_while_stmt()
        } else if self.check(TokenKind::Match) {
            self.parse_match_stmt()
        } else if self.check(TokenKind::Break) {
            self.advance();
            self.consume(TokenKind::Newline);
            Stmt::Break
        } else if self.check(TokenKind::Continue) {
            self.advance();
            self.consume(TokenKind::Newline);
            Stmt::Continue
        } else if self.check(TokenKind::Return) {
            self.parse_return_stmt()
        } else if self.check(TokenKind::Const) {
            let const_decl = self.parse_const_decl();
            self.consume(TokenKind::Newline);
            Stmt::ConstDecl(const_decl)
        } else if self.check(TokenKind::Defer) {
            self.advance();
            let expr = self.parse_expr();
            self.consume(TokenKind::Newline);
            Stmt::Defer(expr)
        } else if self.check(TokenKind::Guard) {
            self.parse_guard_stmt()
        } else {
            // Either assign statement or expression statement
            let target = self.parse_expr_comma();
            if self.is_assign_op() {
                let op = self.parse_assign_op();
                let value = self.parse_expr_comma();
                self.consume(TokenKind::Newline);
                Stmt::Assign { target, op, value }
            } else {
                self.consume(TokenKind::Newline);
                Stmt::Expr(target)
            }
        }
    }

    pub(crate) fn is_assign_op(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
        )
    }

    pub(crate) fn parse_assign_op(&mut self) -> AssignOp {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Eq => AssignOp::Eq,
            TokenKind::PlusEq => AssignOp::PlusEq,
            TokenKind::MinusEq => AssignOp::MinusEq,
            TokenKind::StarEq => AssignOp::StarEq,
            TokenKind::SlashEq => AssignOp::SlashEq,
            other => panic!(
                "Parser error: Expected assignment operator, found '{}' at {}:{}",
                other, tok.line, tok.column
            ),
        }
    }

    pub(crate) fn parse_if_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::If);
        let cond = self.parse_expr();
        self.consume(TokenKind::Newline);
        let then_branch = self.parse_block();

        let mut elifs = Vec::new();
        while self.check(TokenKind::Elif) {
            self.consume(TokenKind::Elif);
            let elif_cond = self.parse_expr();
            self.consume(TokenKind::Newline);
            let elif_body = self.parse_block();
            elifs.push((elif_cond, elif_body));
        }

        let else_branch = if self.check(TokenKind::Else) {
            self.consume(TokenKind::Else);
            self.consume(TokenKind::Newline);
            Some(self.parse_block())
        } else {
            None
        };

        Stmt::If {
            cond,
            then_branch,
            elifs,
            else_branch,
        }
    }

    pub(crate) fn parse_for_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::For);
        let var_tok = self.advance();
        let var = match &var_tok.kind {
            TokenKind::Ident(name) => name.clone(),
            other => panic!(
                "Parser error: Expected loop variable name, found '{}' at {}:{}",
                other, var_tok.line, var_tok.column
            ),
        };
        self.consume(TokenKind::In);
        let iterable = self.parse_expr();
        self.consume(TokenKind::Newline);
        let body = self.parse_block();

        Stmt::For {
            var,
            iterable,
            body,
        }
    }

    pub(crate) fn parse_return_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::Return);
        let val = if !self.check(TokenKind::Newline) && !self.is_at_end() {
            Some(self.parse_expr())
        } else {
            None
        };
        self.consume(TokenKind::Newline);
        Stmt::Return(val)
    }

    pub(crate) fn parse_while_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::While);
        let cond = self.parse_expr();
        self.consume(TokenKind::Newline);
        let body = self.parse_block();
        Stmt::While { cond, body }
    }

    pub(crate) fn parse_match_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::Match);
        let expr = self.parse_expr();
        self.consume(TokenKind::Newline);
        self.consume(TokenKind::Indent);
        let mut cases = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }
            let arm = self.parse_match_arm();
            self.consume(TokenKind::Arrow);
            let body = if self.check(TokenKind::Newline) {
                self.consume(TokenKind::Newline);
                self.parse_block()
            } else {
                let stmt = self.parse_stmt();
                Block { stmts: vec![stmt] }
            };
            cases.push((arm, body));
        }
        self.consume(TokenKind::Dedent);
        Stmt::Match { expr, cases }
    }

    pub(crate) fn parse_match_arm(&mut self) -> MatchArm {
        let tok = self.peek();
        match &tok.kind {
            TokenKind::Ident(name) if name == "_" => {
                self.advance();
                MatchArm::Wildcard
            }
            TokenKind::Ident(name) => {
                let var_name = name.clone();
                self.advance();
                let mut bindings = Vec::new();
                if self.check(TokenKind::LParen) {
                    self.consume(TokenKind::LParen);
                    if !self.check(TokenKind::RParen) {
                        loop {
                            let b_tok = self.advance();
                            let b_name = match &b_tok.kind {
                                TokenKind::Ident(n) => n.clone(),
                                other => panic!(
                                    "Parser error: Expected identifier in variant pattern, found '{}' at {}:{}",
                                    other, b_tok.line, b_tok.column
                                ),
                            };
                            bindings.push(b_name);
                            if self.check(TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RParen);
                }
                MatchArm::Variant {
                    variant_name: var_name,
                    bindings,
                }
            }
            TokenKind::Number(n) => {
                let val = *n;
                self.advance();
                MatchArm::Literal(Literal::Int(val))
            }
            TokenKind::Float(f) => {
                let val = *f;
                self.advance();
                MatchArm::Literal(Literal::Float(val))
            }
            TokenKind::String(s) => {
                let val = s.clone();
                self.advance();
                MatchArm::Literal(Literal::String(val))
            }
            TokenKind::Bool(b) => {
                let val = *b;
                self.advance();
                MatchArm::Literal(Literal::Bool(val))
            }
            other => panic!(
                "Parser error: Expected match arm (literal, variant or wildcard '_'), found '{}' at {}:{}",
                other, tok.line, tok.column
            ),
        }
    }

    pub(crate) fn parse_guard_stmt(&mut self) -> Stmt {
        self.consume(TokenKind::Guard);
        let cond = self.parse_expr();
        self.consume(TokenKind::Else);
        self.consume(TokenKind::Newline);
        let else_branch = self.parse_block();
        Stmt::Guard { cond, else_branch }
    }
}
