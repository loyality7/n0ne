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
