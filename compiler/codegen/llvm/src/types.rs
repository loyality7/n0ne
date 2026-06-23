use ast::{Type, Expr, Literal};
use crate::LLVMGenerator;

impl LLVMGenerator {
    pub(crate) fn resolve_alias(&self, ty: &Type) -> Type {
        match ty {
            Type::Basic(name) => {
                if let Some(target) = self.aliases.get(name) {
                    self.resolve_alias(target)
                } else {
                    ty.clone()
                }
            }
            Type::List(inner) => Type::List(Box::new(self.resolve_alias(inner))),
            Type::Map(k, v) => Type::Map(Box::new(self.resolve_alias(k)), Box::new(self.resolve_alias(v))),
            Type::Result(inner) => Type::Result(Box::new(self.resolve_alias(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.resolve_alias(inner))),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.resolve_alias(t)).collect()),
            Type::Function(params, ret) => Type::Function(params.iter().map(|t| self.resolve_alias(t)).collect(), Box::new(self.resolve_alias(ret))),
        }
    }

    pub(crate) fn llvm_type(&self, ty: &Type) -> String {
        let res_ty = self.resolve_alias(ty);
        match &res_ty {
            Type::Basic(name) => match name.as_str() {
                "int" => "i64".to_string(),
                "bool" => "i64".to_string(),
                "float" => "double".to_string(),
                "string" => "ptr".to_string(),
                _ => "ptr".to_string(),
            },
            _ => "ptr".to_string(),
        }
    }

        pub(crate) fn infer_expr_type(&self, expr: &Expr) -> Type {
        let ty = self.infer_expr_type_inner(expr);
        self.resolve_alias(&ty)
    }

    fn infer_expr_type_inner(&self, expr: &Expr) -> Type {
        match expr {
            Expr::AnonymousFn { params, return_type, .. } => {
                let param_types = params.iter().map(|p| p.type_ann.clone()).collect();
                Type::Function(param_types, Box::new(return_type.clone().unwrap_or(Type::Basic("unknown".to_string()))))
            }
            Expr::Ident(name) => {
                if name == "none" {
                    return Type::Option(Box::new(Type::Basic("unknown".to_string())));
                }
                if let Some((enum_decl, _, _)) = self.find_variant(name) {
                    return Type::Basic(enum_decl.name.clone());
                }
                if let Some(f_decl) = self.functions.get(name) {
                    let param_types = f_decl.params.iter().map(|p| p.type_ann.clone()).collect();
                    Type::Function(param_types, Box::new(f_decl.return_type.clone().unwrap_or(Type::Basic("void".to_string()))))
                } else if let Some((_, ty)) = self.variables.get(name) {
                    ty.clone()
                } else if let Some(ty) = self.global_consts.get(name) {
                    ty.clone()
                } else {
                    Type::Basic("int".to_string())
                }
            }
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Type::Basic("int".to_string()),
                Literal::Float(_) => Type::Basic("float".to_string()),
                Literal::String(_) => Type::Basic("string".to_string()),
                Literal::Bool(_) => Type::Basic("bool".to_string()),
            },
            Expr::CallExpr { callee, args } => {
                if let Expr::Ident(name) = &**callee {
                    if let Some((enum_decl, _, _)) = self.find_variant(name) {
                        return Type::Basic(enum_decl.name.clone());
                    }
                    if name.starts_with("make_") {
                        let type_name = &name[5..];
                        let mut chars = type_name.chars();
                        if let Some(first) = chars.next() {
                            let capitalized = first.to_uppercase().collect::<String>() + chars.as_str();
                            return Type::Basic(capitalized);
                        }
                    }
                    if name == "c_alloc" {
                        return Type::Basic("unknown".to_string());
                    }
                    if name == "c_argv" || name == "c_interpolate" || name == "c_load_string" {
                        return Type::Basic("string".to_string());
                    }
                    if name == "ok" || name == "err" || name == "risky" {
                        return Type::Result(Box::new(Type::Basic("unknown".to_string())));
                    }
                    if name == "some" || name == "none" {
                        return Type::Option(Box::new(Type::Basic("unknown".to_string())));
                    } else if self.functions.contains_key(name) {
                        let f_decl = self.functions.get(name).unwrap();
                        let def_ty = Type::Basic("void".to_string());
                        return f_decl.return_type.as_ref().unwrap_or(&def_ty).clone();
                    }
                    if self.structs.contains_key(name) {
                        return Type::Basic(name.clone());
                    }
                } else if let Expr::FieldAccess { expr: receiver, field: method_name } = &**callee {
                    if method_name == "to_string" {
                        return Type::Basic("string".to_string());
                    }
                    let receiver_ty = self.infer_expr_type(receiver);
                    if let Type::Result(_) | Type::Option(_) = &receiver_ty {
                        if method_name == "is_err" || method_name == "is_ok" || method_name == "error" || method_name == "value" || method_name == "unwrap" || method_name == "is_some" || method_name == "is_none" {
                            let field_expr = Expr::FieldAccess {
                                expr: receiver.clone(),
                                field: method_name.clone(),
                            };
                            return self.infer_expr_type(&field_expr);
                        }
                    }
                    if let Expr::Ident(mod_name) = &**receiver {
                        if self.variables.get(mod_name).is_none() {
                            if mod_name == "io" || mod_name == "fs" || mod_name == "json" || mod_name == "http" {
                                match (mod_name.as_str(), method_name.as_str()) {
                                    ("io", "read") => return Type::Basic("string".to_string()),
                                    ("fs", "read") => return Type::Result(Box::new(Type::Basic("string".to_string()))),
                                    ("fs", "write") => return Type::Result(Box::new(Type::Basic("void".to_string()))),
                                    ("fs", "exists") => return Type::Basic("bool".to_string()),
                                    ("fs", "delete") => return Type::Result(Box::new(Type::Basic("void".to_string()))),
                                    ("fs", "mkdir") => return Type::Result(Box::new(Type::Basic("void".to_string()))),
                                    ("fs", "list") => return Type::Result(Box::new(Type::List(Box::new(Type::Basic("string".to_string()))))),
                                    ("json", "encode") => return Type::Basic("string".to_string()),
                                    ("json", "decode") => return Type::Result(Box::new(Type::Map(Box::new(Type::Basic("string".to_string())), Box::new(Type::Basic("string".to_string()))))),
                                                                         ("http", "get") => return Type::Result(Box::new(Type::Basic("string".to_string()))),
                                                                         ("http", "post") => return Type::Result(Box::new(Type::Basic("string".to_string()))),
                                     ("http", "get_json") => return Type::Result(Box::new(Type::Map(Box::new(Type::Basic("string".to_string())), Box::new(Type::Basic("string".to_string()))))),
                                    _ => {}
                                }
                            } else if let Some(f_decl) = self.functions.get(method_name) {
                                let def_ty = Type::Basic("void".to_string());
                                return f_decl.return_type.as_ref().unwrap_or(&def_ty).clone();
                            }
                        }
                    }

                    let receiver_ty = self.infer_expr_type(receiver);
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
                                "map" => {
                                    if let Some(arg) = args.get(0) {
                                        if let Type::Function(_, ret) = self.infer_expr_type(arg) {
                                            return Type::List(ret.clone());
                                        }
                                    }
                                    return Type::List(Box::new(Type::Basic("unknown".to_string())));
                                }
                                "filter" => return Type::List(inner.clone()),
                                "reduce" => {
                                    if let Some(arg) = args.get(0) {
                                        return self.infer_expr_type(arg);
                                    }
                                    return Type::Basic("unknown".to_string());
                                }
                                "find" => return Type::Option(inner.clone()),
                                "any" | "all" => return Type::Basic("bool".to_string()),
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
                        Type::Basic(name) if name == "HttpServer" => {
                            match method_name.as_str() {
                                "route" | "start" => return Type::Basic("void".to_string()),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Type::Basic("int".to_string())
            }
            Expr::FieldAccess { expr: inner, field } => {
                let inner_ty = self.infer_expr_type(inner);
                let type_name = match &inner_ty {
                    Type::Basic(n) => n.clone(),
                    Type::Result(_) => "result".to_string(),
                    Type::Option(_) => "option".to_string(),
                    _ => "unknown".to_string(),
                };
                if type_name == "result" && field == "is_err" {
                    Type::Basic("int".to_string())
                } else if type_name == "result" && field == "is_ok" {
                    Type::Basic("int".to_string())
                } else if type_name == "result" && field == "error" {
                    Type::Basic("string".to_string())
                } else if type_name == "result" && (field == "value" || field == "unwrap") {
                    match &inner_ty {
                        Type::Result(t) => (**t).clone(),
                        _ => Type::Basic("unknown".to_string()),
                    }
                } else if type_name == "option" && field == "is_some" {
                    Type::Basic("bool".to_string())
                } else if type_name == "option" && field == "is_none" {
                    Type::Basic("bool".to_string())
                } else if type_name == "option" && (field == "value" || field == "unwrap") {
                    match &inner_ty {
                        Type::Option(t) => (**t).clone(),
                        _ => Type::Basic("unknown".to_string()),
                    }
                } else if let Some(decl) = self.structs.get(&type_name) {
                    decl.fields.iter().find(|f| &f.name == field).map(|f| f.type_ann.clone()).unwrap_or(Type::Basic("unknown".to_string()))
                } else {
                    Type::Basic("unknown".to_string())
                }
            }
            Expr::TryExpr(inner) => {
                let inner_ty = self.infer_expr_type(inner);
                match inner_ty {
                    Type::Result(t) => *t,
                    _ => Type::Basic("unknown".to_string()),
                }
            }
            Expr::ListLiteral(items) => {
                let elem_type = if items.is_empty() {
                    Type::Basic("unknown".to_string())
                } else {
                    self.infer_expr_type(&items[0])
                };
                Type::List(Box::new(elem_type))
            }
            Expr::MapLiteral(pairs) => {
                let (key_type, val_type) = if pairs.is_empty() {
                    (Type::Basic("unknown".to_string()), Type::Basic("unknown".to_string()))
                } else {
                    (self.infer_expr_type(&pairs[0].0), self.infer_expr_type(&pairs[0].1))
                };
                Type::Map(Box::new(key_type), Box::new(val_type))
            }
            Expr::BinExpr { left, op, right, .. } => {
                let l_ty = self.infer_expr_type(left);
                let r_ty = self.infer_expr_type(right);
                match op {
                    ast::BinOp::Eq | ast::BinOp::Ne | ast::BinOp::Lt | ast::BinOp::Gt | ast::BinOp::Le | ast::BinOp::Ge | ast::BinOp::And | ast::BinOp::Or => {
                        Type::Basic("bool".to_string())
                    }
                    _ => {
                        if l_ty == Type::Basic("string".to_string()) || r_ty == Type::Basic("string".to_string()) {
                            Type::Basic("string".to_string())
                        } else if l_ty == Type::Basic("float".to_string()) || r_ty == Type::Basic("float".to_string()) {
                            Type::Basic("float".to_string())
                        } else {
                            Type::Basic("int".to_string())
                        }
                    }
                }
            }
            Expr::FStringExpr(_) => Type::Basic("string".to_string()),
            Expr::Tuple(items) => {
                let types = items.iter().map(|item| self.infer_expr_type(item)).collect();
                Type::Tuple(types)
            }
            Expr::Index { expr, index: _, line: _ } => {
                let col_ty = self.infer_expr_type(expr);
                match col_ty {
                    Type::List(elem_ty) => *elem_ty,
                    Type::Map(_, val_ty) => Type::Option(Box::new(*val_ty)),
                    _ => Type::Basic("unknown".to_string()),
                }
            }
            Expr::NamedArg { name: _, value } => self.infer_expr_type(value),
            _ => Type::Basic("int".to_string()),
        }
    }

    pub(crate) fn find_variant(&self, name: &str) -> Option<(ast::EnumDecl, ast::EnumVariant, i64)> {
        for e in self.enums.values() {
            for (idx, var) in e.variants.iter().enumerate() {
                if var.name == name {
                    return Some((e.clone(), var.clone(), idx as i64));
                }
            }
        }
        None
    }
}
