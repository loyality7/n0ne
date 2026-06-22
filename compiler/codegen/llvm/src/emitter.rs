use ast::{TopLevelDecl, FnDecl, TaskDecl, ConstDecl, Block, Expr, Literal, Stmt, Type};
use crate::LLVMGenerator;

pub(crate) fn block_has_return(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Return(_) => true,
        Stmt::If { then_branch, elifs, else_branch, .. } => {
            block_has_return(then_branch)
                && elifs.iter().all(|(_, b)| block_has_return(b))
                && else_branch.as_ref().map_or(false, |b| block_has_return(b))
        }
        _ => false,
    })
}

pub(crate) fn block_has_terminator(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Return(_) | Stmt::Break | Stmt::Continue => true,
        Stmt::If { then_branch, elifs, else_branch, .. } => {
            block_has_terminator(then_branch)
                && elifs.iter().all(|(_, b)| block_has_terminator(b))
                && else_branch.as_ref().map_or(false, |b| block_has_terminator(b))
        }
        _ => false,
    })
}

impl LLVMGenerator {
    pub(crate) fn next_reg(&mut self) -> String {
        self.reg_counter += 1;
        format!("%t{}", self.reg_counter)
    }

    pub(crate) fn next_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}_{}", prefix, self.label_counter)
    }

    pub(crate) fn escape_llvm_string(&self, s: &str) -> (String, usize) {
        let mut escaped = String::new();
        let mut len = 0;
        for c in s.chars() {
            match c {
                '\n' => {
                    escaped.push_str("\\0A");
                    len += 1;
                }
                '\r' => {
                    escaped.push_str("\\0D");
                    len += 1;
                }
                '\t' => {
                    escaped.push_str("\\09");
                    len += 1;
                }
                '\\' => {
                    escaped.push_str("\\5C");
                    len += 1;
                }
                '"' => {
                    escaped.push_str("\\22");
                    len += 1;
                }
                _ => {
                    escaped.push(c);
                    len += 1;
                }
            }
        }
        escaped.push_str("\\00");
        len += 1;
        (escaped, len)
    }

    pub(crate) fn get_field_offset(&self, var_type_name: &str, field_name: &str) -> i64 {
        if var_type_name == "result" {
            match field_name {
                "is_err" | "is_ok" => return 8,
                "value" | "unwrap" => return 16,
                "error" => return 24,
                _ => {}
            }
        }
        if var_type_name == "option" {
            match field_name {
                "is_some" => return 8,
                "is_none" => return 16,
                "value" | "unwrap" => return 24,
                _ => {}
            }
        }
        if let Some(decl) = self.structs.get(var_type_name) {
            let mut offset = 8;
            for field in &decl.fields {
                if field.name == field_name {
                    return offset;
                }
                offset += 8;
            }
        }
        8
    }

    pub(crate) fn add_string_constant(&mut self, s: &str) -> String {
        let name = format!("@.str.{}", self.string_counter);
        self.string_counter += 1;
        let (escaped, len) = self.escape_llvm_string(s);
        self.string_constants.push((name.clone(), escaped, len));
        name
    }

    pub(crate) fn gen_top_level(&mut self, decl: &TopLevelDecl) {
        match decl {
            TopLevelDecl::FnDecl(f) => self.gen_fn(f),
            TopLevelDecl::TaskDecl(t) => self.gen_task(t),
            TopLevelDecl::ConstDecl(c) => self.gen_const(c),
            TopLevelDecl::UseDecl(u) => {
                if let ast::UseKind::Local = u.kind {
                    let resolved_path = self.resolve_local_path(&u.path);
                    if !self.compiled_files.contains(&resolved_path) {
                        self.compiled_files.insert(resolved_path.clone());
                        if let Ok(content) = std::fs::read_to_string(&resolved_path) {
                            let tokens = lexer::Lexer::tokenize(&content);
                            let mut parser = parser::Parser::new(tokens);
                            let sub_prog = parser.parse();

                            // Save current file and set sub file
                            let old_file = self.current_file.clone();
                            self.current_file = Some(resolved_path);

                            // Collect structs, function return types, and global constants
                            for sub_decl in &sub_prog.decls {
                                match sub_decl {
                                    ast::TopLevelDecl::TypeDecl(t) => {
                                        self.structs.insert(t.name.clone(), t.clone());
                                    }
                                    ast::TopLevelDecl::FnDecl(f) => {
                                        self.functions.insert(f.name.clone(), f.clone());
                                    }
                                    ast::TopLevelDecl::TypeAliasDecl(a) => {
                                        self.aliases.insert(a.name.clone(), a.target_type.clone());
                                    }
                                    ast::TopLevelDecl::ConstDecl(c) => {
                                        let val_ty = self.infer_expr_type(&c.value);
                                        self.global_consts.insert(c.name.clone(), val_ty);
                                    }
                                    _ => {}
                                }
                            }

                            // Generate declarations recursively
                            for sub_decl in &sub_prog.decls {
                                self.gen_top_level(sub_decl);
                            }

                            // Restore
                            self.current_file = old_file;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn resolve_local_path(&self, import_path: &str) -> std::path::PathBuf {
        let base_dir = if let Some(cf) = &self.current_file {
            cf.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::env::current_dir().unwrap())
        } else {
            std::env::current_dir().unwrap()
        };
        let file_name = if import_path.ends_with(".n0") {
            import_path.to_string()
        } else {
            format!("{}.n0", import_path)
        };
        let resolved = base_dir.join(file_name);
        resolved.canonicalize().unwrap_or(resolved)
    }   

    pub(crate) fn gen_fn(&mut self, f: &FnDecl) {
        self.variables.clear();
        self.reg_counter = 0;

        let ret_ty = if let Some(rt) = &f.return_type {
            self.llvm_type(rt)
        } else {
            "void".to_string()
        };
        self.current_ret_type = ret_ty.clone();

        let mut params_str = Vec::new();
        if let Some(rec) = &f.receiver {
            params_str.push(format!("{} %{}", self.llvm_type(&Type::Basic(rec.type_name.clone())), rec.name));
        }
        for param in &f.params {
            params_str.push(format!("{} %{}", self.llvm_type(&param.type_ann), param.name));
        }

        if f.name == "main" {
            self.current_ret_type = "i32".to_string();
            self.body.push_str(
                "define i32 @main(i32 %argc, ptr %argv) {\nentry:\n"
            );
            self.body.push_str("    store i32 %argc, ptr @global_argc, align 4\n");
            self.body.push_str("    store ptr %argv, ptr @global_argv, align 8\n");
        } else {
            self.body.push_str(&format!(
                "define {} @n0_{}({}) {{\nentry:\n",
                ret_ty,
                f.name,
                params_str.join(", ")
            ));
        }

        if f.name != "main" {
            if let Some(rec) = &f.receiver {
                let ty = Type::Basic(rec.type_name.clone());
                let ty_str = self.llvm_type(&ty);
                self.body.push_str(&format!(
                    "    %_{} = alloca {}, align 8\n",
                    rec.name, ty_str
                ));
                self.body.push_str(&format!(
                    "    store {} %{}, ptr %_{}, align 8\n",
                    ty_str, rec.name, rec.name
                ));
                self.variables.insert(
                    rec.name.clone(),
                    (format!("%_{}", rec.name), ty),
                );
            }
            for param in &f.params {
                let ty_str = self.llvm_type(&param.type_ann);
                self.body.push_str(&format!(
                    "    %_{} = alloca {}, align 8\n",
                    param.name, ty_str
                ));
                self.body.push_str(&format!(
                    "    store {} %{}, ptr %_{}, align 8\n",
                    ty_str, param.name, param.name
                ));
                self.variables.insert(
                    param.name.clone(),
                    (format!("%_{}", param.name), param.type_ann.clone()),
                );
            }
        }

        self.gen_block(&f.body);

        if !block_has_return(&f.body) {
            if f.name == "main" {
                self.body.push_str("    ret i32 0\n");
            } else if ret_ty == "void" {
                self.body.push_str("    ret void\n");
            } else if ret_ty == "double" {
                self.body.push_str("    ret double 0.0\n");
            } else if ret_ty == "ptr" {
                self.body.push_str("    ret ptr null\n");
            } else {
                self.body.push_str("    ret i64 0\n");
            }
        }

        self.body.push_str("}\n\n");
    }

    pub(crate) fn gen_task(&mut self, t: &TaskDecl) {
        self.variables.clear();
        self.reg_counter = 0;

        if t.name == "main" {
            self.current_ret_type = "i32".to_string();
            self.body.push_str(
                "define i32 @main(i32 %argc, ptr %argv) {\nentry:\n"
            );
            self.body.push_str("    store i32 %argc, ptr @global_argc, align 4\n");
            self.body.push_str("    store ptr %argv, ptr @global_argv, align 8\n");
        } else {
            self.current_ret_type = "void".to_string();
            self.body.push_str(&format!(
                "define void @n0_task_{}() {{\nentry:\n",
                t.name
            ));
        }

        self.gen_block(&t.body);

        if !block_has_return(&t.body) {
            if t.name == "main" {
                self.body.push_str("    ret i32 0\n");
            } else {
                self.body.push_str("    ret void\n");
            }
        }

        self.body.push_str("}\n\n");
    }

    pub(crate) fn gen_const(&mut self, c: &ConstDecl) {
        let val_ty = self.infer_expr_type(&c.value);
        let val_llvm_ty = self.llvm_type(&val_ty);
        let init_val = match &c.value {
            Expr::Literal(Literal::Int(i)) => i.to_string(),
            Expr::Literal(Literal::Float(f)) => {
                let mut s = f.to_string();
                if !s.contains('.') {
                    s.push_str(".0");
                }
                s
            }
            Expr::Literal(Literal::Bool(b)) => if *b { "1".to_string() } else { "0".to_string() },
            Expr::Literal(Literal::String(s)) => {
                let name = self.add_string_constant(s);
                let len = self.string_constants.last().unwrap().2;
                format!("getelementptr inbounds ([{} x i8], ptr {}, i64 0, i64 0)", len, name)
            }
            _ => "0".to_string(),
        };
        self.globals.push_str(&format!(
            "@n0_{} = global {} {}, align 8\n",
            c.name, val_llvm_ty, init_val
        ));
    }

    pub(crate) fn gen_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.gen_stmt(stmt);
        }
    }
}
