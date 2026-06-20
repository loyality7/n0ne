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
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<Field>,
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
pub struct UseDecl {
    pub path: String,
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
    Return(Option<Expr>),
    Expr(Expr),
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
