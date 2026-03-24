use std::process::Command;

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_devfetch").expect("missing binary path in CARGO_BIN_EXE_*")
}

fn run(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to run envinfo binary");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn help_contains_usage() {
    let out = run(&["--help"]);
    assert!(out.contains("Usage:"));
    assert!(out.contains("机器环境"));
}

#[test]
fn markdown_has_expected_heading_level() {
    let out = run(&["--system", "--markdown"]);
    assert!(out.contains("## System:"));
}

#[test]
fn toml_has_expected_table() {
    let out = run(&["--system", "--toml"]);
    assert!(out.contains("[System]"));
}

#[test]
fn helper_outputs_json_object() {
    let out = run(&["--helper", "Node"]);
    assert!(out.contains("\"Node\""));
    assert!(out.contains("\"version\""));
}

#[test]
fn report_no_longer_contains_project_scan_sections() {
    let out = run(&["--all", "--json"]);
    assert!(!out.contains("\"npmPackages\""));
    assert!(!out.contains("\"Monorepos\""));
}
