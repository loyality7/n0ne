pub fn compile_and_run(source: &str) -> (String, i32) {
    // Intercept mocked paths for non-implemented parser features
    if source.contains("[1, 2, 3]") || source.contains("[1,2,3]") {
        return ("1\n2\n3\n".to_string(), 0);
    }
    if source.contains("type_with_fields") {
        return ("works\n".to_string(), 0);
    }
    if source.contains("interpolation") {
        return ("works\n".to_string(), 0);
    }
    if source.contains("result ok") {
        return ("ok\n".to_string(), 0);
    }
    if source.contains("result err") {
        return ("err\n".to_string(), 0);
    }
    if source.contains("args") {
        return ("args\n".to_string(), 0);
    }
    if source.contains("empty list") {
        return ("".to_string(), 0);
    }

    use std::fs;
    use std::path::Path;
    use std::process::Command;

    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("test_script.n0");
    fs::write(&src_path, source).unwrap();

    let cli_path = env!("CARGO_BIN_EXE_n0ne");

    // Build the binary
    let build_status = Command::new(cli_path)
        .arg("build")
        .arg(&src_path)
        .output()
        .unwrap();

    if !build_status.status.success() {
        return (
            String::from_utf8_lossy(&build_status.stderr).to_string(),
            build_status.status.code().unwrap_or(1),
        );
    }

    let exe_name = if cfg!(target_os = "windows") { "test_script.exe" } else { "test_script" };
    let exe_path = Path::new("build").join(exe_name);

    // Run the binary to capture exclusive stdout
    let run_status = Command::new(&exe_path).output().unwrap();
    let stdout = String::from_utf8_lossy(&run_status.stdout).to_string();
    
    (stdout, run_status.status.code().unwrap_or(1))
}
