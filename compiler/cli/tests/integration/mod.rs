pub fn compile_and_run(source: &str) -> (String, i32) {
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();

    // Use a unique temp directory per test to avoid race conditions
    let temp_dir = std::env::temp_dir().join(format!("n0ne_test_{:?}_{}", tid, id));
    let _ = fs::create_dir_all(&temp_dir);

    let src_path = temp_dir.join(format!("test_{}.n0", id));
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
        let _ = fs::remove_dir_all(&temp_dir);
        return (stderr, build_output.status.code().unwrap_or(1));
    }

    let exe_name = if cfg!(target_os = "windows") {
        format!("test_{}.exe", id)
    } else {
        format!("test_{}", id)
    };
    let exe_path = temp_dir.join("build").join(&exe_name);

    let run_output = Command::new(&exe_path)
        .arg("args")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let code = run_output.status.code().unwrap_or(1);

    let _ = fs::remove_dir_all(&temp_dir);

    (stdout, code)
}

pub fn compile_and_run_with_files(files: Vec<(&str, &str)>) -> (String, i32) {
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();

    let temp_dir = std::env::temp_dir().join(format!("n0ne_test_files_{:?}_{}", tid, id));
    let _ = fs::create_dir_all(&temp_dir);

    let mut main_path = None;
    for (name, content) in files {
        let path = temp_dir.join(name);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, content).unwrap();
        if name.ends_with("main.n0") {
            main_path = Some(path);
        }
    }

    let src_path = main_path.unwrap_or_else(|| temp_dir.join("main.n0"));
    let cli_path = env!("CARGO_BIN_EXE_n0ne");

    let build_output = Command::new(cli_path)
        .arg("build")
        .arg(&src_path)
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr).to_string();
        let _ = fs::remove_dir_all(&temp_dir);
        return (stderr, build_output.status.code().unwrap_or(1));
    }

    let exe_name = if cfg!(target_os = "windows") {
        "main.exe".to_string()
    } else {
        "main".to_string()
    };
    let exe_path = temp_dir.join("build").join(&exe_name);

    let run_output = Command::new(&exe_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let code = run_output.status.code().unwrap_or(1);

    let _ = fs::remove_dir_all(&temp_dir);

    (stdout, code)
}

pub fn compile_and_run_with_stdin(source: &str, stdin: &str) -> (String, i32) {
    use std::fs;
    use std::process::Command;
    use std::process::Stdio;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();

    let temp_dir = std::env::temp_dir().join(format!("n0ne_test_stdin_{:?}_{}", tid, id));
    let _ = fs::create_dir_all(&temp_dir);

    let src_path = temp_dir.join(format!("test_{}.n0", id));
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
        let _ = fs::remove_dir_all(&temp_dir);
        return (stderr, build_output.status.code().unwrap_or(1));
    }

    let exe_name = if cfg!(target_os = "windows") {
        format!("test_{}.exe", id)
    } else {
        format!("test_{}", id)
    };
    let exe_path = temp_dir.join("build").join(&exe_name);

    let mut child = Command::new(&exe_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin.write_all(stdin.as_bytes()).unwrap();
    }

    let run_output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let code = run_output.status.code().unwrap_or(1);

    let _ = fs::remove_dir_all(&temp_dir);

    (stdout, code)
}
