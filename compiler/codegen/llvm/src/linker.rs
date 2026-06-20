use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs::File;
use std::io::Write;
use ast::Program;
use crate::LLVMGenerator;
use crate::runtime::RUNTIME_C;

pub(crate) fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

pub(crate) fn get_clang_path() -> PathBuf {
    let filename = if cfg!(target_os = "windows") { "clang.exe" } else { "clang" };
    let bundled_bin = exe_dir().join("clang").join("bin").join(filename);
    if bundled_bin.exists() {
        return bundled_bin;
    }
    let bundled_root = exe_dir().join("clang").join(filename);
    if bundled_root.exists() {
        return bundled_root;
    }

    let mut prefixes = Vec::new();
    for var in &["LLVM_SYS_221_PREFIX", "LLVM_SYS_220_PREFIX", "LLVM_SYS_201_PREFIX", "LLVM_SYS_200_PREFIX", "LLVM_SYS_190_PREFIX", "LLVM_SYS_180_PREFIX", "LLVM_SYS_170_PREFIX"] {
        if let Ok(prefix) = std::env::var(var) {
            prefixes.push(prefix);
        }
    }

    // Windows default paths
    prefixes.push("C:\\LLVM".to_string());
    prefixes.push("C:\\Program Files\\LLVM".to_string());

    // Linux / macOS — /usr/bin/clang is a symlink to whatever version is installed
    prefixes.push("/usr".to_string());
    prefixes.push("/usr/local".to_string());
    prefixes.push("/opt/homebrew".to_string());         // macOS Homebrew ARM
    prefixes.push("/usr/local/opt/llvm".to_string());  // macOS Homebrew Intel

    for prefix in prefixes {
        let p = Path::new(&prefix).join("bin").join(if cfg!(target_os = "windows") { "clang.exe" } else { "clang" });
        if p.exists() {
            return p;
        }
    }

    // Last resort: rely on PATH
    PathBuf::from("clang")
}

pub(crate) fn which_in_path(cmd: &str) -> bool {
    let cmd_exe = if cfg!(target_os = "windows") {
        format!("{}.exe", cmd)
    } else {
        cmd.to_string()
    };
    if let Ok(path_var) = std::env::var("PATH") {
        for path in std::env::split_paths(&path_var) {
            let p = path.join(&cmd_exe);
            if p.exists() {
                return true;
            }
        }
    }
    false
}

pub fn compile_llvm(ast: &Program, out_path: &Path, debug: bool) -> std::io::Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let mut generator = LLVMGenerator::new();
    let ir = generator.generate(ast);

    let unique_id = format!("{}_{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst));
    let temp_dir = std::env::temp_dir();
    let ll_path = temp_dir.join(format!("n0ne_{}.ll", unique_id));
    let runtime_c_path = temp_dir.join(format!("n0ne_rt_{}.c", unique_id));

    let mut f_ll = File::create(&ll_path)?;
    f_ll.write_all(ir.as_bytes())?;

    let mut f_runtime = File::create(&runtime_c_path)?;
    f_runtime.write_all(RUNTIME_C.as_bytes())?;

    let clang_path = get_clang_path();
    let exists = if clang_path.is_absolute() {
        clang_path.exists()
    } else {
        which_in_path("clang")
    };

    if !exists {
        eprintln!("error: clang not found.");
        eprintln!();
        eprintln!("  Linux / macOS:");
        eprintln!("    curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh");
        eprintln!();
        eprintln!("  Windows:");
        eprintln!("    irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex");
        std::process::exit(1);
    }

    let opt_flag = if debug { "-O0" } else { "-O2" };

    let mut cmd = Command::new(&clang_path);
    cmd.arg(&ll_path)
        .arg(&runtime_c_path)
        .arg("-o")
        .arg(out_path)
        .arg(opt_flag);

    if cfg!(target_os = "windows") {
        cmd.arg("-target").arg("x86_64-pc-windows-msvc");
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let _ = std::fs::remove_file(&ll_path);
        let _ = std::fs::remove_file(&runtime_c_path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "LLVM IR compilation and linking failed.\nStdout:\n{}\nStderr:\n{}",
                stdout, stderr
            ),
        ));
    }

    let _ = std::fs::remove_file(&ll_path);
    let _ = std::fs::remove_file(&runtime_c_path);

    Ok(())
}
