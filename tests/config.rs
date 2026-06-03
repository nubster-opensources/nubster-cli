use std::process::Command;

fn nub() -> Command {
    Command::new(env!("CARGO_BIN_EXE_nub"))
}

#[test]
fn config_path_respects_env_override() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = nub()
        .args(["config", "path"])
        .env("NUB_CONFIG_DIR", dir.path())
        .output()
        .expect("failed to run nub");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    let printed = stdout.trim();
    let base = dir.path().display().to_string();

    assert!(
        printed.starts_with(base.as_str()),
        "path `{printed}` should be under the override dir"
    );
    assert!(printed.ends_with("config.toml"));
}

#[test]
fn config_show_reports_effective_host_from_flag() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = nub()
        .args(["config", "show", "--host", "cli.example"])
        .env("NUB_CONFIG_DIR", dir.path())
        .output()
        .expect("failed to run nub");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(stdout.contains("effective host: cli.example"));
}
