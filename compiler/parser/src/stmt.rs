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
        } else if self.check(TokenKind::Return) {
            self.parse_return_stmt()
        } else if self.check(TokenKind::Const) {
            let const_decl = self.parse_const_decl();
            self.consume(TokenKind::Newline);
            Stmt::ConstDecl(const_decl)
        } else {
            // Either assign statement or expression statement
            let target = self.parse_expr();
            if self.is_assign_op() {
                let op = self.parse_assign_op();
                let value = self.parse_expr();
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
}
