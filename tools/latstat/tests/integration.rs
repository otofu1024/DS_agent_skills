use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn binary_summarizes_two_runs_with_warmup_and_across_run_ci() {
    let output = Command::new(env!("CARGO_BIN_EXE_latstat"))
        .arg(fixture("run1.csv"))
        .arg(fixture("run2.csv"))
        .args(["--warmup", "1"])
        .output()
        .expect("latstat binary should start");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stdout.contains("Per-run summary (latency in ms)"));
    assert!(stdout.contains("Across-run summaries"));
    assert!(stdout.contains("latency_p50 | 2 | ms"));
    assert!(stderr.contains("fewer than 5 runs"));
}
