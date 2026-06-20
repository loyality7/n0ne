//! n0ne Compiler - Formatter
//! Walks the AST and re-emits canonical n0ne source code.

use ast::*;
use lexer::Lexer;
use parser::Parser;

pub struct Formatter {
    out: String,
    indent_level: usize,
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            out: String::new(),
            indent_level: 0,
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.out.push_str("    ");
        }
    }

    fn emit(&mut self, s: &str) {
        self.push_indent();
        self.out.push_str(s);
    }

    fn newline(&mut self) {
        self.out.push('\n');
    }

    pub fn format_program(&mut self, ast: &Program) -> String {
        self.out.clear();
        self.indent_level = 0;

        for (i, decl) in ast.decls.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.format_top_level(decl);
        }

        if !self.out.ends_with('\n') {
            self.newline();
        }

        self.out.clone()
    }

    fn format_top_level(&mut self, decl: &TopLevelDecl) {
        match decl {
            TopLevelDecl::FnDecl(f) => self.format_fn(f),
            TopLevelDecl::TaskDecl(t) => self.format_task(t),
            TopLevelDecl::TypeDecl(t) => self.format_type(t),
            TopLevelDecl::ConstDecl(c) => {
                self.emit(&format!("const {} = ", c.name));
                self.format_expr(&c.value);
                self.newline();
            }
            TopLevelDecl::UseDecl(u) => {
                self.emit(&format!("use {}", u.path));
                self.newline();
            }
        }
    }

    fn type_to_str(&self, ty: &Type) -> String {
        match ty {
            Type::Basic(name) => name.clone(),
            Type::List(inner) => format!("list[{}]", self.type_to_str(inner)),
            Type::Map(k, v) => format!("map[{}, {}]", self.type_to_str(k), self.type_to_str(v)),
            Type::Result(inner) => format!("result[{}]", self.type_to_str(inner)),
            Type::Option(inner) => format!("option[{}]", self.type_to_str(inner)),
        }
    }

    fn format_fn(&mut self, f: &FnDecl) {
        self.push_indent();
        self.out.push_str("fn ");

        if let Some(receiver) = &f.receiver {
            self.out.push_str(&format!("({}: {}) ", receiver.name, receiver.type_name));
        }

        self.out.push_str(&f.name);
        self.out.push('(');

        for (i, param) in f.params.iter().enumerate() {
            if i > 0 {
                self.out.push_str(", ");
            }
            self.out.push_str(&format!("{}: {}", param.name, self.type_to_str(&param.type_ann)));
        }

        self.out.push(')');

        if let Some(rt) = &f.return_type {
            self.out.push_str(&format!(" -> {}", self.type_to_str(rt)));
        }

        self.newline();
        self.format_block(&f.body);
    }

    fn format_task(&mut self, t: &TaskDecl) {
        self.emit(&format!("task {}", t.name));
        self.newline();
        self.format_block(&t.body);
    }

    fn format_type(&mut self, t: &TypeDecl) {
        self.emit(&format!("type {}", t.name));
        self.newline();
        self.indent_level += 1;
        for field in &t.fields {
            self.emit(&format!("{}: {}", field.name, self.type_to_str(&field.type_ann)));
            self.newline();
        }
        self.indent_level -= 1;
    }

    fn format_block(&mut self, b: &Block) {
        self.indent_level += 1;
        for stmt in &b.stmts {
            self.format_stmt(stmt);
        }
        self.indent_level -= 1;
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, op, value } => {
                self.push_indent();
                self.format_expr(target);
                let op_str = match op {
                    AssignOp::Eq => "=",
                    AssignOp::PlusEq => "+=",
                    AssignOp::MinusEq => "-=",
                    AssignOp::StarEq => "*=",
                    AssignOp::SlashEq => "/=",
                };
                self.out.push_str(&format!(" {} ", op_str));
                self.format_expr(value);
                self.newline();
            }
            Stmt::ConstDecl(c) => {
                self.emit(&format!("const {} = ", c.name));
                self.format_expr(&c.value);
                self.newline();
            }
            Stmt::If {
                cond,
                then_branch,
                elifs,
                else_branch,
            } => {
                self.push_indent();
                self.out.push_str("if ");
                self.format_expr(cond);
                self.newline();
                self.format_block(then_branch);

                for (e_cond, e_block) in elifs {
                    self.push_indent();
                    self.out.push_str("elif ");
                    self.format_expr(e_cond);
                    self.newline();
                    self.format_block(e_block);
                }

                if let Some(eb) = else_branch {
                    self.emit("else");
                    self.newline();
                    self.format_block(eb);
                }
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                self.push_indent();
                self.out.push_str(&format!("for {} in ", var));
                self.format_expr(iterable);
                self.newline();
                self.format_block(body);
            }
            Stmt::Return(opt_val) => {
                self.push_indent();
                self.out.push_str("return");
                if let Some(v) = opt_val {
                    self.out.push(' ');
                    self.format_expr(v);
                }
                self.newline();
            }
            Stmt::Expr(e) => {
                self.push_indent();
                self.format_expr(e);
                self.newline();
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => self.out.push_str(name),
            Expr::Literal(lit) => match lit {
                Literal::Int(i) => self.out.push_str(&i.to_string()),
                Literal::Float(f) => {
                    let mut s = f.to_string();
                    if !s.contains('.') {
                        s.push_str(".0");
                    }
                    self.out.push_str(&s);
                }
                Literal::String(s) => self.out.push_str(&format!("\"{}\"", s.replace('\"', "\\\"").replace('\n', "\\n"))),
                Literal::Bool(b) => self.out.push_str(if *b { "true" } else { "false" }),
            },
            Expr::ListLiteral(items) => {
                self.out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.format_expr(item);
                }
                self.out.push(']');
            }
            Expr::MapLiteral(pairs) => {
                self.out.push('{');
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.format_expr(key);
                    self.out.push_str(": ");
                    self.format_expr(value);
                }
                self.out.push('}');
            }
            Expr::FStringExpr(parts) => {
                self.out.push_str("f\"");
                for part in parts {
                    match part {
                        FStringPart::Text(text) => {
                            let escaped = text
                                .replace('\\', "\\\\")
                                .replace('\"', "\\\"")
                                .replace('{', "\\{")
                                .replace('}', "\\}")
                                .replace('\n', "\\n")
                                .replace('\r', "\\r")
                                .replace('\t', "\\t");
                            self.out.push_str(&escaped);
                        }
                        FStringPart::Expr(expr) => {
                            self.out.push('{');
                            self.format_expr(expr);
                            self.out.push('}');
                        }
                    }
                }
                self.out.push('\"');
            }
            Expr::BinExpr { left, op, right } => {
                self.format_expr(left);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Pow => "**",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Le => "<=",
                    BinOp::Ge => ">=",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                };
                self.out.push_str(&format!(" {} ", op_str));
                self.format_expr(right);
            }
            Expr::UnaryExpr { op, expr: inner } => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "not ",
                };
                self.out.push_str(op_str);
                self.format_expr(inner);
            }
            Expr::CallExpr { callee, args } => {
                self.format_expr(callee);
                self.out.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.format_expr(arg);
                }
                self.out.push(')');
            }
            Expr::FieldAccess { expr: inner, field } => {
                self.format_expr(inner);
                self.out.push('.');
                self.out.push_str(field);
            }
            Expr::TryExpr(inner) => {
                self.out.push_str("try ");
                self.format_expr(inner);
            }
        }
    }
}

pub fn format(source: &str) -> String {
    let tokens = Lexer::tokenize(source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut formatter = Formatter::new();
    formatter.format_program(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_messy_spaces_and_blank_lines() {
        // Valid indentations but messy inline spacing and tons of blank lines
        let input = "task main\n\n    x    =    1+2\n\n\n    show( x ,  y )\n";
        let expected = "task main\n    x = 1 + 2\n    show(x, y)\n";
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_type_decl() {
        let input = "type User\n    name : string\n    age : int\n";
        let expected = "type User\n    name: string\n    age: int\n";
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_fn_args() {
        let input = "fn add( a :int,b: int ) -> int\n    return a+b\n";
        let expected = "fn add(a: int, b: int) -> int\n    return a + b\n";
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_list_and_map() {
        let input = "task main\n    items = [ 1, 2 , 3 ]\n    data = { \"key\" : \"value\" , 10 : 20 }\n";
        let expected = "task main\n    items = [1, 2, 3]\n    data = {\"key\": \"value\", 10: 20}\n";
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_fstring() {
        let input = "task main\n    msg = f\"hello { name } you are { age } years old\"\n";
        let expected = "task main\n    msg = f\"hello {name} you are {age} years old\"\n";
        assert_eq!(format(input), expected);
    }
}
