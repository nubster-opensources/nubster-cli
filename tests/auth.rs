use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run_nub(args: &[&str], config_dir: &Path, stdin: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nub"));
    cmd.args(args)
        .env("NUB_CONFIG_DIR", config_dir)
        .env("NUB_NO_KEYCHAIN", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(input) = stdin {
        cmd.stdin(Stdio::piped());
        let mut child = cmd.spawn().expect("spawn nub");
        child
            .stdin
            .take()
            .expect("child stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
        child.wait_with_output().expect("wait for nub")
    } else {
        cmd.stdin(Stdio::null());
        cmd.output().expect("run nub")
    }
}

#[test]
fn login_status_logout_lifecycle() {
    let dir = tempfile::tempdir().expect("tempdir");

    let login = run_nub(
        &["auth", "login", "--with-token", "--host", "cli.example"],
        dir.path(),
        Some("nub_pat_secret\n"),
    );
    assert!(
        login.status.success(),
        "login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let status = run_nub(
        &["auth", "status", "--host", "cli.example"],
        dir.path(),
        None,
    );
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("Logged in"));

    let logout = run_nub(
        &["auth", "logout", "--host", "cli.example"],
        dir.path(),
        None,
    );
    assert!(logout.status.success());

    let after = run_nub(
        &["auth", "status", "--host", "cli.example"],
        dir.path(),
        None,
    );
    assert_eq!(after.status.code(), Some(4));
}

#[test]
fn login_without_with_token_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = run_nub(
        &["auth", "login", "--host", "cli.example"],
        dir.path(),
        None,
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn login_with_empty_stdin_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = run_nub(
        &["auth", "login", "--with-token", "--host", "cli.example"],
        dir.path(),
        Some("   \n"),
    );
    assert_eq!(output.status.code(), Some(1));
}
