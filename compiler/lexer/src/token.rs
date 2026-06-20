#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Ident(String),
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Special Layout
    Newline,
    Indent,
    Dedent,
    Eof,

    // Keywords
    Fn,
    If,
    Else,
    Elif,
    For,
    In,
    Type,
    Task,
    Return,
    Use,
    Const,
    Priv,
    Extern,
    Export,
    Unsafe,
    Weak,
    And,
    Or,
    Not,
    Try,
    Spawn,
    While,
    Break,
    Continue,
    Match,

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Power,      // **
    EqEq,       // ==
    NotEq,      // !=
    Lt,         // <
    Gt,         // >
    LtEq,       // <=
    GtEq,       // >=
    Eq,         // =
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }
    Dot,        // .
    Comma,      // ,
    Colon,      // :
    Arrow,      // ->
    FStringStart,
    FStringEnd,
    FStringText(String),
    FStringExprStart,
    FStringExprEnd,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "IDENT({})", s),
            TokenKind::Number(n) => write!(f, "NUMBER({})", n),
            TokenKind::Float(val) => write!(f, "FLOAT({})", val),
            TokenKind::String(s) => write!(f, "STRING({:?})", s),
            TokenKind::Bool(b) => write!(f, "BOOL({})", b),
            TokenKind::Newline => write!(f, "NEWLINE"),
            TokenKind::Indent => write!(f, "INDENT"),
            TokenKind::Dedent => write!(f, "DEDENT"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Task => write!(f, "task"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Priv => write!(f, "priv"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Unsafe => write!(f, "unsafe"),
            TokenKind::Weak => write!(f, "weak"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Spawn => write!(f, "spawn"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Power => write!(f, "**"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::Eq => write!(f, "="),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FStringStart => write!(f, "FSTRING_START"),
            TokenKind::FStringEnd => write!(f, "FSTRING_END"),
            TokenKind::FStringText(s) => write!(f, "FSTRING_TEXT({:?})", s),
            TokenKind::FStringExprStart => write!(f, "FSTRING_EXPR_START"),
            TokenKind::FStringExprEnd => write!(f, "FSTRING_EXPR_END"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.kind, self.line, self.column)
    }
}
