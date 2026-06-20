use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TestResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

// Helper to get unique paths for isolation
fn get_temp_paths(suffix: &str) -> (PathBuf, PathBuf, String) {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();
    let temp_dir = std::env::temp_dir().join(format!("n0ne_prod_test_{:?}_{}_{}", tid, id, suffix));
    let _ = fs::create_dir_all(&temp_dir);
    let name = format!("test_{}", id);
    let src_path = temp_dir.join(format!("{}.n0", name));
    (temp_dir, src_path, name)
}

fn wrap_source_if_needed(source: &str) -> String {
    if source.contains("task main") || source.contains("fn ") || source.contains("task ") || source.contains("type ") || source.contains("use ") || source.contains("const ") {
        source.to_string()
    } else {
        let mut wrapped = String::new();
        wrapped.push_str("task main\n");
        for line in source.lines() {
            if line.trim().is_empty() {
                wrapped.push_str("\n");
            } else {
                wrapped.push_str("    ");
                wrapped.push_str(line);
                wrapped.push_str("\n");
            }
        }
        wrapped
    }
}

// Compile n0ne source string to binary and run it
pub fn run_n0ne(source: &str) -> TestResult {
    let source = wrap_source_if_needed(source);
    let (temp_dir, src_path, name) = get_temp_paths("run");
    fs::write(&src_path, source).unwrap();

    let cli_path = env!("CARGO_BIN_EXE_n0ne");
    let build_output = Command::new(cli_path)
        .arg("build")
        .arg(&src_path)
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&build_output.stdout).to_string();
        let exit_code = build_output.status.code().unwrap_or(1);
        let _ = fs::remove_dir_all(&temp_dir);
        return TestResult { stdout, stderr, exit_code };
    }

    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name
    };
    let exe_path = temp_dir.join("build").join(&exe_name);

    let run_output = Command::new(&exe_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
    let exit_code = run_output.status.code().unwrap_or(1);

    let _ = fs::remove_dir_all(&temp_dir);

    TestResult { stdout, stderr, exit_code }
}

// Compile only, return error messages if any
pub fn compile_n0ne(source: &str) -> Result<(), Vec<String>> {
    let source = wrap_source_if_needed(source);
    let (temp_dir, src_path, _name) = get_temp_paths("compile");
    fs::write(&src_path, &source).unwrap();

    let cli_path = env!("CARGO_BIN_EXE_n0ne");
    let output = Command::new(cli_path)
        .arg("build")
        .arg(&src_path)
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    let _ = fs::remove_dir_all(&temp_dir);

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let errors = stderr.lines().map(|l| l.to_string()).filter(|l| !l.is_empty()).collect();
        Err(errors)
    }
}

// Compile to binary path for performance size testing
pub fn compile_to_binary(source: &str) -> (PathBuf, PathBuf) {
    let source = wrap_source_if_needed(source);
    let (temp_dir, src_path, name) = get_temp_paths("perf");
    fs::write(&src_path, &source).unwrap();

    let cli_path = env!("CARGO_BIN_EXE_n0ne");
    let build_output = Command::new(cli_path)
        .arg("build")
        .arg(&src_path)
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(
        build_output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name
    };
    let exe_path = temp_dir.join("build").join(&exe_name);
    (temp_dir, exe_path)
}

// Assert stdout equals expected exactly (normalized line endings)
pub fn assert_output(result: &TestResult, expected: &str) {
    assert_eq!(result.exit_code, 0, "Execution failed with exit code: {}\nStderr:\n{}", result.exit_code, result.stderr);
    let norm_stdout = result.stdout.replace("\r\n", "\n");
    let norm_expected = expected.replace("\r\n", "\n");
    assert_eq!(norm_stdout, norm_expected);
}

// Assert compile fails with specific error code
pub fn assert_error(source: &str, error_code: &str) {
    match compile_n0ne(source) {
        Ok(_) => panic!("Compilation succeeded, but expected failure with code {}", error_code),
        Err(errs) => {
            let combined = errs.join("\n");
            assert!(
                combined.contains(error_code),
                "Expected error code '{}', but got errors:\n{}",
                error_code,
                combined
            );
        }
    }
}

// Assert compile fails with message containing string
pub fn assert_error_contains(source: &str, msg: &str) {
    match compile_n0ne(source) {
        Ok(_) => panic!("Compilation succeeded, but expected failure containing: {}", msg),
        Err(errs) => {
            let combined = errs.join("\n");
            assert!(
                combined.contains(msg),
                "Expected error containing '{}', but got errors:\n{}",
                msg,
                combined
            );
        }
    }
}

// Clean up temp files after each test
pub fn cleanup(path: &str) {
    let p = Path::new(path);
    if p.exists() {
        if p.is_dir() {
            let _ = fs::remove_dir_all(p);
        } else {
            let _ = fs::remove_file(p);
        }
    }
}
