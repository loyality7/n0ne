use ast::{Stmt, Expr, Type, AssignOp, MatchArm, Literal};
use crate::LLVMGenerator;
use crate::emitter::block_has_terminator;

impl LLVMGenerator {
    pub(crate) fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, op, value } => {
                let val_reg = self.gen_expr(value);
                let val_ty = self.infer_expr_type(value);

                if let Expr::Tuple(targets) = target {
                    if let Type::Tuple(types) = &val_ty {
                        for (i, t_expr) in targets.iter().enumerate() {
                            if let Expr::Ident(name) = t_expr {
                                let field_ty = &types[i];
                                let field_llvm_ty = self.llvm_type(field_ty);
                                let offset = (i * 8) as i64;

                                let field_reg = self.next_reg();
                                if field_llvm_ty == "ptr" {
                                    self.body.push_str(&format!(
                                        "    {} = call ptr @n0_c_load_string(ptr {}, i64 {})\n",
                                        field_reg, val_reg, offset
                                    ));
                                } else if field_llvm_ty == "double" {
                                    let temp_reg = self.next_reg();
                                    self.body.push_str(&format!(
                                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                                        temp_reg, val_reg, offset
                                    ));
                                    self.body.push_str(&format!(
                                        "    {} = bitcast i64 {} to double\n",
                                        field_reg, temp_reg
                                    ));
                                } else {
                                    self.body.push_str(&format!(
                                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                                        field_reg, val_reg, offset
                                    ));
                                }

                                if !self.variables.contains_key(name) {
                                    self.body.push_str(&format!(
                                        "    %_{} = alloca {}, align 8\n",
                                        name, field_llvm_ty
                                    ));
                                    self.variables.insert(name.clone(), (format!("%_{}", name), field_ty.clone()));
                                }

                                let (ptr, _) = self.variables.get(name).unwrap().clone();
                                self.body.push_str(&format!(
                                    "    store {} {}, ptr {}, align 8\n",
                                    field_llvm_ty, field_reg, ptr
                                ));
                            }
                        }
                    }
                    return;
                }

                let val_llvm_ty = self.llvm_type(&val_ty);

                if let Expr::Ident(name) = target {
                    if let AssignOp::Eq = op {
                        if !self.variables.contains_key(name) {
                            self.body.push_str(&format!(
                                "    %_{} = alloca {}, align 8\n",
                                name, val_llvm_ty
                            ));
                            self.variables.insert(name.clone(), (format!("%_{}", name), val_ty.clone()));
                        }
                    }
                    let (ptr, _) = self.variables.get(name).unwrap().clone();
                    let store_val = match op {
                        AssignOp::Eq => val_reg,
                        _ => {
                            let r1 = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = load {}, ptr {}, align 8\n",
                                r1, val_llvm_ty, ptr
                            ));
                            let op_instr = match op {
                                AssignOp::PlusEq => "add",
                                AssignOp::MinusEq => "sub",
                                AssignOp::StarEq => "mul",
                                AssignOp::SlashEq => "sdiv",
                                _ => "add",
                            };
                            let r2 = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = {} {} {}, {}\n",
                                r2, op_instr, val_llvm_ty, r1, val_reg
                            ));
                            r2
                        }
                    };
                    self.body.push_str(&format!(
                        "    store {} {}, ptr {}, align 8\n",
                        val_llvm_ty, store_val, ptr
                    ));
                } else if let Expr::FieldAccess { expr: inner, field } = target {
                    let ptr_reg = self.gen_expr(inner);
                    let inner_ty = self.infer_expr_type(inner);
                    let type_name = match &inner_ty {
                        Type::Basic(n) => n.clone(),
                        Type::Result(_) => "result".to_string(),
                        _ => "unknown".to_string(),
                    };
                    let offset = self.get_field_offset(&type_name, field);

                    if val_llvm_ty == "double" {
                        let cast_reg = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = bitcast double {} to i64\n",
                            cast_reg, val_reg
                        ));
                        self.body.push_str(&format!(
                            "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                            ptr_reg, offset, cast_reg
                        ));
                    } else if val_llvm_ty == "ptr" {
                        self.body.push_str(&format!(
                            "    call void @n0_c_store_string(ptr {}, i64 {}, ptr {})\n",
                            ptr_reg, offset, val_reg
                        ));
                    } else {
                        self.body.push_str(&format!(
                            "    call void @n0_c_store_int(ptr {}, i64 {}, i64 {})\n",
                            ptr_reg, offset, val_reg
                        ));
                    }
                }
            }
            Stmt::ConstDecl(c) => {
                let val_reg = self.gen_expr(&c.value);
                let val_ty = self.infer_expr_type(&c.value);
                let val_llvm_ty = self.llvm_type(&val_ty);
                self.body.push_str(&format!(
                    "    %_{} = alloca {}, align 8\n",
                    c.name, val_llvm_ty
                ));
                self.variables.insert(c.name.clone(), (format!("%_{}", c.name), val_ty.clone()));
                self.body.push_str(&format!(
                    "    store {} {}, ptr %_{}, align 8\n",
                    val_llvm_ty, val_reg, c.name
                ));
            }
            Stmt::If { cond, then_branch, elifs, else_branch } => {
                let cond_reg = self.gen_expr(cond);
                let cond_ty = self.infer_expr_type(cond);
                let cond_llvm_ty = self.llvm_type(&cond_ty);
                let cmp_reg = self.next_reg();

                if cond_llvm_ty == "ptr" {
                    self.body.push_str(&format!(
                        "    {} = icmp ne ptr {}, null\n",
                        cmp_reg, cond_reg
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    {} = icmp ne i64 {}, 0\n",
                        cmp_reg, cond_reg
                    ));
                }

                let then_lbl = self.next_label("then");
                let cont_lbl = self.next_label("cont");
                let mut next_lbl = cont_lbl.clone();

                if !elifs.is_empty() || else_branch.is_some() {
                    next_lbl = self.next_label("else");
                }

                self.body.push_str(&format!(
                    "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                    cmp_reg, then_lbl, next_lbl, then_lbl
                ));

                self.gen_block(then_branch);
                if !block_has_terminator(then_branch) {
                    self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                }

                let mut current_else_lbl = next_lbl;
                for (e_cond, e_block) in elifs {
                    self.body.push_str(&format!("\n{}:\n", current_else_lbl));
                    let e_cond_reg = self.gen_expr(e_cond);
                    let e_cond_ty = self.infer_expr_type(e_cond);
                    let e_cond_llvm_ty = self.llvm_type(&e_cond_ty);
                    let e_cmp_reg = self.next_reg();

                    if e_cond_llvm_ty == "ptr" {
                        self.body.push_str(&format!(
                            "    {} = icmp ne ptr {}, null\n",
                            e_cmp_reg, e_cond_reg
                        ));
                    } else {
                        self.body.push_str(&format!(
                            "    {} = icmp ne i64 {}, 0\n",
                            e_cmp_reg, e_cond_reg
                        ));
                    }

                    let elif_then_lbl = self.next_label("elif_then");
                    current_else_lbl = self.next_label("elif_else");

                    self.body.push_str(&format!(
                        "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                        e_cmp_reg, elif_then_lbl, current_else_lbl, elif_then_lbl
                    ));

                    self.gen_block(e_block);
                    if !block_has_terminator(e_block) {
                        self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                    }
                }

                if let Some(eb) = else_branch {
                    self.body.push_str(&format!("\n{}:\n", current_else_lbl));
                    self.gen_block(eb);
                    if !block_has_terminator(eb) {
                        self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                    }
                } else if !elifs.is_empty() {
                    self.body.push_str(&format!("\n{}:\n", current_else_lbl));
                    self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                }

                self.body.push_str(&format!("\n{}:\n", cont_lbl));
            }
            Stmt::For { var, iterable, body } => {
                let iter_reg = self.gen_expr(iterable);

                let len_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = call i64 @n0_c_load_int(ptr {}, i64 16)\n",
                    len_reg, iter_reg
                ));

                let data_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = call ptr @n0_c_load_string(ptr {}, i64 8)\n",
                    data_reg, iter_reg
                ));

                let i_ptr = self.next_reg();
                self.body.push_str(&format!("    {} = alloca i64, align 8\n", i_ptr));
                self.body.push_str(&format!("    store i64 0, ptr {}, align 8\n", i_ptr));

                let iter_ty = self.infer_expr_type(iterable);
                let elem_ty = match iter_ty {
                    Type::List(inner) => *inner,
                    _ => Type::Basic("int".to_string()),
                };
                let var_ptr = format!("%_{}", var);
                let var_llvm_ty = self.llvm_type(&elem_ty);
                self.body.push_str(&format!("    {} = alloca {}, align 8\n", var_ptr, var_llvm_ty));
                self.variables.insert(var.clone(), (var_ptr.clone(), elem_ty.clone()));

                let cond_lbl = self.next_label("loop_cond");
                let body_lbl = self.next_label("loop_body");
                let step_lbl = self.next_label("loop_step");
                let end_lbl = self.next_label("loop_end");

                self.body.push_str(&format!("    br label %{}\n\n{}:\n", cond_lbl, cond_lbl));

                let i_val = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = load i64, ptr {}, align 8\n",
                    i_val, i_ptr
                ));
                let cmp_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = icmp slt i64 {}, {}\n",
                    cmp_reg, i_val, len_reg
                ));
                self.body.push_str(&format!(
                    "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                    cmp_reg, body_lbl, end_lbl, body_lbl
                ));

                let offset_reg = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = mul i64 {}, 8\n",
                    offset_reg, i_val
                ));
                let item_reg = self.next_reg();
                if var_llvm_ty == "ptr" {
                    self.body.push_str(&format!(
                        "    {} = call ptr @n0_c_load_string(ptr {}, i64 {})\n",
                        item_reg, data_reg, offset_reg
                    ));
                    self.body.push_str(&format!(
                        "    store ptr {}, ptr {}, align 8\n",
                        item_reg, var_ptr
                    ));
                } else if var_llvm_ty == "double" {
                    let r_int = self.next_reg();
                    self.body.push_str(&format!(
                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                        r_int, data_reg, offset_reg
                    ));
                    self.body.push_str(&format!(
                        "    {} = bitcast i64 {} to double\n",
                        item_reg, r_int
                    ));
                    self.body.push_str(&format!(
                        "    store double {}, ptr {}, align 8\n",
                        item_reg, var_ptr
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                        item_reg, data_reg, offset_reg
                    ));
                    self.body.push_str(&format!(
                        "    store i64 {}, ptr {}, align 8\n",
                        item_reg, var_ptr
                    ));
                }

                self.loop_stack.push((step_lbl.clone(), end_lbl.clone()));
                self.gen_block(body);
                self.loop_stack.pop();

                if !block_has_terminator(body) {
                    self.body.push_str(&format!("    br label %{}\n", step_lbl));
                }

                self.body.push_str(&format!("\n{}:\n", step_lbl));
                let next_i = self.next_reg();
                self.body.push_str(&format!(
                    "    {} = add i64 {}, 1\n",
                    next_i, i_val
                ));
                self.body.push_str(&format!(
                    "    store i64 {}, ptr {}, align 8\n",
                    next_i, i_ptr
                ));
                self.body.push_str(&format!("    br label %{}\n\n{}:\n", cond_lbl, end_lbl));
            }
            Stmt::While { cond, body } => {
                let cond_lbl = self.next_label("while_cond");
                self.body.push_str(&format!("    br label %{}\n\n{}:\n", cond_lbl, cond_lbl));

                let cond_reg = self.gen_expr(cond);
                let cond_ty = self.infer_expr_type(cond);
                let cond_llvm_ty = self.llvm_type(&cond_ty);
                let cmp_reg = self.next_reg();

                if cond_llvm_ty == "ptr" {
                    self.body.push_str(&format!(
                        "    {} = icmp ne ptr {}, null\n",
                        cmp_reg, cond_reg
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    {} = icmp ne i64 {}, 0\n",
                        cmp_reg, cond_reg
                    ));
                }

                let body_lbl = self.next_label("while_body");
                let end_lbl = self.next_label("while_end");

                self.body.push_str(&format!(
                    "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                    cmp_reg, body_lbl, end_lbl, body_lbl
                ));

                self.loop_stack.push((cond_lbl.clone(), end_lbl.clone()));
                self.gen_block(body);
                self.loop_stack.pop();

                if !block_has_terminator(body) {
                    self.body.push_str(&format!("    br label %{}\n", cond_lbl));
                }

                self.body.push_str(&format!("\n{}:\n", end_lbl));
            }
            Stmt::Break => {
                if let Some((_, break_lbl)) = self.loop_stack.last().cloned() {
                    self.body.push_str(&format!("    br label %{}\n", break_lbl));
                }
            }
            Stmt::Continue => {
                if let Some((cont_lbl, _)) = self.loop_stack.last().cloned() {
                    self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                }
            }
            Stmt::Match { expr, cases } => {
                let val_reg = self.gen_expr(expr);

                let exit_lbl = self.next_label("match_exit");
                let mut current_cmp_lbl = self.next_label("match_case");
                
                let first_cmp_lbl = current_cmp_lbl.clone();
                self.body.push_str(&format!("    br label %{}\n", first_cmp_lbl));

                for (i, (arm, body)) in cases.iter().enumerate() {
                    let next_cmp_lbl = if i + 1 < cases.len() {
                        self.next_label("match_case")
                    } else {
                        exit_lbl.clone()
                    };

                    self.body.push_str(&format!("\n{}:\n", current_cmp_lbl));

                    match arm {
                        MatchArm::Literal(lit) => {
                            let cmp_reg = self.next_reg();
                            match lit {
                                Literal::Int(val) => {
                                    self.body.push_str(&format!(
                                        "    {} = icmp eq i64 {}, {}\n",
                                        cmp_reg, val_reg, val
                                    ));
                                }
                                Literal::Float(val) => {
                                    self.body.push_str(&format!(
                                        "    {} = fcmp oeq double {}, {}\n",
                                        cmp_reg, val_reg, val
                                    ));
                                }
                                Literal::Bool(val) => {
                                    let b_val = if *val { 1 } else { 0 };
                                    self.body.push_str(&format!(
                                        "    {} = icmp eq i64 {}, {}\n",
                                        cmp_reg, val_reg, b_val
                                    ));
                                }
                                Literal::String(val) => {
                                    let str_const_name = self.add_string_constant(val);
                                    let len = self.string_constants.last().unwrap().2;
                                    let lit_ptr = self.next_reg();
                                    self.body.push_str(&format!(
                                        "    {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0\n",
                                        lit_ptr, len, str_const_name
                                    ));
                                    let strcmp_res = self.next_reg();
                                    self.body.push_str(&format!(
                                        "    {} = call i32 @strcmp(ptr {}, ptr {})\n",
                                        strcmp_res, val_reg, lit_ptr
                                    ));
                                    self.body.push_str(&format!(
                                        "    {} = icmp eq i32 {}, 0\n",
                                        cmp_reg, strcmp_res
                                    ));
                                }
                            }
                            let case_body_lbl = self.next_label("match_body");
                            self.body.push_str(&format!(
                                "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                                cmp_reg, case_body_lbl, next_cmp_lbl, case_body_lbl
                            ));
                            self.gen_block(body);
                            if !block_has_terminator(body) {
                                self.body.push_str(&format!("    br label %{}\n", exit_lbl));
                            }
                        }
                        MatchArm::Variant { variant_name, bindings } => {
                            let (_, var, tag_val) = self.find_variant(variant_name).expect("Sema guaranteed variant exists");
                            let tag_reg = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = call i64 @n0_c_load_int(ptr {}, i64 0)\n",
                                tag_reg, val_reg
                            ));
                            let cmp_reg = self.next_reg();
                            self.body.push_str(&format!(
                                "    {} = icmp eq i64 {}, {}\n",
                                cmp_reg, tag_reg, tag_val
                            ));

                            let case_body_lbl = self.next_label("match_body");
                            self.body.push_str(&format!(
                                "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                                cmp_reg, case_body_lbl, next_cmp_lbl, case_body_lbl
                            ));

                            let old_vars = self.variables.clone();

                            for (idx, binding) in bindings.iter().enumerate() {
                                if idx < var.fields.len() {
                                    let field_ty = &var.fields[idx];
                                    let llvm_ty = self.llvm_type(field_ty);
                                    let offset = 8 + (idx * 8) as i64;

                                    self.body.push_str(&format!(
                                        "    %_{} = alloca {}, align 8\n",
                                        binding, llvm_ty
                                    ));
                                    self.variables.insert(binding.clone(), (format!("%_{}", binding), field_ty.clone()));

                                    let val_loaded = self.next_reg();
                                    if llvm_ty == "ptr" {
                                        self.body.push_str(&format!(
                                            "    {} = call ptr @n0_c_load_string(ptr {}, i64 {})\n",
                                            val_loaded, val_reg, offset
                                        ));
                                        self.body.push_str(&format!(
                                            "    call void @n0_c_store_string(ptr %_{}, i64 0, ptr {})\n",
                                            binding, val_loaded
                                        ));
                                    } else if llvm_ty == "double" {
                                        self.body.push_str(&format!(
                                            "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                                            val_loaded, val_reg, offset
                                        ));
                                        let cast_reg = self.next_reg();
                                        self.body.push_str(&format!(
                                            "    {} = bitcast i64 {} to double\n",
                                            cast_reg, val_loaded
                                        ));
                                        self.body.push_str(&format!(
                                            "    call void @n0_c_store_int(ptr %_{}, i64 0, i64 {})\n",
                                            binding, cast_reg
                                        ));
                                    } else {
                                        self.body.push_str(&format!(
                                            "    {} = call i64 @n0_c_load_int(ptr {}, i64 {})\n",
                                            val_loaded, val_reg, offset
                                        ));
                                        self.body.push_str(&format!(
                                            "    call void @n0_c_store_int(ptr %_{}, i64 0, i64 {})\n",
                                            binding, val_loaded
                                        ));
                                    }
                                }
                            }

                            self.gen_block(body);
                            self.variables = old_vars;

                            if !block_has_terminator(body) {
                                self.body.push_str(&format!("    br label %{}\n", exit_lbl));
                            }
                        }
                        MatchArm::Wildcard => {
                            self.gen_block(body);
                            if !block_has_terminator(body) {
                                self.body.push_str(&format!("    br label %{}\n", exit_lbl));
                            }
                        }
                    }

                    current_cmp_lbl = next_cmp_lbl;
                }

                self.body.push_str(&format!("\n{}:\n", exit_lbl));
            }
            Stmt::Return(opt_val) => {
                let final_reg_opt = if let Some(v) = opt_val {
                    let val_reg = self.gen_expr(v);
                    let val_ty = self.infer_expr_type(v);
                    let mut val_llvm_ty = self.llvm_type(&val_ty);
                    let mut final_reg = val_reg;
                    if self.current_ret_type == "i32" && val_llvm_ty == "i64" {
                        let cast_reg = self.next_reg();
                        self.body.push_str(&format!(
                            "    {} = trunc i64 {} to i32\n",
                            cast_reg, final_reg
                        ));
                        final_reg = cast_reg;
                        val_llvm_ty = "i32".to_string();
                    }
                    Some((val_llvm_ty, final_reg))
                } else {
                    None
                };

                self.gen_deferred_calls();

                if let Some((val_llvm_ty, final_reg)) = final_reg_opt {
                    self.body.push_str(&format!(
                        "    ret {} {}\n",
                        val_llvm_ty, final_reg
                    ));
                } else {
                    if self.current_ret_type == "void" {
                        self.body.push_str("    ret void\n");
                    } else if self.current_ret_type == "i32" {
                        self.body.push_str("    ret i32 0\n");
                    } else if self.current_ret_type == "double" {
                        self.body.push_str("    ret double 0.0\n");
                    } else if self.current_ret_type == "ptr" {
                        self.body.push_str("    ret ptr null\n");
                    } else {
                        self.body.push_str(&format!(
                            "    ret {} 0\n",
                            self.current_ret_type
                        ));
                    }
                }
            }
            Stmt::Expr(e) => {
                self.gen_expr(e);
            }
            Stmt::Defer(expr) => {
                self.deferred_calls.push(expr.clone());
            }
            Stmt::Guard { cond, else_branch } => {
                let cond_reg = self.gen_expr(cond);
                let cond_ty = self.infer_expr_type(cond);
                let cond_llvm_ty = self.llvm_type(&cond_ty);
                let cmp_reg = self.next_reg();

                if cond_llvm_ty == "ptr" {
                    self.body.push_str(&format!(
                        "    {} = icmp ne ptr {}, null\n",
                        cmp_reg, cond_reg
                    ));
                } else {
                    self.body.push_str(&format!(
                        "    {} = icmp ne i64 {}, 0\n",
                        cmp_reg, cond_reg
                    ));
                }

                let else_lbl = self.next_label("guard_else");
                let merge_lbl = self.next_label("guard_merge");

                self.body.push_str(&format!(
                    "    br i1 {}, label %{}, label %{}\n\n{}:\n",
                    cmp_reg, merge_lbl, else_lbl, else_lbl
                ));

                self.gen_block(else_branch);
                if !block_has_terminator(else_branch) {
                    self.body.push_str("    unreachable\n");
                }

                self.body.push_str(&format!("\n{}:\n", merge_lbl));
            }
        }
    }
}
