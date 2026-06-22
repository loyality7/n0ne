use ast::{Program, TypeDecl, Type};
use std::collections::HashMap;

pub(crate) mod emitter;
pub(crate) mod expr;
pub(crate) mod stmt;
pub(crate) mod types;
pub(crate) mod runtime;
pub(crate) mod linker;
pub(crate) mod builtins;
pub mod stdlib;

pub use linker::compile_llvm;

pub struct LLVMGenerator {
    pub(crate) globals: String,
    pub(crate) body: String,
    pub(crate) reg_counter: usize,
    pub(crate) label_counter: usize,
    pub(crate) string_counter: usize,
    pub(crate) string_constants: Vec<(String, String, usize)>, // (name, escaped_val, len)
    pub(crate) variables: HashMap<String, (String, Type)>,    // name -> (alloca_reg, type)
    pub(crate) structs: HashMap<String, TypeDecl>,
    pub(crate) enums: HashMap<String, ast::EnumDecl>,
    pub(crate) current_ret_type: String,
    pub(crate) functions: HashMap<String, ast::FnDecl>,
    pub(crate) global_consts: HashMap<String, Type>,
    pub(crate) loop_stack: Vec<(String, String)>, // (continue_lbl, break_lbl)
    pub(crate) compiled_files: std::collections::HashSet<std::path::PathBuf>,
    pub(crate) current_file: Option<std::path::PathBuf>,
}

impl LLVMGenerator {
    pub fn new() -> Self {
        Self {
            globals: String::new(),
            body: String::new(),
            reg_counter: 0,
            label_counter: 0,
            string_counter: 0,
            string_constants: Vec::new(),
            variables: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_ret_type: "void".to_string(),
            functions: HashMap::new(),
            global_consts: HashMap::new(),
            loop_stack: Vec::new(),
            compiled_files: std::collections::HashSet::new(),
            current_file: None,
        }
    }

    pub fn generate(&mut self, ast: &Program) -> String {
        // Collect structs, function return types, and global constants
        for decl in &ast.decls {
            match decl {
                ast::TopLevelDecl::TypeDecl(t) => {
                    self.structs.insert(t.name.clone(), t.clone());
                }
                ast::TopLevelDecl::EnumDecl(e) => {
                    self.enums.insert(e.name.clone(), e.clone());
                }
                ast::TopLevelDecl::FnDecl(f) => {
                    self.functions.insert(f.name.clone(), f.clone());
                }
                ast::TopLevelDecl::ConstDecl(c) => {
                    let val_ty = self.infer_expr_type(&c.value);
                    self.global_consts.insert(c.name.clone(), val_ty);
                }
                _ => {}
            }
        }

        // Add standard declarations
        self.globals.push_str("declare void @n0_show_string(ptr)\n");
        self.globals.push_str("declare void @n0_show_int(i64)\n");
        self.globals.push_str("declare void @n0_show_float(double)\n");
        self.globals.push_str("declare ptr @n0_c_alloc(i64)\n");
        self.globals.push_str("declare void @n0_c_store_int(ptr, i64, i64)\n");
        self.globals.push_str("declare void @n0_c_store_string(ptr, i64, ptr)\n");
        self.globals.push_str("declare i64 @n0_c_load_int(ptr, i64)\n");
        self.globals.push_str("declare ptr @n0_c_load_string(ptr, i64)\n");
        self.globals.push_str("declare ptr @n0_c_interpolate(ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_int_to_string(i64)\n");
        self.globals.push_str("declare ptr @n0_float_to_string(double)\n");
        self.globals.push_str("declare ptr @n0_bool_to_string(i64)\n");
        self.globals.push_str("declare void @n0_show_bool(i64)\n");
        self.globals.push_str("declare ptr @n0_string_concat(ptr, i32)\n");
        self.globals.push_str("declare i64 @n0_c_argc()\n");
        self.globals.push_str("declare ptr @n0_c_argv(i64)\n");
        self.globals.push_str("declare double @pow(double, double)\n");
        self.globals.push_str("declare ptr @n0_make_some(i64)\n");
        self.globals.push_str("declare ptr @n0_make_none()\n");
        self.globals.push_str("declare ptr @n0_make_ok(i64)\n");
        self.globals.push_str("declare ptr @n0_make_err(ptr)\n");
        
        // stdlib C runtime declarations
        self.globals.push_str("declare void @n0_show_err(ptr)\n");
        self.globals.push_str("declare ptr @n0_io_read_line()\n");
        self.globals.push_str("declare ptr @n0_fs_read(ptr)\n");
        self.globals.push_str("declare ptr @n0_fs_write(ptr, ptr)\n");
        self.globals.push_str("declare i1 @n0_fs_exists(ptr)\n");
        self.globals.push_str("declare ptr @n0_fs_delete(ptr)\n");
        self.globals.push_str("declare ptr @n0_fs_mkdir(ptr)\n");
        self.globals.push_str("declare ptr @n0_fs_list(ptr)\n");
        self.globals.push_str("declare ptr @n0_json_encode_string(ptr)\n");
        self.globals.push_str("declare ptr @n0_json_encode_int(i64)\n");
        self.globals.push_str("declare ptr @n0_json_encode_float(double)\n");
        self.globals.push_str("declare ptr @n0_json_encode_bool(i64)\n");
        self.globals.push_str("declare ptr @n0_json_encode_list(ptr)\n");
        self.globals.push_str("declare ptr @n0_json_encode_map(ptr)\n");
        self.globals.push_str("declare ptr @n0_json_decode(ptr)\n");
        self.globals.push_str("declare ptr @n0_http_get(ptr)\n");
        self.globals.push_str("declare ptr @n0_http_post(ptr, ptr)\n");
        
        // String primitive methods
        self.globals.push_str("declare i64 @n0_str_len(ptr)\n");
        self.globals.push_str("declare i64 @n0_str_contains(ptr, ptr)\n");
        self.globals.push_str("declare i64 @n0_str_starts_with(ptr, ptr)\n");
        self.globals.push_str("declare i64 @n0_str_ends_with(ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_str_upper(ptr)\n");
        self.globals.push_str("declare ptr @n0_str_lower(ptr)\n");
        self.globals.push_str("declare ptr @n0_str_trim(ptr)\n");
        self.globals.push_str("declare ptr @n0_str_split(ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_str_replace(ptr, ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_str_slice(ptr, i64, i64)\n");
        self.globals.push_str("declare ptr @n0_str_to_int(ptr)\n");
        self.globals.push_str("declare ptr @n0_str_to_float(ptr)\n");

        // List primitive methods
        self.globals.push_str("declare i64 @n0_list_len(ptr)\n");
        self.globals.push_str("declare void @n0_list_push(ptr, i64)\n");
        self.globals.push_str("declare ptr @n0_list_pop(ptr)\n");
        self.globals.push_str("declare i64 @n0_list_contains_int(ptr, i64)\n");
        self.globals.push_str("declare i64 @n0_list_contains_str(ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_list_first(ptr)\n");
        self.globals.push_str("declare ptr @n0_list_last(ptr)\n");

        // Map primitive methods
        self.globals.push_str("declare ptr @n0_map_get(ptr, ptr)\n");
        self.globals.push_str("declare void @n0_map_set(ptr, ptr, i64)\n");
        self.globals.push_str("declare i64 @n0_map_has(ptr, ptr)\n");
        self.globals.push_str("declare ptr @n0_map_keys(ptr)\n");
        self.globals.push_str("declare ptr @n0_map_values(ptr)\n");
        self.globals.push_str("declare void @n0_map_delete(ptr, ptr)\n");

        // Numeric conversion primitive methods
        self.globals.push_str("declare double @n0_int_to_float(i64)\n");
        self.globals.push_str("declare i64 @n0_float_to_int(double)\n");
        self.globals.push_str("declare i32 @strcmp(ptr, ptr)\n\n");

        self.globals.push_str("@global_argc = external global i32\n");
        self.globals.push_str("@global_argv = external global ptr\n\n");

        self.declare_stdlib();

        for decl in &ast.decls {
            self.gen_top_level(decl);
        }

        // Prepend string constants to output
        let mut final_out = String::new();
        for (name, escaped, len) in &self.string_constants {
            final_out.push_str(&format!(
                "{} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n",
                name, len, escaped
            ));
        }
        final_out.push_str("\n");
        final_out.push_str(&self.globals);
        final_out.push_str(&self.body);
        final_out
    }
}
