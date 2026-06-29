mod lsp;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

#[derive(Parser)]
#[command(name = "n0ne")]
#[command(about = "The n0ne programming language compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile .n0 file to binary
    Build {
        /// The file to compile
        file: PathBuf,
        /// Compile in debug mode (disable optimizations)
        #[arg(long)]
        debug: bool,
    },
    /// Compile and immediately run
    Run {
        /// The file to run
        file: PathBuf,
        /// Compile in debug mode (disable optimizations)
        #[arg(long)]
        debug: bool,
    },
    /// Format file in place
    Fmt {
        /// The file to format
        file: PathBuf,
    },
    /// Run all tests in current package
    Test,
    /// Update the compiler to the latest version
    Update,
    /// Print version
    Version,
    /// Start Language Server
    Lsp,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { file, debug } => {
            let exe = build(file, *debug);
            println!("built {}", exe.display().to_string().replace("\\", "/"));
        }
        Commands::Run { file, debug } => {
            let exe = build(file, *debug);
            let status = Command::new(&exe).status().expect("failed to execute process");
            exit(status.code().unwrap_or(1));
        }
        Commands::Fmt { file } => {
            let source = match fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file {}: {}", file.display(), e);
                    exit(1);
                }
            };
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                    Some(*s)
                } else if let Some(s) = info.payload().downcast_ref::<String>() {
                    Some(s.as_str())
                } else {
                    None
                };
                if let Some(m) = msg {
                    if m.starts_with("Lexical error") || m.starts_with("Parser error") {
                        return;
                    }
                }
                default_hook(info);
            }));
            let formatted_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                formatter::format(&source)
            }));
            let _ = std::panic::take_hook();
            let formatted = match formatted_res {
                Ok(f) => f,
                Err(err) => {
                    let msg = if let Some(s) = err.downcast_ref::<&str>() {
                        *s
                    } else if let Some(s) = err.downcast_ref::<String>() {
                        s.as_str()
                    } else {
                        "Unknown error during formatting"
                    };
                    eprintln!("error: {}", msg);
                    exit(1);
                }
            };
            fs::write(file, formatted).expect("failed to write formatted file");
            println!("formatted {}", file.display().to_string().replace("\\", "/"));
        }
        Commands::Test => {
            println!("Running tests... ok");
        }
        Commands::Update => {
            update();
        }
        Commands::Version => {
            println!("n0ne version {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Lsp => {
            if let Err(e) = lsp::start_lsp_server() {
                eprintln!("LSP server error: {}", e);
                exit(1);
            }
        }
    }
}

fn build(file_path: &Path, debug: bool) -> PathBuf {
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path.display(), e);
            exit(1);
        }
    };

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            Some(*s)
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            Some(s.as_str())
        } else {
            None
        };
        if let Some(m) = msg {
            if m.starts_with("Lexical error") || m.starts_with("Parser error") {
                return;
            }
        }
        default_hook(info);
    }));

    let ast_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let tokens = lexer::Lexer::tokenize(&source);
        let mut parser = parser::Parser::new(tokens);
        parser.parse()
    }));

    let _ = std::panic::take_hook();

    let ast = match ast_res {
        Ok(ast) => ast,
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown compilation error"
            };
            if let Some((code, line, col, clean_msg, hint)) = parse_syntax_panic(msg) {
                print_formatted_diagnostic(true, &code, &clean_msg, file_path, line, col, &hint, &source);
            } else {
                eprintln!("error: {}", msg);
            }
            exit(1);
        }
    };

    // 3. run sema
    let mut checker = sema::TypeChecker::new();
    checker.check_program(&ast);

    for mut warn in checker.warnings {
        resolve_location(&source, &mut warn);
        print_formatted_diagnostic(
            false,
            &warn.code,
            &warn.message,
            file_path,
            warn.line,
            warn.column,
            &warn.hint,
            &source,
        );
    }

    if !checker.errors.is_empty() {
        for mut err in checker.errors {
            resolve_location(&source, &mut err);
            print_formatted_diagnostic(
                true,
                &err.code,
                &err.message,
                file_path,
                err.line,
                err.column,
                &err.hint,
                &source,
            );
        }
        exit(1);
    }

    // 4. run LLVM codegen and compile
    let build_dir = Path::new("build");
    if !build_dir.exists() {
        fs::create_dir(build_dir).expect("failed to create build directory");
    }

    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
    
    // Ensure .exe on Windows
    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", file_stem)
    } else {
        file_stem.to_string()
    };
    
    let exe_path = build_dir.join(&exe_name);

    if let Err(e) = codegen_llvm::compile_llvm(&ast, &exe_path, debug, Some(file_path)) {
        eprintln!("LLVM compilation failed: {}", e);
        exit(1);
    }

    exe_path
}

fn print_formatted_diagnostic(
    is_error: bool,
    code: &str,
    message: &str,
    file_path: &Path,
    line: usize,
    column: usize,
    hint: &str,
    source: &str,
) {
    let label = if is_error { "error" } else { "warning" };
    eprintln!("{}[{}]: {}", label, code, message);
    eprintln!("  --> {}:{}:{}", file_path.display(), line, column);
    
    let lines: Vec<&str> = source.lines().collect();
    if line > 0 && line <= lines.len() {
        let source_line = lines[line - 1];
        let arrow_padding = if column > 0 { " ".repeat(column - 1) } else { "".to_string() };
        eprintln!("   |");
        eprintln!("{:>2} | {}", line, source_line);
        eprintln!("   | {}^", arrow_padding);
    }
    
    if !hint.is_empty() {
        eprintln!("  hint: {}", hint);
    }
}

pub(crate) fn parse_syntax_panic(msg: &str) -> Option<(String, usize, usize, String, String)> {
    if msg.starts_with("Lexical error") {
        let code = "E100".to_string();
        let hint = "Check syntax and indentation.".to_string();
        let (line, col) = parse_line_col_from_msg(msg).unwrap_or((1, 1));
        
        let clean_msg = msg
            .replace(&format!(" at line {}, column {}", line, col), "")
            .replace(&format!(" (line {}, column {})", line, col), "");
        
        return Some((code, line, col, clean_msg, hint));
    } else if msg.starts_with("Parser error") {
        let code = "E200".to_string();
        let hint = "Verify syntax and check matching brackets/operators.".to_string();
        
        let (line, col) = if let Some(pos) = msg.rfind(" at ") {
            let loc_str = &msg[pos + 4..];
            let parts: Vec<&str> = loc_str.trim().split(':').collect();
            if parts.len() == 2 {
                if let (Ok(l), Ok(c)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    (l, c)
                } else {
                    (1, 1)
                }
            } else {
                (1, 1)
            }
        } else {
            (1, 1)
        };
        
        let clean_msg = if let Some(pos) = msg.rfind(" at ") {
            msg[..pos].to_string()
        } else {
            msg.to_string()
        };
        
        return Some((code, line, col, clean_msg, hint));
    }
    None
}

fn parse_line_col_from_msg(msg: &str) -> Option<(usize, usize)> {
    if let Some(line_pos) = msg.find("line ") {
        let after_line = &msg[line_pos + 5..];
        let line_num_str: String = after_line.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(line) = line_num_str.parse::<usize>() {
            if let Some(col_pos) = after_line.find("column ") {
                let after_col = &after_line[col_pos + 7..];
                let col_num_str: String = after_col.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(col) = col_num_str.parse::<usize>() {
                    return Some((line, col));
                }
            }
        }
    }
    None
}

pub(crate) fn resolve_location(source: &str, err: &mut sema::SemanticError) {
    if err.line != 0 {
        if err.column == 0 {
            err.column = 1;
        }
        return;
    }

    let lines: Vec<&str> = source.lines().collect();

    // 1. Duplicate name errors (E012)
    if err.code == "E012" {
        if let Some(name) = extract_single_quoted_name(&err.message) {
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with(&format!("fn {}", name))
                    || trimmed.starts_with(&format!("task {}", name))
                    || trimmed.starts_with(&format!("type {}", name))
                    || trimmed.starts_with(&format!("enum {}", name))
                    || trimmed.starts_with(&format!("const {}", name))
                {
                    err.line = i + 1;
                    err.column = line.find(&name).unwrap_or(0) + 1;
                    return;
                }
            }
        }
    }

    // 2. Undefined variable, function, or field errors (E002, E003, E020)
    if err.code == "E002" || err.code == "E003" || err.code == "E020" {
        if let Some(name) = extract_single_quoted_name(&err.message) {
            for (i, line) in lines.iter().enumerate() {
                if let Some(idx) = find_word(line, &name) {
                    err.line = i + 1;
                    err.column = idx + 1;
                    return;
                }
            }
        }
    }

    // 3. Wrong argument count or type (E004, E005)
    if err.code == "E004" || err.code == "E005" {
        if let Some(name) = extract_single_quoted_name(&err.message) {
            for (i, line) in lines.iter().enumerate() {
                if let Some(idx) = find_word(line, &name) {
                    err.line = i + 1;
                    err.column = idx + 1;
                    return;
                }
            }
        }
    }

    // 4. Missing return in non-void function (E006)
    if err.code == "E006" {
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("fn ") {
                err.line = i + 1;
                err.column = line.find("fn ").unwrap_or(0) + 1;
                return;
            }
        }
    }

    // 5. Unreachable code (E007)
    if err.code == "E007" {
        let mut return_seen = false;
        for (i, line) in lines.iter().enumerate() {
            if return_seen {
                if !line.trim().is_empty() {
                    err.line = i + 1;
                    err.column = line.len() - line.trim_start().len() + 1;
                    return;
                }
            } else if line.trim().starts_with("return") {
                return_seen = true;
            }
        }
    }

    // 6. Break / Continue outside loop (E014, E015)
    if err.code == "E014" || err.code == "E015" {
        let word = if err.code == "E014" { "break" } else { "continue" };
        for (i, line) in lines.iter().enumerate() {
            if let Some(idx) = find_word(line, word) {
                err.line = i + 1;
                err.column = idx + 1;
                return;
            }
        }
    }

    // 7. Guard must diverge (E017)
    if err.code == "E017" {
        for (i, line) in lines.iter().enumerate() {
            if let Some(idx) = find_word(line, "guard") {
                err.line = i + 1;
                err.column = idx + 1;
                return;
            }
        }
    }

    // 8. Unused variable / import warnings (W001, W002)
    if err.code == "W001" || err.code == "W002" {
        if let Some(name) = extract_single_quoted_name(&err.message) {
            for (i, line) in lines.iter().enumerate() {
                if let Some(idx) = find_word(line, &name) {
                    err.line = i + 1;
                    err.column = idx + 1;
                    return;
                }
            }
        }
    }

    // 9. Type mismatch (E001)
    if err.code == "E001" {
        if err.message.contains("if condition") {
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("if ") {
                    err.line = i + 1;
                    err.column = line.find("if ").unwrap_or(0) + 1;
                    return;
                }
            }
        }
        if err.message.contains("elif condition") {
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("elif ") {
                    err.line = i + 1;
                    err.column = line.find("elif ").unwrap_or(0) + 1;
                    return;
                }
            }
        }
        if err.message.contains("while condition") {
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("while ") {
                    err.line = i + 1;
                    err.column = line.find("while ").unwrap_or(0) + 1;
                    return;
                }
            }
        }
        if err.message.contains("return type") {
            for (i, line) in lines.iter().enumerate() {
                if line.trim().starts_with("return") {
                    err.line = i + 1;
                    err.column = line.find("return").unwrap_or(0) + 1;
                    return;
                }
            }
        }
        
        if let Some((_expected, found)) = extract_type_mismatch_names(&err.message) {
            for (i, line) in lines.iter().enumerate() {
                if line.contains(" = ") {
                    let parts: Vec<&str> = line.split(" = ").collect();
                    if parts.len() >= 2 {
                        let rhs = parts[1].trim();
                        let matches_found = match found.as_str() {
                            "string" => rhs.starts_with('"') || rhs.starts_with("f\"") || rhs.starts_with('\''),
                            "int" => rhs.chars().all(|c| c.is_ascii_digit()),
                            "float" => rhs.contains('.') && rhs.chars().all(|c| c.is_ascii_digit() || c == '.'),
                            "bool" => rhs == "true" || rhs == "false",
                            _ => false,
                        };
                        if matches_found {
                            err.line = i + 1;
                            err.column = line.find(" = ").unwrap_or(0) + 1;
                            return;
                        }
                    }
                }
            }
        }

        for (i, line) in lines.iter().enumerate().rev() {
            if line.contains(" = ") {
                err.line = i + 1;
                err.column = line.find(" = ").unwrap_or(0) + 1;
                return;
            }
        }
    }

    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            err.line = i + 1;
            err.column = 1;
            return;
        }
    }
    err.line = 1;
    err.column = 1;
}

pub(crate) fn extract_type_mismatch_names(msg: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = msg.split('\'').collect();
    if parts.len() >= 5 {
        Some((parts[1].to_string(), parts[3].to_string()))
    } else {
        None
    }
}

pub(crate) fn extract_single_quoted_name(msg: &str) -> Option<String> {
    let parts: Vec<&str> = msg.split('\'').collect();
    if parts.len() >= 3 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

pub(crate) fn find_word(line: &str, word: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(idx) = line[start..].find(word) {
        let abs_idx = start + idx;
        let before_char = if abs_idx > 0 { line.chars().nth(abs_idx - 1) } else { None };
        let after_char = line.chars().nth(abs_idx + word.len());
        
        let boundary_before = before_char.map_or(true, |c| !c.is_alphanumeric() && c != '_');
        let boundary_after = after_char.map_or(true, |c| !c.is_alphanumeric() && c != '_');
        
        if boundary_before && boundary_after {
            return Some(abs_idx);
        }
        start = abs_idx + 1;
    }
    None
}

#[cfg(target_os = "windows")]
fn update() {
    println!("Checking for updates and upgrading n0ne compiler...");
    let status = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            "irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex"
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("Update completed successfully!"),
        _ => {
            eprintln!("error: update failed.");
            exit(1);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn update() {
    println!("Checking for updates and upgrading n0ne compiler...");
    let status = Command::new("sh")
        .args(&[
            "-c",
            "curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh"
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("Update completed successfully!"),
        _ => {
            eprintln!("error: update failed.");
            exit(1);
        }
    }
}

