//! n0ne Compiler - Semantic Analysis
//! Type checking, scoping, and semantic error reporting.

use ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub line: usize,
    pub column: usize,
    pub code: String,
    pub message: String,
    pub hint: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolInfo {
    Variable(Type),
    Function {
        params: Vec<Type>,
        ret_type: Option<Type>,
    },
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolInfo>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, info: SymbolInfo) -> bool {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            false
        } else {
            current_scope.insert(name, info);
            true
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn insert_global(&mut self, name: String, info: SymbolInfo) -> bool {
        let global_scope = &mut self.scopes[0];
        if global_scope.contains_key(&name) {
            false
        } else {
            global_scope.insert(name, info);
            true
        }
    }
}

pub struct TypeChecker {
    pub table: SymbolTable,
    pub errors: Vec<SemanticError>,
    current_fn_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = Self {
            table: SymbolTable::new(),
            errors: Vec::new(),
            current_fn_return_type: None,
        };
        tc.table.insert_global("show".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("c_alloc".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("int".to_string())],
            ret_type: Some(Type::Basic("unknown".to_string())),
        });
        tc.table.insert_global("c_store_int".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string()), Type::Basic("int".to_string()), Type::Basic("int".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("c_store_string".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string()), Type::Basic("int".to_string()), Type::Basic("string".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("c_load_int".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string()), Type::Basic("int".to_string())],
            ret_type: Some(Type::Basic("int".to_string())),
        });
        tc.table.insert_global("c_load_string".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string()), Type::Basic("int".to_string())],
            ret_type: Some(Type::Basic("string".to_string())),
        });
        tc.table.insert_global("c_interpolate".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("string".to_string()), Type::Basic("string".to_string())],
            ret_type: Some(Type::Basic("string".to_string())),
        });
        tc.table.insert_global("c_argc".to_string(), SymbolInfo::Function {
            params: vec![],
            ret_type: Some(Type::Basic("int".to_string())),
        });
        tc.table.insert_global("c_argv".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("int".to_string())],
            ret_type: Some(Type::Basic("string".to_string())),
        });
        tc
    }

    pub fn check_program(&mut self, prog: &Program) {
        // Pass 1: Forward declarations
        for decl in &prog.decls {
            match decl {
                TopLevelDecl::FnDecl(f) => {
                    let params = f.params.iter().map(|p| p.type_ann.clone()).collect();
                    let info = SymbolInfo::Function {
                        params,
                        ret_type: f.return_type.clone(),
                    };
                    if !self.table.insert_global(f.name.clone(), info) {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E012".to_string(),
                            message: format!("duplicate function name '{}'", f.name),
                            hint: "Ensure all top-level functions have unique names.".to_string(),
                        });
                    }
                }
                TopLevelDecl::TaskDecl(t) => {
                    let info = SymbolInfo::Function {
                        params: vec![],
                        ret_type: None,
                    };
                    if !self.table.insert_global(t.name.clone(), info) {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E012".to_string(),
                            message: format!("duplicate task name '{}'", t.name),
                            hint: "Ensure all tasks have unique names.".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        // Pass 2: Check bodies
        for decl in &prog.decls {
            match decl {
                TopLevelDecl::FnDecl(f) => self.check_fn_decl(f),
                TopLevelDecl::TaskDecl(t) => self.check_task_decl(t),
                TopLevelDecl::ConstDecl(c) => self.check_const_decl(c),
                _ => {}
            }
        }
    }

    fn check_fn_decl(&mut self, decl: &FnDecl) {
        self.current_fn_return_type = decl.return_type.clone();
        self.table.enter_scope();

        for param in &decl.params {
            self.table.insert(
                param.name.clone(),
                SymbolInfo::Variable(param.type_ann.clone()),
            );
        }

        self.check_block(&decl.body);

        // Check E006: missing return in non-void fn
        if decl.return_type.is_some() {
            if !self.has_return(&decl.body) {
                self.errors.push(SemanticError {
                    line: 0,
                    column: 0,
                    code: "E006".to_string(),
                    message: format!("missing return in non-void function '{}'", decl.name),
                    hint: "Add a return statement that matches the return type.".to_string(),
                });
            }
        }

        self.table.exit_scope();
        self.current_fn_return_type = None;
    }

    fn check_task_decl(&mut self, decl: &TaskDecl) {
        self.current_fn_return_type = None;
        self.table.enter_scope();
        self.check_block(&decl.body);
        self.table.exit_scope();
    }

    fn check_const_decl(&mut self, decl: &ConstDecl) {
        let t = self.infer_expr(&decl.value);
        self.table.insert_global(decl.name.clone(), SymbolInfo::Variable(t));
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, op, value } => {
                let rhs_type = self.infer_expr(value);

                if let AssignOp::Eq = op {
                    if let Expr::Ident(name) = target {
                        if self.table.lookup(name).is_none() {
                            self.table.insert(name.clone(), SymbolInfo::Variable(rhs_type.clone()));
                            return;
                        }
                    }
                }

                let lhs_type = self.infer_expr(target);

                if lhs_type != Type::Basic("unknown".to_string())
                    && rhs_type != Type::Basic("unknown".to_string())
                {
                    if lhs_type != rhs_type {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E001".to_string(),
                            message: format!(
                                "type mismatch: expected '{:?}', found '{:?}'",
                                lhs_type, rhs_type
                            ),
                            hint: "Ensure the assigned value matches the variable's type."
                                .to_string(),
                        });
                    }
                }
            }
            Stmt::ConstDecl(c) => {
                let t = self.infer_expr(&c.value);
                self.table.insert(c.name.clone(), SymbolInfo::Variable(t));
            }
            Stmt::If {
                cond,
                then_branch,
                elifs,
                else_branch,
            } => {
                let cond_type = self.infer_expr(cond);
                if cond_type != Type::Basic("bool".to_string())
                    && cond_type != Type::Basic("unknown".to_string())
                {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E001".to_string(),
                        message: format!(
                            "type mismatch: if condition expected 'bool', found '{:?}'",
                            cond_type
                        ),
                        hint: "Use a boolean expression in if condition.".to_string(),
                    });
                }

                self.table.enter_scope();
                self.check_block(then_branch);
                self.table.exit_scope();

                for (e_cond, e_block) in elifs {
                    let c_type = self.infer_expr(e_cond);
                    if c_type != Type::Basic("bool".to_string())
                        && c_type != Type::Basic("unknown".to_string())
                    {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E001".to_string(),
                            message: format!(
                                "type mismatch: elif condition expected 'bool', found '{:?}'",
                                c_type
                            ),
                            hint: "Use a boolean expression in elif condition.".to_string(),
                        });
                    }
                    self.table.enter_scope();
                    self.check_block(e_block);
                    self.table.exit_scope();
                }

                if let Some(eb) = else_branch {
                    self.table.enter_scope();
                    self.check_block(eb);
                    self.table.exit_scope();
                }
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                let iter_type = self.infer_expr(iterable);
                let var_type = match iter_type {
                    Type::List(inner) => *inner,
                    _ => Type::Basic("unknown".to_string()),
                };

                self.table.enter_scope();
                self.table.insert(var.clone(), SymbolInfo::Variable(var_type));
                self.check_block(body);
                self.table.exit_scope();
            }
            Stmt::Return(expr_opt) => {
                let actual = if let Some(e) = expr_opt {
                    self.infer_expr(e)
                } else {
                    Type::Basic("void".to_string())
                };

                if let Some(expected) = &self.current_fn_return_type {
                    if expected != &actual && actual != Type::Basic("unknown".to_string()) {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E001".to_string(),
                            message: format!(
                                "type mismatch: expected return type '{:?}', found '{:?}'",
                                expected, actual
                            ),
                            hint: "Return a value that matches the function's signature."
                                .to_string(),
                        });
                    }
                }
            }
            Stmt::Expr(expr) => {
                self.infer_expr(expr);
            }
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Ident(name) => match self.table.lookup(name).cloned() {
                Some(SymbolInfo::Variable(t)) => t,
                Some(SymbolInfo::Function { .. }) => Type::Basic("unknown".to_string()),
                None => {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E002".to_string(),
                        message: format!("undefined variable '{}'", name),
                        hint: "Declare the variable before using it.".to_string(),
                    });
                    Type::Basic("unknown".to_string())
                }
            },
            Expr::ListLiteral(items) => {
                let elem_type = if items.is_empty() {
                    Type::Basic("unknown".to_string())
                } else {
                    self.infer_expr(&items[0])
                };
                for item in items.iter().skip(1) {
                    self.infer_expr(item);
                }
                Type::List(Box::new(elem_type))
            }
            Expr::MapLiteral(pairs) => {
                let (key_type, val_type) = if pairs.is_empty() {
                    (Type::Basic("unknown".to_string()), Type::Basic("unknown".to_string()))
                } else {
                    (self.infer_expr(&pairs[0].0), self.infer_expr(&pairs[0].1))
                };
                for (k, v) in pairs.iter().skip(1) {
                    self.infer_expr(k);
                    self.infer_expr(v);
                }
                Type::Map(Box::new(key_type), Box::new(val_type))
            }
            Expr::FStringExpr(parts) => {
                for part in parts {
                    match part {
                        FStringPart::Text(_) => {}
                        FStringPart::Expr(expr) => {
                            self.infer_expr(expr);
                        }
                    }
                }
                Type::Basic("string".to_string())
            }
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Type::Basic("int".to_string()),
                Literal::Float(_) => Type::Basic("float".to_string()),
                Literal::String(_) => Type::Basic("string".to_string()),
                Literal::Bool(_) => Type::Basic("bool".to_string()),
            },
            Expr::BinExpr { left, op, right } => {
                let t1 = self.infer_expr(left);
                let _t2 = self.infer_expr(right);
                match op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge | BinOp::And | BinOp::Or => {
                        Type::Basic("bool".to_string())
                    }
                    _ => t1,
                }
            }
            Expr::UnaryExpr { op: _, expr: inner } => self.infer_expr(inner),
            Expr::CallExpr { callee, args } => {
                if let Expr::Ident(name) = &**callee {
                    if let Some(SymbolInfo::Function { params, ret_type }) =
                        self.table.lookup(name).cloned()
                    {
                        if args.len() != params.len() {
                            self.errors.push(SemanticError {
                                line: 0,
                                column: 0,
                                code: "E004".to_string(),
                                message: format!(
                                    "wrong argument count for function '{}': expected {}, found {}",
                                    name,
                                    params.len(),
                                    args.len()
                                ),
                                hint: "Pass the correct number of arguments.".to_string(),
                            });
                        } else {
                            for (i, arg) in args.iter().enumerate() {
                                let arg_type = self.infer_expr(arg);
                                let expected_type = &params[i];
                                if &arg_type != expected_type
                                    && arg_type != Type::Basic("unknown".to_string())
                                    && expected_type != &Type::Basic("unknown".to_string())
                                {
                                    self.errors.push(SemanticError {
                                        line: 0,
                                        column: 0,
                                        code: "E005".to_string(),
                                        message: format!(
                                            "wrong argument type in call to '{}': expected '{:?}', found '{:?}'",
                                            name, expected_type, arg_type
                                        ),
                                        hint: "Check the parameter types of the function."
                                            .to_string(),
                                    });
                                }
                            }
                        }
                        return ret_type.unwrap_or(Type::Basic("void".to_string()));
                    } else if self.table.lookup(name).is_none() {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E003".to_string(),
                            message: format!("undefined function '{}'", name),
                            hint: "Declare the function before calling it.".to_string(),
                        });
                        return Type::Basic("unknown".to_string());
                    }
                } else if let Expr::FieldAccess { expr: receiver, field: method_name } = &**callee {
                    let receiver_ty = self.infer_expr(receiver);
                    let mut arg_types = Vec::new();
                    for arg in args {
                        arg_types.push(self.infer_expr(arg));
                    }
                    match &receiver_ty {
                        Type::Basic(name) if name == "string" => {
                            match method_name.as_str() {
                                "len" => return Type::Basic("int".to_string()),
                                "contains" | "starts_with" | "ends_with" => return Type::Basic("bool".to_string()),
                                "upper" | "lower" | "trim" | "replace" | "slice" => return Type::Basic("string".to_string()),
                                "split" => return Type::List(Box::new(Type::Basic("string".to_string()))),
                                "to_int" => return Type::Option(Box::new(Type::Basic("int".to_string()))),
                                "to_float" => return Type::Option(Box::new(Type::Basic("float".to_string()))),
                                _ => {}
                            }
                        }
                        Type::Basic(name) if name == "int" => {
                            match method_name.as_str() {
                                "to_string" => return Type::Basic("string".to_string()),
                                "to_float" => return Type::Basic("float".to_string()),
                                _ => {}
                            }
                        }
                        Type::Basic(name) if name == "float" => {
                            match method_name.as_str() {
                                "to_int" => return Type::Basic("int".to_string()),
                                "to_string" => return Type::Basic("string".to_string()),
                                _ => {}
                            }
                        }
                        Type::List(inner) => {
                            match method_name.as_str() {
                                "len" => return Type::Basic("int".to_string()),
                                "push" => return Type::Basic("void".to_string()),
                                "pop" | "first" | "last" => return Type::Option(inner.clone()),
                                "contains" => return Type::Basic("bool".to_string()),
                                _ => {}
                            }
                        }
                        Type::Map(_key_ty, val_ty) => {
                            match method_name.as_str() {
                                "get" => return Type::Option(val_ty.clone()),
                                "set" | "delete" => return Type::Basic("void".to_string()),
                                "has" => return Type::Basic("bool".to_string()),
                                "keys" => return Type::List(Box::new(Type::Basic("string".to_string()))),
                                "values" => return Type::List(val_ty.clone()),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                for arg in args {
                    self.infer_expr(arg);
                }
                Type::Basic("unknown".to_string())
            }
            Expr::FieldAccess { expr: inner, field: _ } => {
                self.infer_expr(inner);
                Type::Basic("unknown".to_string())
            }
            Expr::TryExpr(inner) => {
                let inner_type = self.infer_expr(inner);
                match inner_type {
                    Type::Result(t) => *t,
                    _ => Type::Basic("unknown".to_string()),
                }
            }
        }
    }

    fn has_return(&self, block: &Block) -> bool {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Return(_) => return true,
                Stmt::If {
                    then_branch,
                    elifs,
                    else_branch,
                    ..
                } => {
                    let mut all_return = self.has_return(then_branch);
                    for (_, e_block) in elifs {
                        all_return = all_return && self.has_return(e_block);
                    }
                    if let Some(eb) = else_branch {
                        all_return = all_return && self.has_return(eb);
                    } else {
                        all_return = false;
                    }
                    if all_return {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(code: &str) -> Vec<SemanticError> {
        let tokens = lexer::Lexer::tokenize(code);
        let mut parser = parser::Parser::new(tokens);
        let prog = parser.parse();
        let mut checker = TypeChecker::new();
        checker.check_program(&prog);
        checker.errors
    }

    #[test]
    fn test_valid_program() {
        let code = "
fn add(a: int, b: int) -> int
    return a + b

task main
    x = 10
    y = 20
    z = add(x, y)
";
        let errors = check(code);
        assert!(errors.is_empty(), "Expected no errors, got {:?}", errors);
    }

    #[test]
    fn test_e001_type_mismatch() {
        let code = "
task main
    x = 10
    x = \"hello\"
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E001");
    }

    #[test]
    fn test_e002_undefined_variable() {
        let code = "
task main
    x = y + 1
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E002");
    }

    #[test]
    fn test_e003_undefined_function() {
        let code = "
task main
    foo()
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E003");
    }

    #[test]
    fn test_e004_wrong_argument_count() {
        let code = "
fn add(a: int, b: int) -> int
    return a + b

task main
    add(1)
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E004");
    }

    #[test]
    fn test_e005_wrong_argument_type() {
        let code = "
fn add(a: int, b: int) -> int
    return a + b

task main
    add(1, \"hello\")
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E005");
    }

    #[test]
    fn test_e006_missing_return() {
        let code = "
fn do_something() -> int
    x = 10
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E006");
    }

    #[test]
    fn test_e012_duplicate_function() {
        let code = "
fn add() -> int
    return 1

fn add() -> int
    return 2
";
        let errors = check(code);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E012");
    }
}
