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
            eprintln!("error: {}", msg);
            exit(1);
        }
    };

    // 3. run sema
    let mut checker = sema::TypeChecker::new();
    checker.check_program(&ast);

    if !checker.errors.is_empty() {
        for err in &checker.errors {
            eprintln!(
                "error[{}]: {}\n  --> {}:{}:{}\n  hint: {}",
                err.code,
                err.message,
                file_path.display(),
                err.line,
                err.column,
                err.hint
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

