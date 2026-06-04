use std::process::Command;

fn nub() -> Command {
    Command::new(env!("CARGO_BIN_EXE_nub"))
}

#[test]
fn auth_help_lists_all_subcommands() {
    let output = nub()
        .args(["auth", "--help"])
        .output()
        .expect("failed to run nub");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    for sub in ["login", "logout", "status", "setup-git", "git-credential"] {
        assert!(stdout.contains(sub), "auth help should list `{sub}`");
    }
}

#[test]
fn repo_help_lists_all_subcommands() {
    let output = nub()
        .args(["repo", "--help"])
        .output()
        .expect("failed to run nub");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-utf8 stdout");
    for sub in ["create", "clone", "list", "view"] {
        assert!(stdout.contains(sub), "repo help should list `{sub}`");
    }
}

#[test]
fn unimplemented_leaf_exits_with_not_implemented_code() {
    let output = nub()
        .args(["repo", "create"])
        .output()
        .expect("failed to run nub");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn global_flags_are_accepted_at_leaf_level() {
    // The leaf is not implemented yet, but the global flags must still parse
    // (a parse failure would exit with clap's usage code 2, not 3).
    let output = nub()
        .args(["repo", "list", "--json", "--no-color"])
        .output()
        .expect("failed to run nub");
    assert_eq!(output.status.code(), Some(3));
}
