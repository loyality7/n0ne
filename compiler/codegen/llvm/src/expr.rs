use ast::{Expr, Literal, BinOp, UnaryOp, Type, FStringPart};
use crate::LLVMGenerator;

impl LLVMGenerator {
    pub(crate) fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(name) => {
                if let Some((ptr, ty)) = self.variables.get(name).cloned() {
                    let r = self.next_reg();
                    let ty_str = self.llvm_type(&ty);
                    self.body.push_str(&format!(
                        "    {} = load {}, ptr {}, align 8\n",
                        r, ty_str, ptr
                    ));
                    r
                } else {
                    let r = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = load i64, ptr @n0_{}, align 8\n",
                        r, name
                    ));
                    r
                }
            }
            Expr::Literal(lit) => match lit {
                Literal::Int(i) => i.to_string(),
                Literal::Float(f) => {
                    let mut s = f.to_string();
                    if !s.contains('.') {
                        s.push_str(".0");
                    }
                    s
                }
                Literal::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                Literal::String(s) => {
                    let name = self.add_string_constant(s);
                    let len = self.string_constants.last().unwrap().2;
                    let r = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0\n",
                        r, len, name
                    ));
                    r
                }
            },
            Expr::ListLiteral(items) => {
                let list_ptr = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = call ptr @n0_c_alloc(i64 24)\n",
                    list_ptr
                ));
                let n = items.len();
                if n > 0 {
                    let data_size = (n * 8) as i64;
                    let data_ptr = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = call ptr @n0_c_alloc(i64 {})\n",
                        data_ptr, data_size
                    ));
                    for (i, item) in items.iter().enumerate() {
                        let item_reg = self.gen_expr(item);
                        let item_ty = self.infer_expr_type(item);
                        let item_llvm_ty = self.llvm_type(&item_ty);
                        let offset = (i * 8) as i64;

                        if item_llvm_ty == "double" {
                            let cast_reg = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = bitcast double {} to i64\n",
                                cast_reg, item_reg
                            ));
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                data_ptr, offset, cast_reg
                            ));
                        } else if item_llvm_ty == "ptr" {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_string(ptr {}, i64 {}, ptr {})\n",
                                data_ptr, offset, item_reg
                            ));
                        } else {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                data_ptr, offset, item_reg
                            ));
                        }
                    }
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 8, ptr {})\n",
                        list_ptr, data_ptr
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 8, ptr null)\n",
                        list_ptr
                    ));
                }
                self.body.push_str(&format!(
                    "    call void @n0_c_store_int(ptr {}, i64 16, i64 {})\n",
                    list_ptr, n
                ));
                list_ptr
            }
            Expr::MapLiteral(pairs) => {
                let map_ptr = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = call ptr @n0_c_alloc(i64 32)\n",
                    map_ptr
                ));
                let n = pairs.len();
                if n > 0 {
                    let buffer_size = (n * 8) as i64;
                    let keys_ptr = self.next_reg();
                    let vals_ptr = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = call ptr @n0_c_alloc(i64 {})\n",
                        keys_ptr, buffer_size
                    ));
                    self.body.push_str(&format!(
                        "    {} = call ptr @n0_c_alloc(i64 {})\n",
                        vals_ptr, buffer_size
                    ));
                    for (i, (key, value)) in pairs.iter().enumerate() {
                        let k_reg = self.gen_expr(key);
                        let k_ty = self.infer_expr_type(key);
                        let k_llvm_ty = self.llvm_type(&k_ty);
                        let offset = (i * 8) as i64;

                        if k_llvm_ty == "double" {
                            let cast_reg = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = bitcast double {} to i64\n",
                                cast_reg, k_reg
                            ));
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                keys_ptr, offset, cast_reg
                            ));
                        } else if k_llvm_ty == "ptr" {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_string(ptr {}, i64 {}, ptr {})\n",
                                keys_ptr, offset, k_reg
                            ));
                        } else {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                keys_ptr, offset, k_reg
                            ));
                        }

                        let v_reg = self.gen_expr(value);
                        let v_ty = self.infer_expr_type(value);
                        let v_llvm_ty = self.llvm_type(&v_ty);

                        if v_llvm_ty == "double" {
                            let cast_reg = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = bitcast double {} to i64\n",
                                cast_reg, v_reg
                            ));
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                vals_ptr, offset, cast_reg
                            ));
                        } else if v_llvm_ty == "ptr" {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_string(ptr {}, i64 {}, ptr {})\n",
                                vals_ptr, offset, v_reg
                            ));
                        } else {
                            self.body.push_str(&format!(
                                "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                                vals_ptr, offset, v_reg
                            ));
                        }
                    }
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 8, ptr {})\n",
                        map_ptr, keys_ptr
                    ));
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 16, ptr {})\n",
                        map_ptr, vals_ptr
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 8, ptr null)\n",
                        map_ptr
                    ));
                    self.body.push_str(&format!(
                        "    call void @n0_c_store_string(ptr {}, i64 16, ptr null)\n",
                        map_ptr
                    ));
                }
                self.body.push_str(&format!(
                    "    call void @n0_c_store_int(ptr {}, i64 24, i64 {})\n",
                    map_ptr, n
                ));
                map_ptr
            }
            Expr::BinExpr { left, op, right } => {
                let l_reg = self.gen_expr(left);
                let r_reg = self.gen_expr(right);
                let l_ty = self.infer_expr_type(left);
                let l_llvm_ty = self.llvm_type(&l_ty);

                if l_llvm_ty == "double" {
                    let r = self.next_reg();
                    let op_instr = match op {
                        BinOp::Add => "fadd",
                        BinOp::Sub => "fsub",
                        BinOp::Mul => "fmul",
                        BinOp::Div => "fdiv",
                        BinOp::Mod => "frem",
                        _ => "fadd",
                    };
                    self.body.push_str(&format!(
                        "    {} = {} double {}, {}\n",
                        r, op_instr, l_reg, r_reg
                    ));
                    r
                } else if l_llvm_ty == "ptr" {
                    if let BinOp::Add = op {
                        let r = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = call ptr @n0_c_interpolate(ptr {}, ptr {})\n",
                            r, l_reg, r_reg
                        ));
                        r
                    } else {
                        let cmp_op = match op {
                            BinOp::Eq => "eq",
                            BinOp::Ne => "ne",
                            _ => "eq",
                        };
                        let cmp_res = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = icmp {} ptr {}, {}\n",
                            cmp_res, cmp_op, l_reg, r_reg
                        ));
                        let r = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = zext i1 {} to i64\n",
                            r, cmp_res
                        ));
                        r
                    }
                } else {
                    match op {
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::And | BinOp::Or => {
                            let r = self.next_reg();
                            let op_instr = match op {
                                BinOp::Add => "add",
                                BinOp::Sub => "sub",
                                BinOp::Mul => "mul",
                                BinOp::Div => "sdiv",
                                BinOp::Mod => "srem",
                                BinOp::And => "and",
                                BinOp::Or => "or",
                                _ => "add",
                            };
                            self.body.push_str(&format!(
                                "    {} = {} i64 {}, {}\n",
                                r, op_instr, l_reg, r_reg
                            ));
                            r
                        }
                        BinOp::Pow => {
                            let d1 = self.next_reg();
                            let d2 = self.next_reg();
                            self.body.push_str(&format!("    {} = sitofp i64 {} to double\n", d1, l_reg));
                            self.body.push_str(&format!("    {} = sitofp i64 {} to double\n", d2, r_reg));
                            let d3 = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = call double @pow(double {}, double {})\n",
                                d3, d1, d2
                            ));
                            let r = self.next_reg();
                            self.body.push_str(&format!("    {} = fptosi double {} to i64\n", r, d3));
                            r
                        }
                        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                            let cmp_op = match op {
                                BinOp::Eq => "eq",
                                BinOp::Ne => "ne",
                                BinOp::Lt => "slt",
                                BinOp::Gt => "sgt",
                                BinOp::Le => "sle",
                                BinOp::Ge => "sge",
                                _ => "eq",
                            };
                            let cmp_res = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = icmp {} i64 {}, {}\n",
                                cmp_res, cmp_op, l_reg, r_reg
                            ));
                            let r = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = zext i1 {} to i64\n",
                                r, cmp_res
                            ));
                            r
                        }
                    }
                }
            }
            Expr::UnaryExpr { op, expr: inner } => {
                let inner_reg = self.gen_expr(inner);
                let inner_ty = self.infer_expr_type(inner);
                let inner_llvm_ty = self.llvm_type(&inner_ty);

                match op {
                    UnaryOp::Neg => {
                        let r = self.next_reg();
                        if inner_llvm_ty == "double" {
                            self.body.push_str(&format!(
                                "    {} = fneg double {}\n",
                                r, inner_reg
                            ));
                        } else {
                            self.body.push_str(&format!(
                                "    {} = sub i64 0, {}\n",
                                r, inner_reg
                            ));
                        }
                        r
                    }
                    UnaryOp::Not => {
                        let cmp_res = self.next_reg();
                        if inner_llvm_ty == "ptr" {
                            self.body.push_str(&format!(
                                "    {} = icmp eq ptr {}, null\n",
                                cmp_res, inner_reg
                            ));
                        } else {
                            self.body.push_str(&format!(
                                "    {} = icmp eq i64 {}, 0\n",
                                cmp_res, inner_reg
                            ));
                        }
                        let r = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = zext i1 {} to i64\n",
                            r, cmp_res
                        ));
                        r
                    }
                }
            }
            Expr::CallExpr { callee, args } => {
                if let Expr::Ident(name) = &**callee {
                    let (mapped_name, is_builtin) = match name.as_str() {
                        "show" => {
                            if let Some(first) = args.first() {
                                let arg_ty = self.infer_expr_type(first);
                                match arg_ty {
                                    Type::Basic(n) if n == "string" => ("n0_show_string", true),
                                    Type::Basic(n) if n == "float" => ("n0_show_float", true),
                                    _ => ("n0_show_int", true),
                                }
                            } else {
                                ("n0_show_int", true)
                            }
                        }
                        _ => {
                            if let Some((mapped, is_b)) = crate::builtins::get_builtin_mapping(name.as_str()) {
                                (mapped, is_b)
                            } else {
                                (name.as_str(), false)
                            }
                        }
                    };

                    let llvm_name = if is_builtin { mapped_name.to_string() } else { format!("n0_{}", mapped_name) };
                    let mut arg_regs = Vec::new();
                    for arg in args {
                        let arg_reg = self.gen_expr(arg);
                        let arg_ty = self.infer_expr_type(arg);
                        let arg_llvm_ty = self.llvm_type(&arg_ty);
                        arg_regs.push(format!("{} {}", arg_llvm_ty, arg_reg));
                    }

                    let mut ret_llvm_ty = "i64".to_string();
                    if is_builtin {
                        if mapped_name == "n0_c_alloc" || mapped_name == "n0_c_load_string" || mapped_name == "n0_c_interpolate" || mapped_name == "n0_c_argv" {
                            ret_llvm_ty = "ptr".to_string();
                        } else if mapped_name.starts_with("n0_show") || mapped_name == "n0_c_store_int" || mapped_name == "n0_c_store_string" {
                            ret_llvm_ty = "void".to_string();
                        }
                    } else {
                        if name.starts_with("make_") {
                            ret_llvm_ty = "ptr".to_string();
                        } else if name == "ok" || name == "err" || name == "risky" {
                            ret_llvm_ty = "ptr".to_string();
                        }
                    }

                    let r = self.next_reg();
                    if ret_llvm_ty == "void" {
                        self.body.push_str(&format!(
                            "    call void @{}({})\n",
                            llvm_name,
                            arg_regs.join(", ")
                        ));
                        "0".to_string()
                    } else {
                        self.body.push_str(&format!(
                            "    {} = call {} @{}({})\n",
                            r,
                            ret_llvm_ty,
                            llvm_name,
                            arg_regs.join(", ")
                        ));
                        r
                    }
                } else if let Expr::FieldAccess { expr: receiver, field: method_name } = &**callee {
                    if let Expr::Ident(mod_name) = &**receiver {
                        if self.variables.get(mod_name).is_none() {
                            if mod_name == "io" || mod_name == "fs" || mod_name == "json" || mod_name == "http" {
                                if mod_name == "io" && method_name == "show" {
                                    let first = args.first().unwrap();
                                    let arg_reg = self.gen_expr(first);
                                    let arg_ty = self.infer_expr_type(first);
                                    let (fn_name, arg_llvm_ty) = match arg_ty {
                                        Type::Basic(n) if n == "string" => ("n0_show_string", "ptr"),
                                        Type::Basic(n) if n == "float" => ("n0_show_float", "double"),
                                        _ => ("n0_show_int", "i64"),
                                    };
                                    self.body.push_str(&format!("    call void @{}({} {})\n", fn_name, arg_llvm_ty, arg_reg));
                                    return "0".to_string();
                                }
                                
                                if mod_name == "io" && method_name == "show_err" {
                                    let first = args.first().unwrap();
                                    let arg_reg = self.gen_expr(first);
                                    let arg_ty = self.infer_expr_type(first);
                                    let arg_llvm_ty = match arg_ty {
                                        Type::Basic(n) if n == "string" => "ptr",
                                        _ => "ptr",
                                    };
                                    let ptr_reg = if arg_llvm_ty == "ptr" {
                                        arg_reg
                                    } else if arg_llvm_ty == "double" {
                                        let r = self.next_reg();
                                        self.body.push_str(&format!("    {} = call ptr @n0_float_to_string(double {})\n", r, arg_reg));
                                        r
                                    } else {
                                        let r = self.next_reg();
                                        self.body.push_str(&format!("    {} = call ptr @n0_int_to_string(i64 {})\n", r, arg_reg));
                                        r
                                    };
                                    self.body.push_str(&format!("    call void @n0_io_show_err(ptr {})\n", ptr_reg));
                                    return "0".to_string();
                                }

                                let (fn_name, ret_llvm_ty) = match (mod_name.as_str(), method_name.as_str()) {
                                    ("io", "read") => ("n0_io_read", "ptr"),
                                    ("fs", "read") => ("n0_fs_read", "ptr"),
                                    ("fs", "write") => ("n0_fs_write", "ptr"),
                                    ("fs", "exists") => ("n0_fs_exists", "i64"),
                                    ("fs", "delete") => ("n0_fs_delete", "ptr"),
                                    ("fs", "mkdir") => ("n0_fs_mkdir", "ptr"),
                                    ("fs", "list") => ("n0_fs_list", "ptr"),
                                    ("json", "encode") => ("n0_json_encode", "ptr"),
                                    ("json", "decode") => ("n0_json_decode", "ptr"),
                                    ("http", "get") => ("n0_http_get", "ptr"),
                                    ("http", "post") => ("n0_http_post", "ptr"),
                                    _ => ("", ""),
                                };

                                if !fn_name.is_empty() {
                                    let mut cast_args = Vec::new();
                                    for arg in args {
                                        let arg_reg = self.gen_expr(arg);
                                        let arg_ty = self.infer_expr_type(arg);
                                        let arg_llvm_ty = self.llvm_type(&arg_ty);
                                        if arg_llvm_ty == "i64" {
                                            let r = self.next_reg();
                                            self.body.push_str(&format!("    {} = inttoptr i64 {} to ptr\n", r, arg_reg));
                                            cast_args.push(format!("ptr {}", r));
                                        } else if arg_llvm_ty == "double" {
                                            let r1 = self.next_reg();
                                            self.body.push_str(&format!("    {} = bitcast double {} to i64\n", r1, arg_reg));
                                            let r2 = self.next_reg();
                                            self.body.push_str(&format!("    {} = inttoptr i64 {} to ptr\n", r2, r1));
                                            cast_args.push(format!("ptr {}", r2));
                                        } else {
                                            cast_args.push(format!("ptr {}", arg_reg));
                                        }
                                    }

                                    let r = self.next_reg();
                                    if ret_llvm_ty == "void" {
                                        self.body.push_str(&format!("    call void @{}({})\n", fn_name, cast_args.join(", ")));
                                        return "0".to_string();
                                    } else {
                                        self.body.push_str(&format!("    {} = call {} @{}({})\n", r, ret_llvm_ty, fn_name, cast_args.join(", ")));
                                        return r;
                                    }
                                }
                            } else {
                                let llvm_name = format!("n0_{}", method_name);
                                let mut arg_regs = Vec::new();
                                for arg in args {
                                    let arg_reg = self.gen_expr(arg);
                                    let arg_ty = self.infer_expr_type(arg);
                                    let arg_llvm_ty = self.llvm_type(&arg_ty);
                                    arg_regs.push(format!("{} {}", arg_llvm_ty, arg_reg));
                                }

                                let mut ret_llvm_ty = "i64".to_string();
                                if let Some(ret_ty) = self.functions.get(method_name) {
                                    ret_llvm_ty = self.llvm_type(ret_ty);
                                }

                                let r = self.next_reg();
                                if ret_llvm_ty == "void" {
                                    self.body.push_str(&format!("    call void @{}({})\n", llvm_name, arg_regs.join(", ")));
                                    return "0".to_string();
                                } else {
                                    self.body.push_str(&format!("    {} = call {} @{}({})\n", r, ret_llvm_ty, llvm_name, arg_regs.join(", ")));
                                    return r;
                                }
                            }
                        }
                    }

                    let receiver_reg = self.gen_expr(receiver);
                    let receiver_ty = self.infer_expr_type(receiver);
                    let receiver_llvm_ty = self.llvm_type(&receiver_ty);
                    
                    let mut mapped_fn_name = "".to_string();
                    let mut ret_llvm_ty = "i64".to_string();
                    let mut is_void = false;
                    let mut cast_args = Vec::new();
                    cast_args.push(format!("{} {}", receiver_llvm_ty, receiver_reg));

                    match &receiver_ty {
                        Type::Basic(name) if name == "string" => {
                            match method_name.as_str() {
                                "len" => mapped_fn_name = "n0_str_len".to_string(),
                                "contains" => mapped_fn_name = "n0_str_contains".to_string(),
                                "starts_with" => mapped_fn_name = "n0_str_starts_with".to_string(),
                                "ends_with" => mapped_fn_name = "n0_str_ends_with".to_string(),
                                "upper" => {
                                    mapped_fn_name = "n0_str_upper".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "lower" => {
                                    mapped_fn_name = "n0_str_lower".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "trim" => {
                                    mapped_fn_name = "n0_str_trim".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "split" => {
                                    mapped_fn_name = "n0_str_split".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "replace" => {
                                    mapped_fn_name = "n0_str_replace".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "slice" => {
                                    mapped_fn_name = "n0_str_slice".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "to_int" | "to_float" => {
                                    mapped_fn_name = if method_name == "to_int" { "n0_str_to_int".to_string() } else { "n0_str_to_float".to_string() };
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                _ => {}
                            }
                        }
                        Type::Basic(name) if name == "int" => {
                            match method_name.as_str() {
                                "to_string" => {
                                    mapped_fn_name = "n0_int_to_string".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "to_float" => {
                                    mapped_fn_name = "n0_int_to_float".to_string();
                                    ret_llvm_ty = "double".to_string();
                                }
                                _ => {}
                            }
                        }
                        Type::Basic(name) if name == "float" => {
                            match method_name.as_str() {
                                "to_int" => {
                                    mapped_fn_name = "n0_float_to_int".to_string();
                                    ret_llvm_ty = "i64".to_string();
                                }
                                "to_string" => {
                                    mapped_fn_name = "n0_float_to_string".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                _ => {}
                            }
                        }
                        Type::List(inner) => {
                            match method_name.as_str() {
                                "len" => mapped_fn_name = "n0_list_len".to_string(),
                                "push" => {
                                    mapped_fn_name = "n0_list_push".to_string();
                                    is_void = true;
                                }
                                "pop" | "first" | "last" => {
                                    mapped_fn_name = match method_name.as_str() {
                                        "pop" => "n0_list_pop".to_string(),
                                        "first" => "n0_list_first".to_string(),
                                        _ => "n0_list_last".to_string(),
                                    };
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "contains" => {
                                    mapped_fn_name = if self.llvm_type(inner) == "ptr" {
                                        "n0_list_contains_str".to_string()
                                    } else {
                                        "n0_list_contains_int".to_string()
                                    };
                                }
                                _ => {}
                            }
                        }
                        Type::Map(_, _val_ty) => {
                            match method_name.as_str() {
                                "get" => {
                                    mapped_fn_name = "n0_map_get".to_string();
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "set" => {
                                    mapped_fn_name = "n0_map_set".to_string();
                                    is_void = true;
                                }
                                "has" => {
                                    mapped_fn_name = "n0_map_has".to_string();
                                }
                                "keys" | "values" => {
                                    mapped_fn_name = if method_name == "keys" { "n0_map_keys".to_string() } else { "n0_map_values".to_string() };
                                    ret_llvm_ty = "ptr".to_string();
                                }
                                "delete" => {
                                    mapped_fn_name = "n0_map_delete".to_string();
                                    is_void = true;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    for (i, arg) in args.iter().enumerate() {
                        let arg_reg = self.gen_expr(arg);
                        let arg_ty = self.infer_expr_type(arg);
                        let arg_llvm_ty = self.llvm_type(&arg_ty);
                        
                        let is_generic_val_arg = (method_name == "push" && i == 0)
                            || (method_name == "set" && i == 1)
                            || (method_name == "contains" && i == 0 && mapped_fn_name == "n0_list_contains_int");

                        if is_generic_val_arg {
                            if arg_llvm_ty == "double" {
                                let cast_reg = self.next_reg();
                                self.body.push_str(&format!(
                                    "    {} = bitcast double {} to i64\n",
                                    cast_reg, arg_reg
                                ));
                                cast_args.push(format!("i64 {}", cast_reg));
                            } else if arg_llvm_ty == "ptr" {
                                let cast_reg = self.next_reg();
                                self.body.push_str(&format!(
                                    "    {} = ptrtoint ptr {} to i64\n",
                                    cast_reg, arg_reg
                                ));
                                cast_args.push(format!("i64 {}", cast_reg));
                            } else {
                                cast_args.push(format!("i64 {}", arg_reg));
                            }
                        } else {
                            cast_args.push(format!("{} {}", arg_llvm_ty, arg_reg));
                        }
                    }

                    if is_void {
                        self.body.push_str(&format!(
                            "    call void @{}({})\n",
                            mapped_fn_name,
                            cast_args.join(", ")
                        ));
                        "0".to_string()
                    } else {
                        let r = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = call {} @{}({})\n",
                            r,
                            ret_llvm_ty,
                            mapped_fn_name,
                            cast_args.join(", ")
                        ));
                        r
                    }
                } else {
                    "0".to_string()
                }
            }
            Expr::FieldAccess { expr: inner, field } => {
                let ptr_reg = self.gen_expr(inner);
                let inner_ty = self.infer_expr_type(inner);
                let type_name = match &inner_ty {
                    Type::Basic(n) => n.clone(),
                    Type::Result(_) => "result".to_string(),
                    Type::Option(_) => "option".to_string(),
                    _ => "unknown".to_string(),
                };
                let offset = self.get_field_offset(&type_name, field);

                let field_ty = if type_name == "result" && field == "is_err" {
                    Type::Basic("int".to_string())
                } else if type_name == "result" && field == "error" {
                    Type::Basic("string".to_string())
                } else if type_name == "result" && field == "value" {
                    match &inner_ty {
                        Type::Result(t) => (**t).clone(),
                        _ => Type::Basic("unknown".to_string()),
                    }
                } else if type_name == "option" && (field == "is_some" || field == "is_none") {
                    Type::Basic("bool".to_string())
                } else if type_name == "option" && field == "value" {
                    match &inner_ty {
                        Type::Option(t) => (**t).clone(),
                        _ => Type::Basic("unknown".to_string()),
                    }
                } else if let Some(decl) = self.structs.get(&type_name) {
                    decl.fields.iter().find(|f| &f.name == field).map(|f| f.type_ann.clone()).unwrap_or(Type::Basic("unknown".to_string()))
                } else {
                    Type::Basic("unknown".to_string())
                };

                let field_llvm_ty = self.llvm_type(&field_ty);
                let r = self.next_reg();

                if field_llvm_ty == "ptr" {
                    self.body.push_str(&format!(
                        "    {} = call ptr @n0_c_load_string(ptr {}, i64 {})\n",
                        r, ptr_reg, offset
                    ));
                } else if field_llvm_ty == "double" {
                    let r1 = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                        r1, ptr_reg, offset
                    ));
                    self.body.push_str(&format!(
                        "    {} = bitcast i64 {} to double\n",
                        r, r1
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                        r, ptr_reg, offset
                    ));
                }
                r
            }
            Expr::FStringExpr(parts) => {
                let n = parts.len();
                if n == 0 {
                    let name = self.add_string_constant("");
                    let len = self.string_constants.last().unwrap().2;
                    let r = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0\n",
                        r, len, name
                    ));
                    return r;
                }

                let array_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = alloca [{} x ptr], align 8\n",
                    array_reg, n
                ));

                for (i, part) in parts.iter().enumerate() {
                    let part_str_reg = match part {
                        FStringPart::Text(text) => {
                            let name = self.add_string_constant(text);
                            let len = self.string_constants.last().unwrap().2;
                            let r = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0\n",
                                r, len, name
                            ));
                            r
                        }
                        FStringPart::Expr(expr) => {
                            let val_reg = self.gen_expr(expr);
                            let val_ty = self.infer_expr_type(expr);
                            let val_llvm_ty = self.llvm_type(&val_ty);

                            let r = self.next_reg();
                            if val_llvm_ty == "i64" {
                                if val_ty == Type::Basic("bool".to_string()) {
                                    let trunc_reg = self.next_reg();
                                    self.body.push_str(&format!(
                                        "    {} = trunc i64 {} to i32\n",
                                        trunc_reg, val_reg
                                    ));
                                    self.body.push_str(&format!(
                                        "    {} = call ptr @n0_bool_to_string(i32 {})\n",
                                        r, trunc_reg
                                    ));
                                } else {
                                    self.body.push_str(&format!(
                                        "    {} = call ptr @n0_int_to_string(i64 {})\n",
                                        r, val_reg
                                    ));
                                }
                                r
                            } else if val_llvm_ty == "double" {
                                self.body.push_str(&format!(
                                    "    {} = call ptr @n0_float_to_string(double {})\n",
                                    r, val_reg
                                ));
                                r
                            } else if val_llvm_ty == "ptr" {
                                val_reg.clone()
                            } else {
                                self.body.push_str(&format!(
                                    "    {} = call ptr @n0_int_to_string(i64 {})\n",
                                    r, val_reg
                                ));
                                r
                            }
                        }
                    };

                    let elem_ptr = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = getelementptr inbounds [{} x ptr], ptr {}, i64 0, i64 {}\n",
                        elem_ptr, n, array_reg, i
                    ));
                    self.body.push_str(&format!(
                        "    store ptr {}, ptr {}, align 8\n",
                        part_str_reg, elem_ptr
                    ));
                }

                let first_elem_ptr = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = getelementptr inbounds [{} x ptr], ptr {}, i64 0, i64 0\n",
                    first_elem_ptr, n, array_reg
                ));

                let res_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = call ptr @n0_string_concat(ptr {}, i32 {})\n",
                    res_reg, first_elem_ptr, n
                ));

                res_reg
            }
            Expr::TryExpr(inner) => {
                self.gen_expr(inner)
            }
        }
    }
}
