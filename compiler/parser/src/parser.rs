use ast::*;
use lexer::{Token, TokenKind};

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Program {
        self.parse_program()
    }

    pub(crate) fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    pub(crate) fn peek_next(&self) -> &Token {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            tok
        } else {
            self.peek()
        }
    }

    pub(crate) fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(&kind)
    }

    pub(crate) fn consume(&mut self, kind: TokenKind) -> &Token {
        if self.check(kind.clone()) {
            self.advance()
        } else {
            let tok = self.peek();
            panic!(
                "Parser error: Expected token of kind '{:?}', found '{:?}' at {}:{}",
                kind, tok.kind, tok.line, tok.column
            );
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}
