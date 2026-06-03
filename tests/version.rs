use std::process::Command;

/// `nub --version` exits successfully and prints the crate version.
#[test]
fn version_flag_succeeds_and_prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_nub"))
        .arg("--version")
        .output()
        .expect("failed to run nub");

    assert!(output.status.success(), "nub --version should exit 0");

    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "version output should contain the crate version"
    );
}
