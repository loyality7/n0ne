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

    pub fn update(&mut self, name: &str, info: SymbolInfo) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), info);
                return true;
            }
        }
        false
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

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub info: SymbolInfo,
}

impl Symbol {
    pub fn fn_sym(name: &str, params: Vec<Type>, ret_type: Type) -> Self {
        Self {
            name: name.to_string(),
            info: SymbolInfo::Function {
                params,
                ret_type: Some(ret_type),
            },
        }
    }
}

pub fn get_stdlib_symbols(module: &str) -> Option<Vec<Symbol>> {
    match module {
        "io" => Some(vec![
            Symbol::fn_sym("show", vec![Type::Basic("unknown".to_string())], Type::Basic("void".to_string())),
            Symbol::fn_sym("read", vec![], Type::Basic("string".to_string())),
            Symbol::fn_sym("show_err", vec![Type::Basic("unknown".to_string())], Type::Basic("void".to_string())),
        ]),
        "fs" => Some(vec![
            Symbol::fn_sym("read", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("string".to_string())))),
            Symbol::fn_sym("write", vec![Type::Basic("string".to_string()), Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("void".to_string())))),
            Symbol::fn_sym("exists", vec![Type::Basic("string".to_string())], Type::Basic("bool".to_string())),
            Symbol::fn_sym("delete", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("void".to_string())))),
            Symbol::fn_sym("mkdir", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("void".to_string())))),
            Symbol::fn_sym("list", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::List(Box::new(Type::Basic("string".to_string())))))),
        ]),
        "json" => Some(vec![
            Symbol::fn_sym("encode", vec![Type::Basic("unknown".to_string())], Type::Basic("string".to_string())),
            Symbol::fn_sym("decode", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::Map(Box::new(Type::Basic("string".to_string())), Box::new(Type::Basic("string".to_string())))))),
        ]),
        "http" => Some(vec![
            Symbol::fn_sym("get", vec![Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("string".to_string())))),
            Symbol::fn_sym("post", vec![Type::Basic("string".to_string()), Type::Basic("string".to_string())], Type::Result(Box::new(Type::Basic("string".to_string())))),
        ]),
        _ => None,
    }
}

fn types_match(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        (Type::Basic(e), _) if e == "unknown" => true,
        (_, Type::Basic(a)) if a == "unknown" => true,
        (Type::Basic(e), Type::Basic(a)) => e == a,
        (Type::List(e), Type::List(a)) => types_match(e, a),
        (Type::Map(ek, ev), Type::Map(ak, av)) => types_match(ek, ak) && types_match(ev, av),
        (Type::Result(e), Type::Result(a)) => types_match(e, a),
        (Type::Option(e), Type::Option(a)) => types_match(e, a),
        _ => false,
    }
}

pub struct TypeChecker {
    pub table: SymbolTable,
    pub errors: Vec<SemanticError>,
    current_fn_return_type: Option<Type>,
    pub imported_modules: HashMap<String, HashMap<String, SymbolInfo>>,
    pub import_stack: Vec<std::path::PathBuf>,
    pub current_file: Option<std::path::PathBuf>,
    inside_loop: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = Self {
            table: SymbolTable::new(),
            errors: Vec::new(),
            current_fn_return_type: None,
            imported_modules: HashMap::new(),
            import_stack: Vec::new(),
            current_file: None,
            inside_loop: 0,
        };
        tc.table.insert_global("show".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("ok".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: Some(Type::Result(Box::new(Type::Basic("unknown".to_string())))),
        });
        tc.table.insert_global("err".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("string".to_string())],
            ret_type: Some(Type::Result(Box::new(Type::Basic("unknown".to_string())))),
        });
        tc.table.insert_global("some".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: Some(Type::Option(Box::new(Type::Basic("unknown".to_string())))),
        });
        tc.table.insert_global("none".to_string(), SymbolInfo::Function {
            params: vec![],
            ret_type: Some(Type::Option(Box::new(Type::Basic("unknown".to_string())))),
        });
        tc.table.insert_global("print".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("show_err".to_string(), SymbolInfo::Function {
            params: vec![Type::Basic("unknown".to_string())],
            ret_type: None,
        });
        tc.table.insert_global("print_err".to_string(), SymbolInfo::Function {
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
        // Pass 0: Handle imports (UseDecl)
        for decl in &prog.decls {
            if let TopLevelDecl::UseDecl(u) = decl {
                self.check_use_decl(u);
            }
        }

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
                TopLevelDecl::TypeDecl(t) => {
                    let info = SymbolInfo::Function {
                        params: vec![],
                        ret_type: Some(Type::Basic(t.name.clone())),
                    };
                    self.table.insert_global(t.name.clone(), info);
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

    fn get_module_name(&self, path: &str) -> String {
        std::path::Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn resolve_local_path(&self, import_path: &str) -> Result<std::path::PathBuf, std::io::Error> {
        let base_dir = if let Some(cf) = &self.current_file {
            cf.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::env::current_dir().unwrap())
        } else {
            std::env::current_dir()?
        };
        
        let file_name = if import_path.ends_with(".n0") {
            import_path.to_string()
        } else {
            format!("{}.n0", import_path)
        };

        let resolved = base_dir.join(file_name);
        match resolved.canonicalize() {
            Ok(p) => Ok(p),
            Err(_) => Ok(resolved),
        }
    }

    fn check_use_decl(&mut self, u: &UseDecl) {
        let module_name = self.get_module_name(&u.path);
        match u.kind {
            UseKind::Stdlib => {
                if let Some(symbols) = get_stdlib_symbols(&u.path) {
                    let mut mod_symbols = HashMap::new();
                    for sym in symbols {
                        mod_symbols.insert(sym.name.clone(), sym.info.clone());
                        if let Some(items) = &u.items {
                            if items.contains(&sym.name) {
                                self.table.insert_global(sym.name.clone(), sym.info.clone());
                            }
                        }
                    }
                    self.imported_modules.insert(module_name, mod_symbols);
                } else {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E010".to_string(),
                        message: format!("unknown module '{}'", u.path),
                        hint: "available stdlib modules: io fs json http math".to_string(),
                    });
                }
            }
            UseKind::Local => {
                let resolved_path = match self.resolve_local_path(&u.path) {
                    Ok(p) => p,
                    Err(_) => {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E011".to_string(),
                            message: format!("local module file not found at '{}'", u.path),
                            hint: "Verify the file path exists.".to_string(),
                        });
                        return;
                    }
                };

                if !resolved_path.exists() {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E011".to_string(),
                        message: format!("local module file not found at '{}'", resolved_path.display()),
                        hint: "Verify the file path exists relative to the importing file.".to_string(),
                    });
                    return;
                }

                if self.import_stack.contains(&resolved_path) {
                    let mut path_names: Vec<String> = self.import_stack.iter()
                        .map(|p| p.file_stem().unwrap_or_default().to_string_lossy().to_string())
                        .collect();
                    path_names.push(resolved_path.file_stem().unwrap_or_default().to_string_lossy().to_string());
                    let circular_chain = path_names.join(" -> ");
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E009".to_string(),
                        message: format!("circular import detected {}", circular_chain),
                        hint: "Remove the circular dependency chain.".to_string(),
                    });
                    return;
                }

                let content = match std::fs::read_to_string(&resolved_path) {
                    Ok(c) => c,
                    Err(e) => {
                        self.errors.push(SemanticError {
                            line: 0,
                            column: 0,
                            code: "E011".to_string(),
                            message: format!("failed to read local module file '{}': {}", resolved_path.display(), e),
                            hint: "Check file permissions.".to_string(),
                        });
                        return;
                    }
                };

                let tokens = lexer::Lexer::tokenize(&content);
                let mut parser = parser::Parser::new(tokens);
                let sub_prog = parser.parse();

                let mut sub_checker = TypeChecker::new();
                sub_checker.current_file = Some(resolved_path.clone());
                sub_checker.import_stack = self.import_stack.clone();
                sub_checker.import_stack.push(resolved_path.clone());
                sub_checker.imported_modules = self.imported_modules.clone();

                sub_checker.check_program(&sub_prog);

                let mut mod_symbols = HashMap::new();
                if let Some(global_scope) = sub_checker.table.scopes.first() {
                    for (name, info) in global_scope {
                        if name != "show" && !name.starts_with("c_") {
                            mod_symbols.insert(name.clone(), info.clone());
                            if let Some(items) = &u.items {
                                if items.contains(name) {
                                    self.table.insert_global(name.clone(), info.clone());
                                }
                            }
                        }
                    }
                }
                
                self.imported_modules.insert(module_name, mod_symbols);
                self.errors.extend(sub_checker.errors);
            }
            UseKind::Package => {
                self.errors.push(SemanticError {
                    line: 0,
                    column: 0,
                    code: "E010".to_string(),
                    message: format!("package imports are not supported yet: '{}'", u.path),
                    hint: "Use stdlib or local imports.".to_string(),
                });
            }
        }
    }

    fn check_fn_decl(&mut self, decl: &FnDecl) {
        self.current_fn_return_type = decl.return_type.clone();
        self.table.enter_scope();

        if let Some(recv) = &decl.receiver {
            self.table.insert(
                "self".to_string(),
                SymbolInfo::Variable(Type::Basic(recv.type_name.clone())),
            );
        }

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
        let mut returned = false;
        for stmt in &block.stmts {
            if returned {
                self.errors.push(SemanticError {
                    line: 0,
                    column: 0,
                    code: "E007".to_string(),
                    message: "unreachable code detected after return".to_string(),
                    hint: "Remove or move the statements after return.".to_string(),
                });
            }
            self.check_stmt(stmt);
            if let Stmt::Return(_) = stmt {
                returned = true;
            }
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
                    if !types_match(&lhs_type, &rhs_type) {
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

                self.inside_loop += 1;
                self.table.enter_scope();
                self.table.insert(var.clone(), SymbolInfo::Variable(var_type));
                self.check_block(body);
                self.table.exit_scope();
                self.inside_loop -= 1;
            }
            Stmt::While { cond, body } => {
                let cond_type = self.infer_expr(cond);
                if cond_type != Type::Basic("bool".to_string())
                    && cond_type != Type::Basic("unknown".to_string())
                {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E001".to_string(),
                        message: format!(
                            "type mismatch: while condition expected 'bool', found '{:?}'",
                            cond_type
                        ),
                        hint: "Use a boolean expression in while condition.".to_string(),
                    });
                }

                self.inside_loop += 1;
                self.table.enter_scope();
                self.check_block(body);
                self.table.exit_scope();
                self.inside_loop -= 1;
            }
            Stmt::Break => {
                if self.inside_loop == 0 {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E014".to_string(),
                        message: "break statement outside of loop".to_string(),
                        hint: "break can only be used inside for or while loops.".to_string(),
                    });
                }
            }
            Stmt::Continue => {
                if self.inside_loop == 0 {
                    self.errors.push(SemanticError {
                        line: 0,
                        column: 0,
                        code: "E015".to_string(),
                        message: "continue statement outside of loop".to_string(),
                        hint: "continue can only be used inside for or while loops.".to_string(),
                    });
                }
            }
            Stmt::Match { expr, cases } => {
                let expr_type = self.infer_expr(expr);
                for (arm, body) in cases {
                    match arm {
                        MatchArm::Literal(lit) => {
                            let arm_type = match lit {
                                Literal::Int(_) => Type::Basic("int".to_string()),
                                Literal::Float(_) => Type::Basic("float".to_string()),
                                Literal::String(_) => Type::Basic("string".to_string()),
                                Literal::Bool(_) => Type::Basic("bool".to_string()),
                            };
                            if !types_match(&arm_type, &expr_type) && expr_type != Type::Basic("unknown".to_string()) {
                                self.errors.push(SemanticError {
                                    line: 0,
                                    column: 0,
                                    code: "E001".to_string(),
                                    message: format!(
                                        "type mismatch: match pattern type '{:?}' does not match expression type '{:?}'",
                                        arm_type, expr_type
                                    ),
                                    hint: "All match patterns must match the type of the matched expression.".to_string(),
                                });
                            }
                        }
                        MatchArm::Wildcard => {}
                    }
                    self.table.enter_scope();
                    self.check_block(body);
                    self.table.exit_scope();
                }
            }
            Stmt::Return(expr_opt) => {
                let actual = if let Some(e) = expr_opt {
                    self.infer_expr(e)
                } else {
                    Type::Basic("void".to_string())
                };

                if let Some(expected) = &self.current_fn_return_type {
                    if !types_match(expected, &actual) && actual != Type::Basic("unknown".to_string()) {
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
                                 if !types_match(expected_type, &arg_type)
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
                        let mut resolved_ret_type = ret_type.clone().unwrap_or(Type::Basic("void".to_string()));
                        if name == "some" && !args.is_empty() {
                            let arg_type = self.infer_expr(&args[0]);
                            resolved_ret_type = Type::Option(Box::new(arg_type));
                        } else if name == "ok" && !args.is_empty() {
                            let arg_type = self.infer_expr(&args[0]);
                            resolved_ret_type = Type::Result(Box::new(arg_type));
                        }
                        return resolved_ret_type;
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
                    if let Expr::Ident(mod_name) = &**receiver {
                        let fn_sig = self.imported_modules.get(mod_name)
                            .and_then(|mod_syms| mod_syms.get(method_name))
                            .cloned();
                        if let Some(SymbolInfo::Function { params, ret_type }) = fn_sig {
                            if args.len() != params.len() {
                                self.errors.push(SemanticError {
                                    line: 0,
                                    column: 0,
                                    code: "E004".to_string(),
                                    message: format!(
                                        "wrong argument count for function '{}.{}': expected {}, found {}",
                                        mod_name, method_name, params.len(), args.len()
                                    ),
                                    hint: "Pass the correct number of arguments.".to_string(),
                                });
                            } else {
                                for (i, arg) in args.iter().enumerate() {
                                    let arg_type = self.infer_expr(arg);
                                    let expected_type = &params[i];
                                    if !types_match(expected_type, &arg_type)
                                        && arg_type != Type::Basic("unknown".to_string())
                                        && expected_type != &Type::Basic("unknown".to_string())
                                    {
                                        self.errors.push(SemanticError {
                                            line: 0,
                                            column: 0,
                                            code: "E005".to_string(),
                                            message: format!(
                                                "wrong argument type in call to '{}.{}': expected '{:?}', found '{:?}'",
                                                mod_name, method_name, expected_type, arg_type
                                            ),
                                            hint: "Check the parameter types of the function.".to_string(),
                                        });
                                    }
                                }
                            }
                            return ret_type.clone().unwrap_or(Type::Basic("void".to_string()));
                        }
                    }

                    let mut receiver_ty = self.infer_expr(receiver);
                    let mut arg_types = Vec::new();
                    for arg in args {
                        arg_types.push(self.infer_expr(arg));
                    }
                    if method_name == "set" && arg_types.len() == 2 {
                        if let Expr::Ident(var_name) = &**receiver {
                            if let Type::Map(k_ty, v_ty) = &receiver_ty {
                                if **k_ty == Type::Basic("unknown".to_string()) || **v_ty == Type::Basic("unknown".to_string()) {
                                    let new_k = if **k_ty == Type::Basic("unknown".to_string()) {
                                        arg_types[0].clone()
                                    } else {
                                        (**k_ty).clone()
                                    };
                                    let new_v = if **v_ty == Type::Basic("unknown".to_string()) {
                                        arg_types[1].clone()
                                    } else {
                                        (**v_ty).clone()
                                    };
                                    let refined_ty = Type::Map(Box::new(new_k), Box::new(new_v));
                                    self.table.update(var_name, SymbolInfo::Variable(refined_ty.clone()));
                                    receiver_ty = refined_ty;
                                }
                            }
                        }
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
                        Type::Basic(name) if name == "bool" => {
                            match method_name.as_str() {
                                "to_string" => return Type::Basic("string".to_string()),
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
                        Type::Option(inner) => {
                            match method_name.as_str() {
                                "unwrap" => return *inner.clone(),
                                "is_some" | "is_none" => return Type::Basic("bool".to_string()),
                                _ => {}
                            }
                        }
                        Type::Result(inner) => {
                            match method_name.as_str() {
                                "unwrap" => return *inner.clone(),
                                "is_ok" | "is_err" => return Type::Basic("bool".to_string()),
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
            Expr::FieldAccess { expr: inner, field } => {
                if let Expr::Ident(mod_name) = &**inner {
                    if let Some(mod_syms) = self.imported_modules.get(mod_name) {
                        if let Some(sym_info) = mod_syms.get(field) {
                            return match sym_info {
                                SymbolInfo::Variable(t) => t.clone(),
                                SymbolInfo::Function { ret_type, .. } => ret_type.clone().unwrap_or(Type::Basic("void".to_string())),
                            };
                        }
                    }
                }
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
