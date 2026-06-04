use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_nub(args: &[&str], envs: &[(&str, &str)], stdin: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nub"));
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (key, value) in envs {
        cmd.env(key, value);
    }

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
fn setup_git_writes_credential_helper() {
    let config_dir = tempfile::tempdir().expect("config dir");
    let git_home = tempfile::tempdir().expect("git home");
    let git_config = git_home.path().join("gitconfig");
    let config_dir_s = config_dir.path().to_str().expect("utf8 config dir");
    let git_config_s = git_config.to_str().expect("utf8 git config");

    let out = run_nub(
        &["auth", "setup-git", "--host", "api.example"],
        &[
            ("NUB_CONFIG_DIR", config_dir_s),
            ("NUB_NO_KEYCHAIN", "1"),
            ("GIT_CONFIG_GLOBAL", git_config_s),
        ],
        None,
    );
    assert!(
        out.status.success(),
        "setup-git failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let written = fs::read_to_string(&git_config).expect("read gitconfig");
    assert!(
        written.contains("git.example"),
        "missing git host: {written}"
    );
    assert!(
        written.contains("auth git-credential"),
        "missing helper command: {written}"
    );
}

#[test]
fn git_credential_get_returns_stored_token() {
    let config_dir = tempfile::tempdir().expect("config dir");
    let config_dir_s = config_dir.path().to_str().expect("utf8 config dir");
    let env = [("NUB_CONFIG_DIR", config_dir_s), ("NUB_NO_KEYCHAIN", "1")];

    let login = run_nub(
        &["auth", "login", "--with-token", "--host", "api.example"],
        &env,
        Some("nub_pat_secret\n"),
    );
    assert!(
        login.status.success(),
        "login failed: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let out = run_nub(
        &["auth", "git-credential", "get", "--host", "api.example"],
        &env,
        Some("protocol=https\nhost=git.example\n\n"),
    );
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("username=x-access-token"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("password=nub_pat_secret"),
        "stdout: {stdout}"
    );
}

#[test]
fn git_credential_store_and_erase_are_noops() {
    let config_dir = tempfile::tempdir().expect("config dir");
    let config_dir_s = config_dir.path().to_str().expect("utf8 config dir");
    let env = [("NUB_CONFIG_DIR", config_dir_s), ("NUB_NO_KEYCHAIN", "1")];

    for op in ["store", "erase"] {
        let out = run_nub(
            &["auth", "git-credential", op, "--host", "api.example"],
            &env,
            Some("protocol=https\nhost=git.example\nusername=x\npassword=y\n\n"),
        );
        assert!(out.status.success(), "{op} failed");
        assert!(out.stdout.is_empty(), "{op} produced output");
    }
}
