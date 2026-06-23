//! n0ne Compiler - AST Node definitions.
//! Contains data structures representing the Abstract Syntax Tree (AST).

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub decls: Vec<TopLevelDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDecl {
    FnDecl(FnDecl),
    TypeDecl(TypeDecl),
    TypeAliasDecl(TypeAliasDecl),
    EnumDecl(EnumDecl),
    TaskDecl(TaskDecl),
    UseDecl(UseDecl),
    ConstDecl(ConstDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub receiver: Option<Receiver>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Receiver {
    pub name: String,      // e.g. "self"
    pub type_name: String, // e.g. "User"
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: Type,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDecl {
    pub name: String,
    pub target_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_ann: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskDecl {
    pub name: String,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
    Stdlib,
    Local,
    Package,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub path: String,
    pub kind: UseKind,
    pub items: Option<Vec<String>>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Basic(String),             // e.g. "int", "float", "string", "bool", or custom "User"
    List(Box<Type>),           // list[T]
    Map(Box<Type>, Box<Type>), // map[K, V]
    Result(Box<Type>),         // result[T]
    Option(Box<Type>),         // option[T]
    Tuple(Vec<Type>),          // (T1, T2, ...)
    Function(Vec<Type>, Box<Type>), // fn(T1, T2) -> R
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
        target: Expr,
        op: AssignOp,
        value: Expr,
    },
    ConstDecl(ConstDecl),
    If {
        cond: Expr,
        then_branch: Block,
        elifs: Vec<(Expr, Block)>,
        else_branch: Option<Block>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Block,
    },
    While {
        cond: Expr,
        body: Block,
    },
    Break,
    Continue,
    Match {
        expr: Expr,
        cases: Vec<(MatchArm, Block)>,
    },
    Return(Option<Expr>),
    Expr(Expr),
    Defer(Expr),
    Guard {
        cond: Expr,
        else_branch: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchArm {
    Literal(Literal),
    Variant {
        variant_name: String,
        bindings: Vec<String>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Eq,      // =
    PlusEq,  // +=
    MinusEq, // -=
    StarEq,  // *=
    SlashEq, // /=
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Literal(Literal),
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    FStringExpr(Vec<FStringPart>),
    BinExpr {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        line: usize,
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    CallExpr {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    TryExpr(Box<Expr>),
    Tuple(Vec<Expr>),          // (e1, e2, ...)
    AnonymousFn {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Block,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
        line: usize,
    },
    NamedArg {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Pow, // **
    Eq,  // ==
    Ne,  // !=
    Lt,  // <
    Gt,  // >
    Le,  // <=
    Ge,  // >=
    And, // and
    Or,  // or
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
}
