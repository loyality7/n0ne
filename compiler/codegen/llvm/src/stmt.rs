use ast::{Stmt, Expr, Type, AssignOp};
use crate::LLVMGenerator;
use crate::emitter::block_has_return;

impl LLVMGenerator {
    pub(crate) fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, op, value } => {
                let val_reg = self.gen_expr(value);
                let val_ty = self.infer_expr_type(value);
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
                if !block_has_return(then_branch) {
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
                    if !block_has_return(e_block) {
                        self.body.push_str(&format!("    br label %{}\n", cont_lbl));
                    }
                }

                if let Some(eb) = else_branch {
                    self.body.push_str(&format!("\n{}:\n", current_else_lbl));
                    self.gen_block(eb);
                    if !block_has_return(eb) {
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

                self.gen_block(body);

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
            Stmt::Return(opt_val) => {
                if let Some(v) = opt_val {
                    let val_reg = self.gen_expr(v);
                    let val_ty = self.infer_expr_type(v);
                    let val_llvm_ty = self.llvm_type(&val_ty);
                    self.body.push_str(&format!(
                        "    ret {} {}\n",
                        val_llvm_ty, val_reg
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
        }
    }
}
